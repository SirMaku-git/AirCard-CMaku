use std::io::Cursor;
use std::path::Path;

use anyhow::{Context, Result};
use eframe::egui;
use image::{DynamicImage, GenericImageView, ImageFormat, RgbaImage};

#[allow(unused_imports)]
pub use crate::card_studio::types::*;
#[allow(unused_imports)]
pub use crate::card_studio::widget::{CustomWidget, CustomWidgetData};
#[allow(unused_imports)]
pub use crate::card_studio::render::{
    render_transformed_custom_image, apply_card_rounded_corners, generate_card_preset_canvas,
};

#[derive(Clone)]
pub struct PreparedSkin {
    pub png: Vec<u8>,
    pub pdf: Vec<u8>,
    #[allow(dead_code)]
    pub preview: egui::ColorImage,
    pub source_width: u32,
    pub source_height: u32,
    pub rgba: RgbaImage,
}

#[allow(dead_code)]
impl PreparedSkin {
    pub fn from_path(path: &Path) -> Result<Self> {
        Self::from_path_with_options(path, &CardOverlayOptions::default())
    }

    pub fn from_path_with_options(path: &Path, options: &CardOverlayOptions) -> Result<Self> {
        let image =
            image::open(path).with_context(|| format!("Could not decode {}", path.display()))?;
        Self::from_image_with_options(image, options)
    }

    pub fn from_image(image: DynamicImage) -> Result<Self> {
        Self::from_image_with_options(image, &CardOverlayOptions::default())
    }

    pub fn from_image_with_options(image: DynamicImage, options: &CardOverlayOptions) -> Result<Self> {
        Self::from_preset_or_image(Some(image), options, None)
    }

    pub fn from_preset_or_image(
        maybe_image: Option<DynamicImage>,
        options: &CardOverlayOptions,
        custom_logo: Option<&RgbaImage>,
    ) -> Result<Self> {
        Self::from_preset_or_image_with_assets(maybe_image, options, custom_logo, None, None, &[])
    }

    pub fn render_preview_canvas(
        maybe_image: Option<&DynamicImage>,
        options: &CardOverlayOptions,
        custom_logo: Option<&RgbaImage>,
        custom_chip: Option<&RgbaImage>,
        custom_finish: Option<&RgbaImage>,
        custom_widgets: &[(&CustomWidgetData, &RgbaImage)],
    ) -> Result<(RgbaImage, egui::ColorImage)> {
        crate::card_studio::render::render_preview_canvas(
            maybe_image,
            options,
            custom_logo,
            custom_chip,
            custom_finish,
            None,
            custom_widgets,
        )
    }

    pub fn render_preview_canvas_with_font(
        maybe_image: Option<&DynamicImage>,
        options: &CardOverlayOptions,
        custom_logo: Option<&RgbaImage>,
        custom_chip: Option<&RgbaImage>,
        custom_finish: Option<&RgbaImage>,
        custom_font: Option<&[u8]>,
        custom_widgets: &[(&CustomWidgetData, &RgbaImage)],
    ) -> Result<(RgbaImage, egui::ColorImage)> {
        crate::card_studio::render::render_preview_canvas(
            maybe_image,
            options,
            custom_logo,
            custom_chip,
            custom_finish,
            custom_font,
            custom_widgets,
        )
    }

    pub fn encode_skin_png_and_pdf(rgba: &RgbaImage) -> Result<(Vec<u8>, Vec<u8>)> {
        let mut png = Vec::new();
        DynamicImage::ImageRgba8(rgba.clone())
            .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
            .context("Could not encode prepared PNG")?;

        let pdf = png_to_pdf(&png).context("Could not generate card PDF artwork")?;
        Ok((png, pdf))
    }

    pub fn from_preset_or_image_with_assets(
        maybe_image: Option<DynamicImage>,
        options: &CardOverlayOptions,
        custom_logo: Option<&RgbaImage>,
        custom_chip: Option<&RgbaImage>,
        custom_finish: Option<&RgbaImage>,
        custom_widgets: &[(&CustomWidgetData, &RgbaImage)],
    ) -> Result<Self> {
        let (source_width, source_height) = if let Some(ref img) = maybe_image {
            img.dimensions()
        } else {
            (CARD_WIDTH, CARD_HEIGHT)
        };

        let (rgba, preview) = Self::render_preview_canvas(
            maybe_image.as_ref(),
            options,
            custom_logo,
            custom_chip,
            custom_finish,
            custom_widgets,
        )?;

        let (png, pdf) = Self::encode_skin_png_and_pdf(&rgba)?;

        Ok(Self {
            png,
            pdf,
            preview,
            source_width,
            source_height,
            rgba,
        })
    }
}

