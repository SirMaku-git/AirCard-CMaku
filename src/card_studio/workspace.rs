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
    #[serde(default)]
    pub source_filename: Option<String>,
    pub has_custom_logo: bool,
    #[serde(default)]
    pub logo_filename: Option<String>,
    pub has_custom_chip: bool,
    #[serde(default)]
    pub chip_filename: Option<String>,
    pub has_custom_finish: bool,
    #[serde(default)]
    pub finish_filename: Option<String>,
    pub has_custom_font: bool,
    #[serde(default)]
    pub font_filename: Option<String>,
    pub custom_font_name: Option<String>,
    pub widgets_data: Vec<CustomWidgetData>,
    #[serde(default)]
    pub widget_filenames: Vec<String>,
}

fn encode_rgba_to_png(img: &RgbaImage) -> Result<Vec<u8>> {
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png)?;
    Ok(buf.into_inner())
}

fn write_asset_entry(
    zip: &mut ZipWriter<File>,
    options: SimpleFileOptions,
    filename: &str,
    raw_data: Option<&[u8]>,
    fallback_image: Option<&RgbaImage>,
) -> Result<Option<String>> {
    if let Some(bytes) = raw_data {
        zip.start_file(filename, options)?;
        zip.write_all(bytes)?;
        return Ok(Some(filename.to_string()));
    }
    if let Some(img) = fallback_image {
        let png_name = if filename.ends_with(".png") {
            filename.to_string()
        } else {
            let base = Path::new(filename)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("asset");
            format!("{base}.png")
        };
        zip.start_file(&png_name, options)?;
        let png_bytes = encode_rgba_to_png(img)?;
        zip.write_all(&png_bytes)?;
        return Ok(Some(png_name));
    }
    Ok(None)
}

pub fn save_workspace_wcm(
    state: &CardStudioState,
    source_image: Option<&DynamicImage>,
    source_raw: Option<&(Vec<u8>, String)>,
    target_path: &Path,
) -> Result<()> {
    let file = File::create(target_path)
        .with_context(|| format!("Cannot create workspace file: {}", target_path.display()))?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let mut manifest = WorkspaceManifest {
        format: "AirCard-CMaku-Workspace".to_string(),
        version: "1.1".to_string(),
        name: target_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("card_workspace")
            .to_string(),
        overlay_options: state.overlay_options.clone(),
        has_source_image: source_image.is_some() || source_raw.is_some(),
        source_filename: None,
        has_custom_logo: state.custom_logo_image.is_some() || state.custom_logo_raw.is_some(),
        logo_filename: None,
        has_custom_chip: state.custom_chip_image.is_some() || state.custom_chip_raw.is_some(),
        chip_filename: None,
        has_custom_finish: state.custom_finish_image.is_some() || state.custom_finish_raw.is_some(),
        finish_filename: None,
        has_custom_font: state.custom_font_bytes.is_some(),
        font_filename: None,
        custom_font_name: state.custom_font_name.clone(),
        widgets_data: state.custom_widgets.iter().map(|w| w.data.clone()).collect(),
        widget_filenames: Vec::new(),
    };

    // 1. Write source background image (preserve raw format like SVG/JPG/WebP to save space and keep 100% quality)
    if let Some((raw_bytes, ext)) = source_raw {
        let name = format!("source_image.{ext}");
        manifest.source_filename = write_asset_entry(&mut zip, options, &name, Some(raw_bytes), None)?;
    } else if let Some(src_img) = source_image {
        let rgba = src_img.to_rgba8();
        manifest.source_filename = write_asset_entry(&mut zip, options, "source_image.png", None, Some(&rgba))?;
    }

    // 2. Write custom logo (preserves raw SVG/PNG)
    if let Some((raw_bytes, ext)) = &state.custom_logo_raw {
        let name = format!("custom_logo.{ext}");
        manifest.logo_filename = write_asset_entry(&mut zip, options, &name, Some(raw_bytes), None)?;
    } else if let Some(ref logo) = state.custom_logo_image {
        manifest.logo_filename = write_asset_entry(&mut zip, options, "custom_logo.png", None, Some(logo))?;
    }

    // 3. Write custom chip (preserves raw SVG/PNG)
    if let Some((raw_bytes, ext)) = &state.custom_chip_raw {
        let name = format!("custom_chip.{ext}");
        manifest.chip_filename = write_asset_entry(&mut zip, options, &name, Some(raw_bytes), None)?;
    } else if let Some(ref chip) = state.custom_chip_image {
        manifest.chip_filename = write_asset_entry(&mut zip, options, "custom_chip.png", None, Some(chip))?;
    }

    // 4. Write custom finish texture
    if let Some((raw_bytes, ext)) = &state.custom_finish_raw {
        let name = format!("custom_finish.{ext}");
        manifest.finish_filename = write_asset_entry(&mut zip, options, &name, Some(raw_bytes), None)?;
    } else if let Some(ref finish) = state.custom_finish_image {
        manifest.finish_filename = write_asset_entry(&mut zip, options, "custom_finish.png", None, Some(finish))?;
    }

    // 5. Write custom font
    if let Some(ref font_bytes) = state.custom_font_bytes {
        let font_ext = state
            .custom_font_path
            .as_ref()
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .unwrap_or("ttf");
        let name = format!("custom_font.{font_ext}");
        zip.start_file(&name, options)?;
        zip.write_all(font_bytes)?;
        manifest.font_filename = Some(name);
    }

    // 6. Write custom widgets (preserves raw SVG/PNG/JPG/WebP per widget)
    for (i, widget) in state.custom_widgets.iter().enumerate() {
        if let (Some(raw), Some(ext)) = (&widget.raw_bytes, &widget.original_ext) {
            let name = format!("widget_{i}.{ext}");
            if let Some(written_name) = write_asset_entry(&mut zip, options, &name, Some(raw), None)? {
                manifest.widget_filenames.push(written_name);
            }
        } else {
            let name = format!("widget_{i}.png");
            if let Some(written_name) = write_asset_entry(&mut zip, options, &name, None, Some(&widget.image))? {
                manifest.widget_filenames.push(written_name);
            }
        }
    }

    // 7. Write manifest.json with full asset paths
    zip.start_file("manifest.json", options)?;
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    zip.write_all(&manifest_bytes)?;

    zip.finish()?;
    Ok(())
}

