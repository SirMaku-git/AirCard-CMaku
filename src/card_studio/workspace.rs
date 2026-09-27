use anyhow::{anyhow, Context, Result};
use image::{DynamicImage, RgbaImage};
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::card_studio::types::*;
use crate::card_studio::widget::{CustomWidget, CustomWidgetData};
use crate::card_studio::CardStudioState;

pub fn load_image_any_format(bytes: &[u8], filename_hint: Option<&str>) -> Result<RgbaImage> {
    let is_svg = filename_hint.map(|h| h.to_lowercase().ends_with(".svg")).unwrap_or(false)
        || bytes.starts_with(b"<?xml")
        || bytes.starts_with(b"<svg")
        || (bytes.len() > 10 && bytes[..100.min(bytes.len())].windows(4).any(|w| w == b"<svg"));

    if is_svg {
        render_svg_to_rgba(bytes)
    } else {
        let dyn_img = image::load_from_memory(bytes)?;
        Ok(dyn_img.to_rgba8())
    }
}

pub fn render_svg_to_rgba(bytes: &[u8]) -> Result<RgbaImage> {
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(bytes, &opt)
        .map_err(|e| anyhow!("Failed to parse SVG vector graphic: {e}"))?;

    let size = tree.size();
    let src_w = size.width();
    let src_h = size.height();

    let target_w = src_w.round().clamp(1.0, 4096.0) as u32;
    let target_h = src_h.round().clamp(1.0, 4096.0) as u32;

    let sx = target_w as f32 / src_w;
    let sy = target_h as f32 / src_h;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(target_w, target_h)
        .ok_or_else(|| anyhow!("Failed to allocate SVG raster pixmap ({}x{})", target_w, target_h))?;

    let transform = resvg::tiny_skia::Transform::from_scale(sx, sy);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Convert premultiplied RGBA from tiny_skia to straight RGBA
    let mut out_img = RgbaImage::new(target_w, target_h);
    for (src, dst) in pixmap.pixels().iter().zip(out_img.pixels_mut()) {
        let a = src.alpha();
        if a == 0 {
            *dst = image::Rgba([0, 0, 0, 0]);
        } else {
            let r = ((src.red() as u32 * 255) / a as u32) as u8;
            let g = ((src.green() as u32 * 255) / a as u32) as u8;
            let b = ((src.blue() as u32 * 255) / a as u32) as u8;
            *dst = image::Rgba([r, g, b, a]);
        }
    }

    Ok(out_img)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceManifest {
    pub format: String,
    pub version: String,
    pub name: String,
    pub overlay_options: CardOverlayOptions,
    pub has_source_image: bool,
    pub has_custom_logo: bool,
    pub has_custom_chip: bool,
    pub has_custom_finish: bool,
    pub has_custom_font: bool,
    pub custom_font_name: Option<String>,
    pub widgets_data: Vec<CustomWidgetData>,
}

fn encode_rgba_to_png(img: &RgbaImage) -> Result<Vec<u8>> {
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png)?;
    Ok(buf.into_inner())
}

pub fn save_workspace_wcm(
    state: &CardStudioState,
    source_image: Option<&DynamicImage>,
    target_path: &Path,
) -> Result<()> {
    let file = File::create(target_path)
        .with_context(|| format!("Cannot create workspace file: {}", target_path.display()))?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let manifest = WorkspaceManifest {
        format: "AirCard-CMaku-Workspace".to_string(),
        version: "1.0".to_string(),
        name: target_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("card_workspace")
            .to_string(),
        overlay_options: state.overlay_options.clone(),
        has_source_image: source_image.is_some(),
        has_custom_logo: state.custom_logo_image.is_some(),
        has_custom_chip: state.custom_chip_image.is_some(),
        has_custom_finish: state.custom_finish_image.is_some(),
        has_custom_font: state.custom_font_bytes.is_some(),
        custom_font_name: state.custom_font_name.clone(),
        widgets_data: state.custom_widgets.iter().map(|w| w.data.clone()).collect(),
    };

    // 1. Write manifest.json
    zip.start_file("manifest.json", options)?;
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    zip.write_all(&manifest_bytes)?;

    // 2. Write source background image
    if let Some(src_img) = source_image {
        zip.start_file("source_image.png", options)?;
        let rgba = src_img.to_rgba8();
        let png_bytes = encode_rgba_to_png(&rgba)?;
        zip.write_all(&png_bytes)?;
    }

    // 3. Write custom logo
    if let Some(ref logo) = state.custom_logo_image {
        zip.start_file("custom_logo.png", options)?;
        let png_bytes = encode_rgba_to_png(logo)?;
        zip.write_all(&png_bytes)?;
    }

    // 4. Write custom chip
    if let Some(ref chip) = state.custom_chip_image {
        zip.start_file("custom_chip.png", options)?;
        let png_bytes = encode_rgba_to_png(chip)?;
        zip.write_all(&png_bytes)?;
    }

    // 5. Write custom finish texture
    if let Some(ref finish) = state.custom_finish_image {
        zip.start_file("custom_finish.png", options)?;
        let png_bytes = encode_rgba_to_png(finish)?;
        zip.write_all(&png_bytes)?;
    }

    // 6. Write custom font
    if let Some(ref font_bytes) = state.custom_font_bytes {
        zip.start_file("custom_font.ttf", options)?;
        zip.write_all(font_bytes)?;
    }

    // 7. Write custom widgets
    for (i, widget) in state.custom_widgets.iter().enumerate() {
        zip.start_file(format!("widget_{i}.png"), options)?;
        let png_bytes = encode_rgba_to_png(&widget.image)?;
        zip.write_all(&png_bytes)?;
    }

    zip.finish()?;
    Ok(())
}