// ---------------------------------------------------------------------------
// PDF Generation & Cropping
// ---------------------------------------------------------------------------
pub fn png_to_pdf(png_bytes: &[u8]) -> Result<Vec<u8>> {
    let img = image::load_from_memory(png_bytes)
        .context("Failed to decode image for PDF conversion")?;
    let rgba = img.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();

    let mut rgb_bytes = Vec::with_capacity((width * height * 3) as usize);
    let mut alpha_bytes = Vec::with_capacity((width * height) as usize);
    let mut has_transparency = false;

    for p in rgba.pixels() {
        rgb_bytes.push(p[0]);
        rgb_bytes.push(p[1]);
        rgb_bytes.push(p[2]);
        alpha_bytes.push(p[3]);
        if p[3] < 255 {
            has_transparency = true;
        }
    }

    let compressed_stream = miniz_oxide::deflate::compress_to_vec_zlib(&rgb_bytes, 6);
    let compressed_alpha = if has_transparency {
        Some(miniz_oxide::deflate::compress_to_vec_zlib(&alpha_bytes, 6))
    } else {
        None
    };

    let mut pdf = Vec::new();
    pdf.extend_from_slice(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n");

    let mut offsets = Vec::new();

    // 1 0 obj: Catalog
    offsets.push(pdf.len());
    pdf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // 2 0 obj: Pages
    offsets.push(pdf.len());
    pdf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    // 3 0 obj: Page
    offsets.push(pdf.len());
    let page_obj = format!(
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Contents 4 0 R /Resources << /XObject << /Im0 5 0 R >> >> >>\nendobj\n",
        width, height
    );
    pdf.extend_from_slice(page_obj.as_bytes());

    // 4 0 obj: Contents stream
    offsets.push(pdf.len());
    let content_stream = format!("q\n{} 0 0 {} 0 0 cm\n/Im0 Do\nQ\n", width, height);
    let contents_obj = format!(
        "4 0 obj\n<< /Length {} >>\nstream\n{}endstream\nendobj\n",
        content_stream.len(),
        content_stream
    );
    pdf.extend_from_slice(contents_obj.as_bytes());

    // 5 0 obj: Image XObject
    offsets.push(pdf.len());
    let smask_ref = if has_transparency { " /SMask 6 0 R" } else { "" };
    let image_header = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode{} /Length {} >>\nstream\n",
        width, height, smask_ref, compressed_stream.len()
    );
    pdf.extend_from_slice(image_header.as_bytes());
    pdf.extend_from_slice(&compressed_stream);
    pdf.extend_from_slice(b"\nendstream\nendobj\n");

    // 6 0 obj: SMask Image XObject (if transparency present)
    if let Some(ref alpha_stream) = compressed_alpha {
        offsets.push(pdf.len());
        let smask_header = format!(
            "6 0 obj\n<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /FlateDecode /Length {} >>\nstream\n",
            width, height, alpha_stream.len()
        );
        pdf.extend_from_slice(smask_header.as_bytes());
        pdf.extend_from_slice(alpha_stream);
        pdf.extend_from_slice(b"\nendstream\nendobj\n");
    }

    // xref table
    let xref_offset = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n", offsets.len() + 1).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");
    for &off in &offsets {
        pdf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }

    // trailer
    let trailer = format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        offsets.len() + 1,
        xref_offset
    );
    pdf.extend_from_slice(trailer.as_bytes());

    Ok(pdf)
}