fn find_and_read_entry(
    zip: &mut ZipArchive<File>,
    preferred_name: Option<&str>,
    fallback_prefix: &str,
    exts: &[&str],
) -> Option<(Vec<u8>, String)> {
    if let Some(name) = preferred_name {
        if let Ok(mut f) = zip.by_name(name) {
            let mut buf = Vec::new();
            if f.read_to_end(&mut buf).is_ok() {
                let ext = Path::new(name)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("png")
                    .to_lowercase();
                return Some((buf, ext));
            }
        }
    }
    for ext in exts {
        let name = format!("{fallback_prefix}.{ext}");
        if let Ok(mut f) = zip.by_name(&name) {
            let mut buf = Vec::new();
            if f.read_to_end(&mut buf).is_ok() {
                return Some((buf, (*ext).to_string()));
            }
        }
    }
    None
}

pub fn load_workspace_wcm(
    state: &mut CardStudioState,
    source_path: &Path,
) -> Result<(Option<DynamicImage>, Option<(Vec<u8>, String)>)> {
    let file = File::open(source_path)
        .with_context(|| format!("Cannot open workspace file: {}", source_path.display()))?;
    let mut zip = ZipArchive::new(file)?;

    // 1. Read manifest.json
    let manifest: WorkspaceManifest = {
        let mut manifest_file = zip
            .by_name("manifest.json")
            .with_context(|| "Invalid .wcm workspace: missing manifest.json")?;
        let mut content = Vec::new();
        manifest_file.read_to_end(&mut content)?;
        serde_json::from_slice(&content)?
    };

    // 2. Restore overlay options
    state.overlay_options = manifest.overlay_options;
    state.overlay_options.details.ensure_items();

    // 3. Restore source background image (preserving raw bytes)
    let mut loaded_source_image: Option<DynamicImage> = None;
    let mut loaded_source_raw: Option<(Vec<u8>, String)> = None;
    if manifest.has_source_image {
        let supported_exts = ["svg", "png", "jpg", "jpeg", "webp", "bmp", "gif", "ico", "tiff"];
        if let Some((bytes, ext)) = find_and_read_entry(
            &mut zip,
            manifest.source_filename.as_deref(),
            "source_image",
            &supported_exts,
        ) {
            if let Ok(rgba) = load_image_any_format(&bytes, Some(&format!("source.{ext}"))) {
                loaded_source_image = Some(DynamicImage::ImageRgba8(rgba));
                loaded_source_raw = Some((bytes, ext));
            }
        }
    }

    // 4. Restore custom logo (preserving raw bytes)
    state.custom_logo_image = None;
    state.custom_logo_raw = None;
    if manifest.has_custom_logo {
        let supported_exts = ["svg", "png", "jpg", "jpeg", "webp", "bmp", "ico"];
        if let Some((bytes, ext)) = find_and_read_entry(
            &mut zip,
            manifest.logo_filename.as_deref(),
            "custom_logo",
            &supported_exts,
        ) {
            if let Ok(rgba) = load_image_any_format(&bytes, Some(&format!("logo.{ext}"))) {
                state.custom_logo_image = Some(rgba);
                state.custom_logo_raw = Some((bytes, ext));
            }
        }
    }

    // 5. Restore custom chip (preserving raw bytes)
    state.custom_chip_image = None;
    state.custom_chip_raw = None;
    if manifest.has_custom_chip {
        let supported_exts = ["svg", "png", "webp", "ico"];
        if let Some((bytes, ext)) = find_and_read_entry(
            &mut zip,
            manifest.chip_filename.as_deref(),
            "custom_chip",
            &supported_exts,
        ) {
            if let Ok(rgba) = load_image_any_format(&bytes, Some(&format!("chip.{ext}"))) {
                state.custom_chip_image = Some(rgba);
                state.custom_chip_raw = Some((bytes, ext));
            }
        }
    }

    // 6. Restore custom finish texture (preserving raw bytes)
    state.custom_finish_image = None;
    state.custom_finish_raw = None;
    if manifest.has_custom_finish {
        let supported_exts = ["svg", "png", "jpg", "jpeg", "webp", "bmp"];
        if let Some((bytes, ext)) = find_and_read_entry(
            &mut zip,
            manifest.finish_filename.as_deref(),
            "custom_finish",
            &supported_exts,
        ) {
            if let Ok(rgba) = load_image_any_format(&bytes, Some(&format!("finish.{ext}"))) {
                state.custom_finish_image = Some(rgba);
                state.custom_finish_raw = Some((bytes, ext));
            }
        }
    }

    // 7. Restore custom font
    state.custom_font_bytes = None;
    state.custom_font_name = manifest.custom_font_name.clone();
    if manifest.has_custom_font {
        if let Some((bytes, _)) = find_and_read_entry(
            &mut zip,
            manifest.font_filename.as_deref(),
            "custom_font",
            &["ttf", "otf"],
        ) {
            state.custom_font_bytes = Some(bytes);
        }
    }

    // 8. Restore custom widgets (preserving raw bytes per widget)
    state.custom_widgets.clear();
    let supported_exts = ["svg", "png", "jpg", "jpeg", "webp", "bmp", "gif", "ico", "tiff"];
    for (i, widget_data) in manifest.widgets_data.into_iter().enumerate() {
        let preferred = manifest.widget_filenames.get(i).map(|s| s.as_str());
        if let Some((bytes, ext)) = find_and_read_entry(
            &mut zip,
            preferred,
            &format!("widget_{i}"),
            &supported_exts,
        ) {
            if let Ok(rgba) = load_image_any_format(&bytes, Some(&format!("widget_{i}.{ext}"))) {
                state.custom_widgets.push(CustomWidget {
                    data: widget_data,
                    image: rgba,
                    path: None,
                    raw_bytes: Some(bytes),
                    original_ext: Some(ext),
                });
            }
        }
    }

    state.selected_text_index = 0;
    Ok((loaded_source_image, loaded_source_raw))
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

        save_workspace_wcm(&state, None, None, &test_wcm_path).expect("Should save .wcm");

        let mut restored_state = CardStudioState::new();
        let _ = load_workspace_wcm(&mut restored_state, &test_wcm_path).expect("Should load .wcm");

        assert_eq!(restored_state.overlay_options.bg_preset, CardBackgroundPreset::BrushedGold);
        assert_eq!(restored_state.overlay_options.details.items.len(), state.overlay_options.details.items.len());
        assert_eq!(restored_state.overlay_options.details.items[item_idx].content, "WORKSPACE TEST");
        assert!(restored_state.overlay_options.details.items[item_idx].has_backdrop);

        let _ = std::fs::remove_file(test_wcm_path);
    }

    #[test]
    fn test_workspace_raw_svg_asset_preservation() {
        let state = CardStudioState::new();
        let svg_data = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"200\" height=\"100\"><rect width=\"200\" height=\"100\" fill=\"#0000ff\" /></svg>".to_vec();
        let source_raw = (svg_data.clone(), "svg".to_string());

        let temp_dir = std::env::temp_dir();
        let test_wcm_path = temp_dir.join("test_aircard_raw_svg.wcm");

        save_workspace_wcm(&state, None, Some(&source_raw), &test_wcm_path).expect("Should save .wcm with raw SVG");

        // Verify that the file size is very small (around a few hundred bytes, NOT megabytes)
        let metadata = std::fs::metadata(&test_wcm_path).expect("File must exist");
        assert!(metadata.len() < 2048, "Raw SVG .wcm must be tiny (< 2KB), was {} bytes", metadata.len());

        let mut restored_state = CardStudioState::new();
        let (loaded_img, loaded_raw) = load_workspace_wcm(&mut restored_state, &test_wcm_path).expect("Should load .wcm");

        assert!(loaded_img.is_some(), "Should render SVG into image");
        let (raw_bytes, ext) = loaded_raw.expect("Should restore raw bytes and ext");
        assert_eq!(ext, "svg");
        assert_eq!(raw_bytes, svg_data);

        let _ = std::fs::remove_file(test_wcm_path);
    }
}