pub fn load_workspace_wcm(
    state: &mut CardStudioState,
    source_path: &Path,
) -> Result<Option<DynamicImage>> {
    let file = File::open(source_path)
        .with_context(|| format!("Cannot open workspace file: {}", source_path.display()))?;
    let mut zip = ZipArchive::new(file)?;

    // 1. Read manifest.json
    let manifest: WorkspaceManifest = {
        let mut manifest_file = zip.by_name("manifest.json")
            .with_context(|| "Invalid .wcm workspace: missing manifest.json")?;
        let mut content = Vec::new();
        manifest_file.read_to_end(&mut content)?;
        serde_json::from_slice(&content)?
    };

    // 2. Restore overlay options
    state.overlay_options = manifest.overlay_options;
    state.overlay_options.details.ensure_items();

    // 3. Restore source background image
    let mut loaded_source_image: Option<DynamicImage> = None;
    if manifest.has_source_image {
        if let Ok(mut img_file) = zip.by_name("source_image.png") {
            let mut bytes = Vec::new();
            if img_file.read_to_end(&mut bytes).is_ok() {
                if let Ok(dyn_img) = image::load_from_memory(&bytes) {
                    loaded_source_image = Some(dyn_img);
                }
            }
        }
    }

    // 4. Restore custom logo
    state.custom_logo_image = None;
    if manifest.has_custom_logo {
        if let Ok(mut logo_file) = zip.by_name("custom_logo.png") {
            let mut bytes = Vec::new();
            if logo_file.read_to_end(&mut bytes).is_ok() {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    state.custom_logo_image = Some(img.to_rgba8());
                }
            }
        }
    }

    // 5. Restore custom chip
    state.custom_chip_image = None;
    if manifest.has_custom_chip {
        if let Ok(mut chip_file) = zip.by_name("custom_chip.png") {
            let mut bytes = Vec::new();
            if chip_file.read_to_end(&mut bytes).is_ok() {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    state.custom_chip_image = Some(img.to_rgba8());
                }
            }
        }
    }

    // 6. Restore custom finish texture
    state.custom_finish_image = None;
    if manifest.has_custom_finish {
        if let Ok(mut finish_file) = zip.by_name("custom_finish.png") {
            let mut bytes = Vec::new();
            if finish_file.read_to_end(&mut bytes).is_ok() {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    state.custom_finish_image = Some(img.to_rgba8());
                }
            }
        }
    }

    // 7. Restore custom font
    state.custom_font_bytes = None;
    state.custom_font_name = manifest.custom_font_name.clone();
    if manifest.has_custom_font {
        if let Ok(mut font_file) = zip.by_name("custom_font.ttf") {
            let mut bytes = Vec::new();
            if font_file.read_to_end(&mut bytes).is_ok() {
                state.custom_font_bytes = Some(bytes);
            }
        }
    }

    // 8. Restore custom widgets
    state.custom_widgets.clear();
    for (i, widget_data) in manifest.widgets_data.into_iter().enumerate() {
        let entry_name = format!("widget_{i}.png");
        if let Ok(mut w_file) = zip.by_name(&entry_name) {
            let mut bytes = Vec::new();
            if w_file.read_to_end(&mut bytes).is_ok() {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    state.custom_widgets.push(CustomWidget {
                        data: widget_data,
                        image: img.to_rgba8(),
                        path: None,
                    });
                }
            }
        }
    }

    state.selected_text_index = 0;
    Ok(loaded_source_image)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_loading() {
        let svg_data = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50">
            <rect width="100" height="50" fill="#ff0000" />
        </svg>"##;
        let img = load_image_any_format(svg_data.as_bytes(), Some("test.svg")).expect("SVG should load");
        assert_eq!(img.width(), 100);
        assert_eq!(img.height(), 50);
        let pixel = img.get_pixel(10, 10);
        assert_eq!(pixel[0], 255); // Red
        assert_eq!(pixel[3], 255); // Opaque
    }

    #[test]
    fn test_workspace_roundtrip() {
        let mut state = CardStudioState::new();
        state.overlay_options.bg_preset = CardBackgroundPreset::BrushedGold;
        state.overlay_options.details.show_details = true;
        let item_idx = state.add_text_item();
        state.overlay_options.details.items[item_idx].content = "WORKSPACE TEST".to_string();
        state.overlay_options.details.items[item_idx].has_backdrop = true;

        let temp_dir = std::env::temp_dir();
        let test_wcm_path = temp_dir.join("test_aircard_workspace.wcm");

        save_workspace_wcm(&state, None, &test_wcm_path).expect("Should save .wcm");

        let mut restored_state = CardStudioState::new();
        let _ = load_workspace_wcm(&mut restored_state, &test_wcm_path).expect("Should load .wcm");

        assert_eq!(restored_state.overlay_options.bg_preset, CardBackgroundPreset::BrushedGold);
        assert_eq!(restored_state.overlay_options.details.items.len(), state.overlay_options.details.items.len());
        assert_eq!(restored_state.overlay_options.details.items[item_idx].content, "WORKSPACE TEST");
        assert!(restored_state.overlay_options.details.items[item_idx].has_backdrop);

        let _ = std::fs::remove_file(test_wcm_path);
    }
}