#[allow(dead_code)]
fn center_crop_for_card(image: DynamicImage) -> DynamicImage {
    let (width, height) = image.dimensions();
    let card_ratio = CARD_WIDTH as f64 / CARD_HEIGHT as f64;
    let source_ratio = width as f64 / height as f64;

    if source_ratio > card_ratio {
        let crop_width = (height as f64 * card_ratio).round() as u32;
        let x = (width - crop_width) / 2;
        image.crop_imm(x, 0, crop_width, height)
    } else {
        let crop_height = (width as f64 / card_ratio).round() as u32;
        let y = (height - crop_height) / 2;
        image.crop_imm(0, y, width, crop_height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_png_to_pdf_conversion() {
        let dummy = DynamicImage::new_rgb8(10, 10);
        let mut png = Vec::new();
        dummy.write_to(&mut Cursor::new(&mut png), ImageFormat::Png).unwrap();

        let pdf = png_to_pdf(&png).expect("png_to_pdf failed");
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.ends_with(b"%%EOF\n"));
        let pdf_str = String::from_utf8_lossy(&pdf);
        assert!(pdf_str.contains("/FlateDecode"));
        assert!(pdf_str.contains("/MediaBox [0 0 10 10]"));
        assert!(pdf_str.contains("xref"));
    }

    #[test]
    fn test_overlay_rendering() {
        let dummy = DynamicImage::new_rgb8(800, 600);

        for net in [
            PaymentNetwork::None,
            PaymentNetwork::Visa,
            PaymentNetwork::Mastercard,
            PaymentNetwork::Napas,
            PaymentNetwork::Jcb,
            PaymentNetwork::Custom,
        ] {
            let options = CardOverlayOptions {
                bg_preset: CardBackgroundPreset::CustomImage,
                network: net,
                logo_style: LogoBadgeStyle::FrostedGlass,
                logo_color: LogoColorTheme::LuxuryGold,
                show_chip: true,
                show_contactless: true,
                finish: CardFinish::MetallicSheen,
                details: CardDetails {
                    show_details: true,
                    card_number: "9704 0334 1234 1234".to_string(),
                    card_holder: "NGUYEN VAN A".to_string(),
                    card_expiry: "09/29".to_string(),
                    card_type_or_bank: "Credit Card".to_string(),
                    emboss_style: EmbossStyle::EmbossedSilver,
                    backdrop: TextBackdropStyle::FrostedGlassStrip,
                    vertical_offset: 0,
                    ..CardDetails::default()
                },
                transform: ImageTransform::default(),
                ..CardOverlayOptions::default()
            };
            let custom_logo = DynamicImage::new_rgba8(100, 100).to_rgba8();
            let skin = PreparedSkin::from_preset_or_image(Some(dummy.clone()), &options, Some(&custom_logo))
                .unwrap_or_else(|e| panic!("rendering failed for {:?}: {}", net, e));
            assert_eq!(skin.preview.size, [CARD_WIDTH as usize, CARD_HEIGHT as usize]);
            assert!(!skin.png.is_empty());
            assert!(!skin.pdf.is_empty());
        }
    }

    #[test]
    fn test_preset_generation_without_image() {
        for preset in [
            CardBackgroundPreset::MatteBlack,
            CardBackgroundPreset::OceanNavy,
            CardBackgroundPreset::BrushedGold,
            CardBackgroundPreset::EmeraldLuxury,
            CardBackgroundPreset::TitaniumMinimal,
            CardBackgroundPreset::CrimsonVelvet,
            CardBackgroundPreset::DeepCyberViolet,
        ] {
            let options = CardOverlayOptions {
                bg_preset: preset,
                network: PaymentNetwork::Visa,
                logo_style: LogoBadgeStyle::ThinOutline,
                logo_color: LogoColorTheme::Original,
                show_chip: true,
                show_contactless: true,
                finish: CardFinish::Standard,
                details: CardDetails {
                    show_details: true,
                    card_number: "1234 5678 9012 3456".to_string(),
                    card_holder: "NAME SURNAME".to_string(),
                    card_expiry: "01/80".to_string(),
                    card_type_or_bank: "Bank Name".to_string(),
                    emboss_style: EmbossStyle::EmbossedGold,
                    backdrop: TextBackdropStyle::None,
                    vertical_offset: 0,
                    ..CardDetails::default()
                },
                transform: ImageTransform::default(),
                ..CardOverlayOptions::default()
            };
            let skin = PreparedSkin::from_preset_or_image(None, &options, None)
                .unwrap_or_else(|e| panic!("preset failed for {:?}: {}", preset, e));
            assert_eq!(skin.preview.size, [CARD_WIDTH as usize, CARD_HEIGHT as usize]);
            assert!(!skin.png.is_empty());
            assert!(!skin.pdf.is_empty());
        }
    }

    #[test]
    fn test_transformed_custom_image() {
        let dummy = DynamicImage::new_rgba8(800, 600);
        let transform = ImageTransform {
            zoom: 1.5,
            pan_x: 50.0,
            pan_y: -30.0,
            rotation: 15.0,
        };
        let res = render_transformed_custom_image(&dummy, transform, &LayerAdjustments::default());
        assert_eq!(res.width(), CARD_WIDTH);
        assert_eq!(res.height(), CARD_HEIGHT);
    }

    #[test]
    fn test_render_preview_canvas_and_shockwave() {
        let mut options = CardOverlayOptions::default();
        options.show_contactless = true;
        options.wave_x = 400.0;
        options.wave_y = 350.0;
        options.wave_scale = 1.2;
        options.wave_adj.rotation = 45.0;
        options.wave_adj.tint_color = [255, 0, 128];
        options.wave_adj.tint_amount = 0.8;

        let (rgba, preview) = PreparedSkin::render_preview_canvas(
            None,
            &options,
            None,
            None,
            None,
            &[],
        ).expect("render_preview_canvas failed");

        assert_eq!(rgba.width(), CARD_WIDTH);
        assert_eq!(rgba.height(), CARD_HEIGHT);
        assert_eq!(preview.size, [CARD_WIDTH as usize, CARD_HEIGHT as usize]);

        let (png, pdf) = PreparedSkin::encode_skin_png_and_pdf(&rgba).expect("encode failed");
        assert!(!png.is_empty());
        assert!(!pdf.is_empty());
    }

    #[test]
    fn test_custom_finish_and_chip_rendering() {
        let dummy = DynamicImage::new_rgba8(800, 600);
        let custom_chip = DynamicImage::new_rgba8(100, 80).to_rgba8();
        let custom_finish = DynamicImage::new_rgba8(200, 200).to_rgba8();

        let options = CardOverlayOptions {
            bg_preset: CardBackgroundPreset::CustomImage,
            network: PaymentNetwork::Visa,
            logo_style: LogoBadgeStyle::FrostedGlass,
            logo_color: LogoColorTheme::LuxuryGold,
            show_chip: true,
            show_contactless: false,
            finish: CardFinish::CustomTexture,
            finish_x: 10.0,
            finish_y: 20.0,
            finish_scale: 1.2,
            finish_opacity: 0.8,
            chip_x: 160.0,
            chip_y: 350.0,
            chip_scale: 1.1,
            logo_x: 1200.0,
            logo_y: 800.0,
            logo_scale: 0.9,
            ..CardOverlayOptions::default()
        };

        let skin = PreparedSkin::from_preset_or_image_with_assets(
            Some(dummy),
            &options,
            None,
            Some(&custom_chip),
            Some(&custom_finish),
            &[],
        ).expect("Custom finish and chip rendering failed");

        assert_eq!(skin.preview.size, [CARD_WIDTH as usize, CARD_HEIGHT as usize]);
        assert!(!skin.png.is_empty());
        assert!(!skin.pdf.is_empty());
    }

    #[test]
    fn test_custom_widget_rendering() {
        let dummy = DynamicImage::new_rgba8(800, 600);
        let widget_img = DynamicImage::new_rgba8(120, 120).to_rgba8();
        let widget_data = CustomWidgetData {
            id: 1,
            name: "Test Sticker".to_string(),
            x: 500.0,
            y: 400.0,
            scale: 1.5,
            base_w: 120.0,
            base_h: 120.0,
            adjustments: LayerAdjustments {
                rotation: 30.0,
                opacity: 0.9,
                tint_color: [0, 200, 255],
                tint_amount: 0.5,
                hue_shift: 15.0,
                saturation: 1.2,
            },
            visible: true,
            has_shadow: true,
        };

        let skin = PreparedSkin::from_preset_or_image_with_assets(
            Some(dummy),
            &CardOverlayOptions::default(),
            None,
            None,
            None,
            &[(&widget_data, &widget_img)],
        ).expect("Custom widget rendering failed");

        assert_eq!(skin.preview.size, [CARD_WIDTH as usize, CARD_HEIGHT as usize]);
        assert!(!skin.png.is_empty());
        assert!(!skin.pdf.is_empty());
    }

    #[test]
    fn test_custom_font_rendering() {
        use crate::card_studio::types::{CardDetails, CardFontPreset};

        let mut options = CardOverlayOptions::default();
        options.details = CardDetails {
            show_details: true,
            card_number: "4000 1234 5678 9010".to_string(),
            card_holder: "SATOSHI NAKAMOTO".to_string(),
            card_expiry: "12/30".to_string(),
            card_type_or_bank: "Crypto Bank".to_string(),
            number_font: CardFontPreset::Custom,
            text_font: CardFontPreset::Custom,
            ..CardDetails::default()
        };

        let fake_custom_font = include_bytes!("assets/card_sans.ttf");

        let (rgba, preview) = PreparedSkin::render_preview_canvas_with_font(
            None,
            &options,
            None,
            None,
            None,
            Some(fake_custom_font),
            &[],
        ).expect("Custom font rendering failed");

        assert_eq!(rgba.width(), CARD_WIDTH);
        assert_eq!(rgba.height(), CARD_HEIGHT);
        assert_eq!(preview.size, [CARD_WIDTH as usize, CARD_HEIGHT as usize]);
    }
}
