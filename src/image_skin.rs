use std::io::Cursor;
use std::path::Path;

use anyhow::{Context, Result};
use eframe::egui;
use image::{DynamicImage, GenericImageView, ImageFormat, Rgba, RgbaImage};
use ab_glyph::{Font, FontRef, PxScale, ScaleFont, point};

pub const CARD_WIDTH: u32 = 1_536;
pub const CARD_HEIGHT: u32 = 969;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum CardBackgroundPreset {
    #[default]
    CustomImage,
    MatteBlack,
    OceanNavy,
    BrushedGold,
    EmeraldLuxury,
    TitaniumMinimal,
    CrimsonVelvet,
    DeepCyberViolet,
}

impl CardBackgroundPreset {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::CustomImage => "Custom Image",
            Self::MatteBlack => "Matte Obsidian Black",
            Self::OceanNavy => "Ocean Bank Navy",
            Self::BrushedGold => "Brushed Gold",
            Self::EmeraldLuxury => "Emerald Luxury",
            Self::TitaniumMinimal => "Titanium Minimal",
            Self::CrimsonVelvet => "Crimson Velvet",
            Self::DeepCyberViolet => "Cyber Violet",
        }
    }

    pub fn display_name_lang(&self, is_vi: bool) -> &'static str {
        if !is_vi { return self.display_name(); }
        match self {
            Self::CustomImage => "Ảnh tùy chỉnh",
            Self::MatteBlack => "Đen nhám Obsidian",
            Self::OceanNavy => "Xanh Navy (Ocean Navy)",
            Self::BrushedGold => "Vàng kim loại (Brushed Gold)",
            Self::EmeraldLuxury => "Xanh ngọc lục bảo (Emerald)",
            Self::TitaniumMinimal => "Titanium tối giản",
            Self::CrimsonVelvet => "Đỏ nhung (Crimson Velvet)",
            Self::DeepCyberViolet => "Tím Cyber Violet",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum LogoBadgeStyle {
    #[default]
    Transparent,
    ThinOutline,
    FrostedGlass,
    SolidDark,
    SolidLight,
    SubtleGlow,
}

impl LogoBadgeStyle {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Transparent => "Transparent (No Frame)",
            Self::ThinOutline => "Thin Outline",
            Self::FrostedGlass => "Frosted Glass (Glassmorphism)",
            Self::SolidDark => "Solid Dark Frame",
            Self::SolidLight => "Solid Light Frame",
            Self::SubtleGlow => "Subtle Glow",
        }
    }

    pub fn display_name_lang(&self, is_vi: bool) -> &'static str {
        if !is_vi { return self.display_name(); }
        match self {
            Self::Transparent => "Trong suốt (Không khung)",
            Self::ThinOutline => "Viền mỏng (Thin Outline)",
            Self::FrostedGlass => "Kính mờ (Frosted Glass)",
            Self::SolidDark => "Khung tối (Solid Dark)",
            Self::SolidLight => "Khung sáng (Solid Light)",
            Self::SubtleGlow => "Phát sáng nhẹ (Subtle Glow)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum LogoColorTheme {
    #[default]
    Original,
    MonochromeWhite,
    LuxuryGold,
    SilverPlatinum,
    StealthBlack,
}

impl LogoColorTheme {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Original => "Original Brand Colors",
            Self::MonochromeWhite => "Monochrome White",
            Self::LuxuryGold => "Luxury Gold",
            Self::SilverPlatinum => "Silver Platinum",
            Self::StealthBlack => "Stealth Black",
        }
    }

    pub fn display_name_lang(&self, is_vi: bool) -> &'static str {
        if !is_vi { return self.display_name(); }
        match self {
            Self::Original => "Màu thương hiệu gốc",
            Self::MonochromeWhite => "Trắng đơn sắc (Monochrome White)",
            Self::LuxuryGold => "Vàng kim loại (Luxury Gold)",
            Self::SilverPlatinum => "Bạch kim (Silver Platinum)",
            Self::StealthBlack => "Đen Stealth Black",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum EmbossStyle {
    #[default]
    EmbossedSilver,
    EmbossedGold,
    CrispWhite,
    StealthDark,
}

impl EmbossStyle {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::EmbossedSilver => "3D Embossed Silver",
            Self::EmbossedGold => "3D Embossed Gold",
            Self::CrispWhite => "Crisp White (Flat)",
            Self::StealthDark => "Stealth Dark (Flat)",
        }
    }

    pub fn display_name_lang(&self, is_vi: bool) -> &'static str {
        if !is_vi { return self.display_name(); }
        match self {
            Self::EmbossedSilver => "Bạc ánh kim 3D (Embossed Silver)",
            Self::EmbossedGold => "Vàng ánh kim 3D (Embossed Gold)",
            Self::CrispWhite => "Trắng phẳng (Crisp White)",
            Self::StealthDark => "Đen tối (Stealth Dark)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum TextBackdropStyle {
    #[default]
    None,
    FrostedGlassStrip,
    SubtleDarkGradient,
    FrostedPills,
}

impl TextBackdropStyle {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::None => "None (Ambient Shadow)",
            Self::FrostedGlassStrip => "Frosted Glass Strip",
            Self::SubtleDarkGradient => "Dark Gradient Scrim",
            Self::FrostedPills => "Frosted Glass Pills",
        }
    }

    pub fn display_name_lang(&self, is_vi: bool) -> &'static str {
        if !is_vi { return self.display_name(); }
        match self {
            Self::None => "Không (Bóng mờ tự nhiên)",
            Self::FrostedGlassStrip => "Thanh kính mờ (Frosted Glass Strip)",
            Self::SubtleDarkGradient => "Dải chuyển màu tối (Dark Gradient)",
            Self::FrostedPills => "Khung kính mờ (Frosted Pills)",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CardDetails {
    pub show_details: bool,
    pub card_number: String,
    pub card_holder: String,
    pub card_expiry: String,
    pub card_type_or_bank: String,
    pub emboss_style: EmbossStyle,
    pub backdrop: TextBackdropStyle,
    pub vertical_offset: i32,
}

impl Default for CardDetails {
    fn default() -> Self {
        Self {
            show_details: false,
            card_number: "9704 0334 1234 1234".to_string(),
            card_holder: "CARD HOLDER NAME".to_string(),
            card_expiry: "09/29".to_string(),
            card_type_or_bank: "Credit Card".to_string(),
            emboss_style: EmbossStyle::EmbossedSilver,
            backdrop: TextBackdropStyle::None,
            vertical_offset: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum PaymentNetwork {
    #[default]
    None,
    Visa,
    Mastercard,
    Napas,
    Jcb,
    Custom,
}

impl PaymentNetwork {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Visa => "Visa",
            Self::Mastercard => "Mastercard",
            Self::Napas => "Napas",
            Self::Jcb => "JCB",
            Self::Custom => "Custom Logo (Uploaded)",
        }
    }

    pub fn display_name_lang(&self, is_vi: bool) -> &'static str {
        if !is_vi { return self.display_name(); }
        match self {
            Self::None => "Không",
            Self::Visa => "Visa",
            Self::Mastercard => "Mastercard",
            Self::Napas => "Napas",
            Self::Jcb => "JCB",
            Self::Custom => "Logo riêng (Upload)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum CardFinish {
    #[default]
    Standard,
    MetallicSheen,
    CarbonWeave,
    CustomTexture,
}

impl CardFinish {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Standard => "Standard Matte",
            Self::MetallicSheen => "Metallic Specular Sheen",
            Self::CarbonWeave => "Carbon Fiber Weave",
            Self::CustomTexture => "Custom Texture / Foil (Upload)",
        }
    }

    pub fn display_name_lang(&self, is_vi: bool) -> &'static str {
        if !is_vi { return self.display_name(); }
        match self {
            Self::Standard => "Nhám tiêu chuẩn (Standard Matte)",
            Self::MetallicSheen => "Ánh kim loại (Metallic Sheen)",
            Self::CarbonWeave => "Vân sợi Carbon (Carbon Weave)",
            Self::CustomTexture => "Texture / Foil riêng (Upload)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ImageTransform {
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub rotation: f32,
}

impl Default for ImageTransform {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            rotation: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LayerAdjustments {
    pub rotation: f32,
    pub opacity: f32,
    pub tint_color: [u8; 3],
    pub tint_amount: f32,
    pub hue_shift: f32,
    pub saturation: f32,
}

impl Default for LayerAdjustments {
    fn default() -> Self {
        Self {
            rotation: 0.0,
            opacity: 1.0,
            tint_color: [255, 255, 255],
            tint_amount: 0.0,
            hue_shift: 0.0,
            saturation: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CardOverlayOptions {
    pub bg_preset: CardBackgroundPreset,
    pub network: PaymentNetwork,
    pub logo_style: LogoBadgeStyle,
    pub logo_color: LogoColorTheme,
    pub show_chip: bool,
    pub show_contactless: bool,
    pub finish: CardFinish,
    pub details: CardDetails,
    pub transform: ImageTransform,
    // Layer Adjustments (Color Tint, Opacity, Hue, Saturation, Rotation)
    pub bg_adj: LayerAdjustments,
    pub finish_adj: LayerAdjustments,
    pub chip_adj: LayerAdjustments,
    pub wave_adj: LayerAdjustments,
    pub logo_adj: LayerAdjustments,
    pub details_adj: LayerAdjustments,
    // Custom Finish Controls
    pub finish_x: f32,
    pub finish_y: f32,
    pub finish_scale: f32,
    pub finish_opacity: f32,
    // Custom Chip Controls
    pub chip_x: f32,
    pub chip_y: f32,
    pub chip_scale: f32,
    // Contactless Wave (Shockwave) Controls
    pub wave_x: f32,
    pub wave_y: f32,
    pub wave_scale: f32,
    // Custom Logo Controls
    pub logo_x: f32,
    pub logo_y: f32,
    pub logo_scale: f32,
}

impl Default for CardOverlayOptions {
    fn default() -> Self {
        Self {
            bg_preset: CardBackgroundPreset::CustomImage,
            network: PaymentNetwork::Visa,
            logo_style: LogoBadgeStyle::Transparent,
            logo_color: LogoColorTheme::Original,
            show_chip: false,
            show_contactless: false,
            finish: CardFinish::Standard,
            details: CardDetails::default(),
            transform: ImageTransform::default(),
            bg_adj: LayerAdjustments::default(),
            finish_adj: LayerAdjustments::default(),
            chip_adj: LayerAdjustments::default(),
            wave_adj: LayerAdjustments::default(),
            logo_adj: LayerAdjustments::default(),
            details_adj: LayerAdjustments::default(),
            finish_x: 0.0,
            finish_y: 0.0,
            finish_scale: 1.0,
            finish_opacity: 0.7,
            chip_x: 140.0,
            chip_y: 340.0,
            chip_scale: 1.0,
            wave_x: 375.0,
            wave_y: 375.0,
            wave_scale: 1.0,
            logo_x: 1215.0,
            logo_y: 812.0,
            logo_scale: 1.0,
        }
    }
}

#[derive(Clone)]
pub struct PreparedSkin {
    pub png: Vec<u8>,
    pub pdf: Vec<u8>,
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
        Self::from_preset_or_image_with_assets(maybe_image, options, custom_logo, None, None)
    }

    pub fn render_preview_canvas(
        maybe_image: Option<&DynamicImage>,
        options: &CardOverlayOptions,
        custom_logo: Option<&RgbaImage>,
        custom_chip: Option<&RgbaImage>,
        custom_finish: Option<&RgbaImage>,
    ) -> Result<(RgbaImage, egui::ColorImage)> {
        let mut rgba = generate_card_preset_canvas(
            options.bg_preset,
            maybe_image,
            options.transform,
            &options.bg_adj,
        );

        // 1. Apply surface finish effects (including Custom Texture if uploaded)
        apply_surface_finish(&mut rgba, options.finish, custom_finish, options);

        // 2. Draw EMV Gold / Custom Chip
        if options.show_chip {
            draw_emv_chip(
                &mut rgba,
                custom_chip,
                options.chip_x,
                options.chip_y,
                options.chip_scale,
                &options.chip_adj,
            );
        }

        // 3. Draw Contactless Waves (Shockwave) with position, scale, and rotation
        if options.show_contactless {
            draw_contactless_wave(
                &mut rgba,
                options.wave_x,
                options.wave_y,
                options.wave_scale,
                &options.wave_adj,
            );
        }

        // 4. Draw Payment Network Brand Badge with Frame, Color Theme, and custom position/scale/rotation
        if options.network != PaymentNetwork::None {
            draw_payment_network(
                &mut rgba,
                options.network,
                options.logo_style,
                options.logo_color,
                custom_logo,
                options.logo_x,
                options.logo_y,
                options.logo_scale,
                &options.logo_adj,
            );
        }

        // 5. Draw Card Details (Embossed numbers, cardholder, expiry, bank name)
        if options.details.show_details {
            draw_card_details(&mut rgba, &options.details, &options.details_adj);
        }

        // 6. Clip Card to authentic ISO 7810 rounded corners (Anti-aliased)
        apply_card_rounded_corners(&mut rgba, 58.0);

        let preview = egui::ColorImage::from_rgba_unmultiplied(
            [CARD_WIDTH as usize, CARD_HEIGHT as usize],
            rgba.as_raw(),
        );

        Ok((rgba, preview))
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
// 2D Drawing Utilities & Overlays
// ---------------------------------------------------------------------------

#[inline]
fn blend_pixel(img: &mut RgbaImage, x: i32, y: i32, r: u8, g: u8, b: u8, a: u8) {
    if x < 0 || y < 0 || x >= CARD_WIDTH as i32 || y >= CARD_HEIGHT as i32 || a == 0 {
        return;
    }
    let px = img.get_pixel_mut(x as u32, y as u32);
    if a == 255 {
        *px = Rgba([r, g, b, 255]);
    } else {
        let alpha = a as u32;
        let inv_alpha = 255 - alpha;
        let dst_r = px[0] as u32;
        let dst_g = px[1] as u32;
        let dst_b = px[2] as u32;
        let out_r = ((r as u32 * alpha + dst_r * inv_alpha) / 255) as u8;
        let out_g = ((g as u32 * alpha + dst_g * inv_alpha) / 255) as u8;
        let out_b = ((b as u32 * alpha + dst_b * inv_alpha) / 255) as u8;
        *px = Rgba([out_r, out_g, out_b, 255]);
    }
}

fn draw_rounded_rect(
    img: &mut RgbaImage,
    x0: i32,
    y0: i32,
    w: i32,
    h: i32,
    radius: f32,
    color: [u8; 4],
) {
    let r2 = radius * radius;
    let x1 = x0 + w;
    let y1 = y0 + h;

    for y in y0..y1 {
        for x in x0..x1 {
            let mut inside = true;
            // Check corners
            let cx = if ((x - x0) as f32) < radius {
                x0 as f32 + radius
            } else if ((x1 - 1 - x) as f32) < radius {
                x1 as f32 - 1.0 - radius
            } else {
                x as f32
            };

            let cy = if ((y - y0) as f32) < radius {
                y0 as f32 + radius
            } else if ((y1 - 1 - y) as f32) < radius {
                y1 as f32 - 1.0 - radius
            } else {
                y as f32
            };

            if cx != x as f32 && cy != y as f32 {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                if dx * dx + dy * dy > r2 {
                    inside = false;
                }
            }

            if inside {
                blend_pixel(img, x, y, color[0], color[1], color[2], color[3]);
            }
        }
    }
}

#[allow(dead_code)]
fn draw_filled_circle(img: &mut RgbaImage, cx: i32, cy: i32, radius: f32, color: [u8; 4]) {
    let r_ceil = radius.ceil() as i32;
    let r2 = radius * radius;
    for dy in -r_ceil..=r_ceil {
        for dx in -r_ceil..=r_ceil {
            let dist2 = (dx * dx + dy * dy) as f32;
            if dist2 <= r2 {
                let dist = dist2.sqrt();
                let alpha = if dist > radius - 1.0 {
                    let edge = (radius - dist).clamp(0.0, 1.0);
                    (color[3] as f32 * edge) as u8
                } else {
                    color[3]
                };
                blend_pixel(img, cx + dx, cy + dy, color[0], color[1], color[2], alpha);
            }
        }
    }
}

fn draw_rounded_rect_outline(
    img: &mut RgbaImage,
    x0: i32,
    y0: i32,
    w: i32,
    h: i32,
    radius: f32,
    thickness: f32,
    color: [u8; 4],
) {
    let r_out = radius;
    let r_in = (radius - thickness).max(0.0);
    let r_out2 = r_out * r_out;
    let r_in2 = r_in * r_in;
    let x1 = x0 + w;
    let y1 = y0 + h;

    for y in y0..y1 {
        for x in x0..x1 {
            let cx = if ((x - x0) as f32) < radius {
                x0 as f32 + radius
            } else if ((x1 - 1 - x) as f32) < radius {
                x1 as f32 - 1.0 - radius
            } else {
                x as f32
            };

            let cy = if ((y - y0) as f32) < radius {
                y0 as f32 + radius
            } else if ((y1 - 1 - y) as f32) < radius {
                y1 as f32 - 1.0 - radius
            } else {
                y as f32
            };

            let on_edge = if cx != x as f32 && cy != y as f32 {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let d2 = dx * dx + dy * dy;
                d2 <= r_out2 && d2 >= r_in2
            } else {
                let on_left = (x - x0) < thickness as i32;
                let on_right = (x1 - 1 - x) < thickness as i32;
                let on_top = (y - y0) < thickness as i32;
                let on_bottom = (y1 - 1 - y) < thickness as i32;
                on_left || on_right || on_top || on_bottom
            };

            if on_edge {
                blend_pixel(img, x, y, color[0], color[1], color[2], color[3]);
            }
        }
    }
}

pub fn render_transformed_custom_image(
    image: &DynamicImage,
    transform: ImageTransform,
    bg_adj: &LayerAdjustments,
) -> RgbaImage {
    let (src_w, src_h) = image.dimensions();
    if src_w == 0 || src_h == 0 {
        return RgbaImage::from_pixel(CARD_WIDTH, CARD_HEIGHT, Rgba([18, 18, 22, 255]));
    }
    let src_rgba = image.to_rgba8();

    let mut dest = RgbaImage::from_pixel(CARD_WIDTH, CARD_HEIGHT, Rgba([18, 18, 22, 255]));

    let zoom = transform.zoom.clamp(0.05, 50.0);
    let scale0 = (CARD_WIDTH as f32 / src_w as f32).max(CARD_HEIGHT as f32 / src_h as f32);
    let scale = scale0 * zoom;

    let inv_scale = 1.0 / scale;
    let src_raw = src_rgba.as_raw();
    let stride = src_w as usize * 4;
    let max_x_idx = src_w - 1;
    let max_y_idx = src_h - 1;

    let center_x = CARD_WIDTH as f32 * 0.5 + transform.pan_x;
    let center_y = CARD_HEIGHT as f32 * 0.5 + transform.pan_y;
    let src_cx = src_w as f32 * 0.5;
    let src_cy = src_h as f32 * 0.5;

    let total_rot = transform.rotation + bg_adj.rotation;
    let rad = (-total_rot).to_radians();
    let cos_r = rad.cos();
    let sin_r = rad.sin();

    for y in 0..CARD_HEIGHT {
        let dy = y as f32 + 0.5 - center_y;
        for x in 0..CARD_WIDTH {
            let dx = x as f32 + 0.5 - center_x;
            let rx = dx * cos_r - dy * sin_r;
            let ry = dx * sin_r + dy * cos_r;

            let sx = rx * inv_scale + src_cx;
            let sy = ry * inv_scale + src_cy;

            if sx < 0.0 || sx > max_x_idx as f32 || sy < 0.0 || sy > max_y_idx as f32 {
                continue;
            }

            let sx_clamped = sx.clamp(0.0, max_x_idx as f32);
            let sy_clamped = sy.clamp(0.0, max_y_idx as f32);
            let x0 = sx_clamped.floor() as u32;
            let x1 = (x0 + 1).min(max_x_idx);
            let y0 = sy_clamped.floor() as u32;
            let y1 = (y0 + 1).min(max_y_idx);

            let fx = sx_clamped - x0 as f32;
            let fy = sy_clamped - y0 as f32;
            let inv_fx = 1.0 - fx;
            let inv_fy = 1.0 - fy;

            let w00 = inv_fx * inv_fy;
            let w10 = fx * inv_fy;
            let w01 = inv_fx * fy;
            let w11 = fx * fy;

            let row0 = y0 as usize * stride;
            let row1 = y1 as usize * stride;

            let idx00 = row0 + x0 as usize * 4;
            let idx10 = row0 + x1 as usize * 4;
            let idx01 = row1 + x0 as usize * 4;
            let idx11 = row1 + x1 as usize * 4;

            let r = src_raw[idx00] as f32 * w00
                + src_raw[idx10] as f32 * w10
                + src_raw[idx01] as f32 * w01
                + src_raw[idx11] as f32 * w11;
            let g = src_raw[idx00 + 1] as f32 * w00
                + src_raw[idx10 + 1] as f32 * w10
                + src_raw[idx01 + 1] as f32 * w01
                + src_raw[idx11 + 1] as f32 * w11;
            let b = src_raw[idx00 + 2] as f32 * w00
                + src_raw[idx10 + 2] as f32 * w10
                + src_raw[idx01 + 2] as f32 * w01
                + src_raw[idx11 + 2] as f32 * w11;
            let a = src_raw[idx00 + 3] as f32 * w00
                + src_raw[idx10 + 3] as f32 * w10
                + src_raw[idx01 + 3] as f32 * w01
                + src_raw[idx11 + 3] as f32 * w11;

            let (out_r, out_g, out_b, out_a) = apply_adjustments_to_color(r, g, b, a, bg_adj, LogoColorTheme::Original);
            dest.put_pixel(x, y, Rgba([out_r, out_g, out_b, out_a]));
        }
    }

    dest
}

pub fn generate_card_preset_canvas(
    preset: CardBackgroundPreset,
    maybe_image: Option<&DynamicImage>,
    transform: ImageTransform,
    bg_adj: &LayerAdjustments,
) -> RgbaImage {
    let mut img = RgbaImage::new(CARD_WIDTH, CARD_HEIGHT);

    match preset {
        CardBackgroundPreset::CustomImage => {
            if let Some(custom) = maybe_image {
                return render_transformed_custom_image(custom, transform, bg_adj);
            } else {
                // Default dark minimalist card canvas if no image has been loaded yet
                for y in 0..CARD_HEIGHT {
                    let fy = y as f32 / CARD_HEIGHT as f32;
                    let v = (25.0 + fy * 15.0) as u8;
                    for x in 0..CARD_WIDTH {
                        img.put_pixel(x, y, Rgba([v, v, v + 4, 255]));
                    }
                }
            }
        }
        CardBackgroundPreset::MatteBlack => {
            // Obsidian Matte Black with diagonal specular sheen (Image 1 style)
            for y in 0..CARD_HEIGHT {
                let fy = y as f32 / CARD_HEIGHT as f32;
                for x in 0..CARD_WIDTH {
                    let fx = x as f32 / CARD_WIDTH as f32;
                    let base = 22.0 - fy * 10.0 + fx * 4.0;
                    let diag_dist = ((x as f32 * 0.7 + y as f32) - 800.0).abs();
                    let specular = if diag_dist < 400.0 {
                        ((400.0 - diag_dist) / 400.0).powi(2) * 22.0
                    } else {
                        0.0
                    };
                    let val = ((base + specular).clamp(10.0, 65.0)) as u8;
                    img.put_pixel(x, y, Rgba([val, val, (val as f32 * 1.05).min(255.0) as u8, 255]));
                }
            }
            draw_rounded_rect_outline(&mut img, 20, 20, (CARD_WIDTH - 40) as i32, (CARD_HEIGHT - 40) as i32, 28.0, 1.5, [255, 255, 255, 25]);
        }
        CardBackgroundPreset::OceanNavy => {
            // Deep bank navy gradient with soft radial glow (Napas Image 2 style)
            let center_x = CARD_WIDTH as f32 * 0.40;
            let center_y = CARD_HEIGHT as f32 * 0.35;
            for y in 0..CARD_HEIGHT {
                let dy = (y as f32 - center_y) / CARD_HEIGHT as f32;
                for x in 0..CARD_WIDTH {
                    let dx = (x as f32 - center_x) / CARD_WIDTH as f32;
                    let dist2 = dx * dx + dy * dy;
                    let glow = (1.0 - dist2.sqrt() * 1.3).clamp(0.0, 1.0);
                    let r = (4.0 + glow * 24.0) as u8;
                    let g = (32.0 + glow * 58.0) as u8;
                    let b = (95.0 + glow * 85.0) as u8;
                    img.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
            // Subtle global map watermark arcs
            for angle_step in 1..=4 {
                let cy = -200i32 + angle_step * 250;
                draw_rounded_rect_outline(&mut img, 100, cy, 1336, 600, 300.0, 1.0, [255, 255, 255, 12]);
            }
        }
        CardBackgroundPreset::BrushedGold => {
            // Warm Champagne Gold with horizontal brush striations
            for y in 0..CARD_HEIGHT {
                let fy = y as f32 / CARD_HEIGHT as f32;
                let base_r = 210.0 - fy * 45.0;
                let base_g = 168.0 - fy * 40.0;
                let base_b = 58.0 - fy * 18.0;
                for x in 0..CARD_WIDTH {
                    let noise = (((x * 17 + y * 73) % 29) as f32 - 14.0) * 0.9;
                    let r = (base_r + noise).clamp(0.0, 255.0) as u8;
                    let g = (base_g + noise).clamp(0.0, 255.0) as u8;
                    let b = (base_b + noise * 0.5).clamp(0.0, 255.0) as u8;
                    img.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
            draw_rounded_rect_outline(&mut img, 20, 20, (CARD_WIDTH - 40) as i32, (CARD_HEIGHT - 40) as i32, 28.0, 2.0, [255, 235, 160, 60]);
        }
        CardBackgroundPreset::EmeraldLuxury => {
            // Royal Emerald Green
            for y in 0..CARD_HEIGHT {
                let fy = y as f32 / CARD_HEIGHT as f32;
                for x in 0..CARD_WIDTH {
                    let fx = x as f32 / CARD_WIDTH as f32;
                    let d = ((fx - 0.5) * (fx - 0.5) + (fy - 0.5) * (fy - 0.5)).sqrt();
                    let factor = (1.0 - d * 0.9).clamp(0.0, 1.0);
                    let r = (5.0 + factor * 25.0) as u8;
                    let g = (45.0 + factor * 75.0) as u8;
                    let b = (35.0 + factor * 45.0) as u8;
                    img.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
            draw_rounded_rect_outline(&mut img, 20, 20, (CARD_WIDTH - 40) as i32, (CARD_HEIGHT - 40) as i32, 28.0, 1.5, [140, 255, 200, 45]);
        }
        CardBackgroundPreset::TitaniumMinimal => {
            // Satin Aerospace Titanium
            for y in 0..CARD_HEIGHT {
                let fy = y as f32 / CARD_HEIGHT as f32;
                let val = (75.0 + fy * 20.0) as u8;
                for x in 0..CARD_WIDTH {
                    let noise = (((x * 19 + y * 53) % 17) as f32 - 8.0) * 0.6;
                    let v = (val as f32 + noise).clamp(0.0, 255.0) as u8;
                    img.put_pixel(x, y, Rgba([v, (v as f32 * 1.02).min(255.0) as u8, (v as f32 * 1.05).min(255.0) as u8, 255]));
                }
            }
            draw_rounded_rect_outline(&mut img, 20, 20, (CARD_WIDTH - 40) as i32, (CARD_HEIGHT - 40) as i32, 28.0, 1.5, [255, 255, 255, 45]);
        }
        CardBackgroundPreset::CrimsonVelvet => {
            // Deep Velvet Ruby
            for y in 0..CARD_HEIGHT {
                let fy = y as f32 / CARD_HEIGHT as f32;
                for x in 0..CARD_WIDTH {
                    let fx = x as f32 / CARD_WIDTH as f32;
                    let r = (75.0 - fy * 30.0 + fx * 15.0).clamp(0.0, 255.0) as u8;
                    let g = (12.0 - fy * 5.0).clamp(0.0, 255.0) as u8;
                    let b = (24.0 - fy * 10.0).clamp(0.0, 255.0) as u8;
                    img.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
            draw_rounded_rect_outline(&mut img, 20, 20, (CARD_WIDTH - 40) as i32, (CARD_HEIGHT - 40) as i32, 28.0, 1.5, [255, 140, 160, 40]);
        }
        CardBackgroundPreset::DeepCyberViolet => {
            // Cyberpunk Dark Violet
            for y in 0..CARD_HEIGHT {
                let fy = y as f32 / CARD_HEIGHT as f32;
                for x in 0..CARD_WIDTH {
                    let fx = x as f32 / CARD_WIDTH as f32;
                    let r = (45.0 + fx * 35.0 - fy * 15.0).clamp(0.0, 255.0) as u8;
                    let g = (12.0 + fx * 10.0).clamp(0.0, 255.0) as u8;
                    let b = (75.0 + fy * 40.0).clamp(0.0, 255.0) as u8;
                    img.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
            draw_rounded_rect_outline(&mut img, 20, 20, (CARD_WIDTH - 40) as i32, (CARD_HEIGHT - 40) as i32, 28.0, 1.5, [200, 160, 255, 45]);
        }
    }

    if bg_adj.tint_amount > 0.001 || bg_adj.hue_shift != 0.0 || (bg_adj.saturation - 1.0).abs() > 0.001 || bg_adj.opacity < 0.999 {
        for p in img.pixels_mut() {
            let (r, g, b, a) = apply_adjustments_to_color(p[0] as f32, p[1] as f32, p[2] as f32, p[3] as f32, bg_adj, LogoColorTheme::Original);
            *p = Rgba([r, g, b, a]);
        }
    }

    img
}

// ---------------------------------------------------------------------------
// Authentic Anti-Aliased Card Assets (Embedded via include_bytes!)
// ---------------------------------------------------------------------------
const ASSET_VISA: &[u8] = include_bytes!("assets/visa.png");
const ASSET_MASTERCARD: &[u8] = include_bytes!("assets/mastercard.png");
const ASSET_NAPAS: &[u8] = include_bytes!("assets/napas.png");
const ASSET_JCB: &[u8] = include_bytes!("assets/jcb.png");
const ASSET_EMV_CHIP: &[u8] = include_bytes!("assets/emv_chip.png");
const ASSET_CONTACTLESS: &[u8] = include_bytes!("assets/contactless.png");
const ASSET_CARD_OCR: &[u8] = include_bytes!("assets/card_ocr.ttf");
const ASSET_CARD_SANS: &[u8] = include_bytes!("assets/card_sans.ttf");

fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta).rem_euclid(6.0))
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };
    let s = if max == 0.0 { 0.0 } else { delta / max };
    let v = max / 255.0;
    (h, s, v)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let h_prime = (h.rem_euclid(360.0)) / 60.0;
    let x = c * (1.0 - (h_prime.rem_euclid(2.0) - 1.0).abs());
    let (r1, g1, b1) = if h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let m = v - c;
    ((r1 + m) * 255.0, (g1 + m) * 255.0, (b1 + m) * 255.0)
}

fn apply_adjustments_to_color(
    mut r: f32,
    mut g: f32,
    mut b: f32,
    mut a: f32,
    adj: &LayerAdjustments,
    theme: LogoColorTheme,
) -> (u8, u8, u8, u8) {
    match theme {
        LogoColorTheme::Original => {}
        LogoColorTheme::MonochromeWhite => {
            r = 255.0;
            g = 255.0;
            b = 255.0;
        }
        LogoColorTheme::LuxuryGold => {
            let lum = (0.299 * r + 0.587 * g + 0.114 * b) / 255.0;
            r = (212.0 * lum + 30.0).clamp(0.0, 255.0);
            g = (175.0 * lum + 20.0).clamp(0.0, 255.0);
            b = (55.0 * lum + 5.0).clamp(0.0, 255.0);
        }
        LogoColorTheme::SilverPlatinum => {
            let lum = (0.299 * r + 0.587 * g + 0.114 * b) / 255.0;
            let val = (lum * 220.0 + 35.0).clamp(0.0, 255.0);
            r = val;
            g = val;
            b = (val * 1.05).clamp(0.0, 255.0);
        }
        LogoColorTheme::StealthBlack => {
            let lum = (0.299 * r + 0.587 * g + 0.114 * b) / 255.0;
            let val = (lum * 60.0 + 15.0).clamp(0.0, 255.0);
            r = val;
            g = val;
            b = val;
        }
    }

    if adj.hue_shift != 0.0 || (adj.saturation - 1.0).abs() > 0.001 {
        let (h, s, v) = rgb_to_hsv(r, g, b);
        let new_h = (h + adj.hue_shift).rem_euclid(360.0);
        let new_s = (s * adj.saturation).clamp(0.0, 1.0);
        let (nr, ng, nb) = hsv_to_rgb(new_h, new_s, v);
        r = nr;
        g = ng;
        b = nb;
    }

    if adj.tint_amount > 0.001 {
        let t_amt = adj.tint_amount.clamp(0.0, 1.0);
        let tr = adj.tint_color[0] as f32;
        let tg = adj.tint_color[1] as f32;
        let tb = adj.tint_color[2] as f32;
        r = r * (1.0 - t_amt) + (r * tr / 255.0) * t_amt;
        g = g * (1.0 - t_amt) + (g * tg / 255.0) * t_amt;
        b = b * (1.0 - t_amt) + (b * tb / 255.0) * t_amt;
    }

    a = (a * adj.opacity.clamp(0.0, 1.0)).clamp(0.0, 255.0);

    (
        r.clamp(0.0, 255.0).round() as u8,
        g.clamp(0.0, 255.0).round() as u8,
        b.clamp(0.0, 255.0).round() as u8,
        a.round() as u8,
    )
}

fn render_transformed_asset(
    img: &mut RgbaImage,
    asset: &RgbaImage,
    center_x: f32,
    center_y: f32,
    scale: f32,
    base_w: f32,
    base_h: f32,
    adj: &LayerAdjustments,
    theme: LogoColorTheme,
    has_shadow: bool,
) {
    let asset_w = asset.width() as f32;
    let asset_h = asset.height() as f32;
    if asset_w <= 0.0 || asset_h <= 0.0 {
        return;
    }

    let scale = scale.clamp(0.1, 10.0);
    let render_w = (base_w * scale).max(1.0);
    let render_h = (base_h * scale).max(1.0);

    let radius = ((render_w * render_w + render_h * render_h).sqrt() * 0.5 + 4.0).ceil();
    let min_x = (center_x - radius).floor().max(0.0) as i32;
    let max_x = (center_x + radius).ceil().min(CARD_WIDTH as f32 - 1.0) as i32;
    let min_y = (center_y - radius).floor().max(0.0) as i32;
    let max_y = (center_y + radius).ceil().min(CARD_HEIGHT as f32 - 1.0) as i32;

    if min_x > max_x || min_y > max_y {
        return;
    }

    let rad = (-adj.rotation).to_radians();
    let cos_r = rad.cos();
    let sin_r = rad.sin();

    if has_shadow && adj.opacity > 0.05 {
        let shadow_off_x = 3.0;
        let shadow_off_y = 4.0;
        let shadow_alpha_factor = 0.45 * adj.opacity;

        for py in min_y..=max_y {
            let dy = py as f32 + 0.5 - (center_y + shadow_off_y);
            for px in min_x..=max_x {
                let dx = px as f32 + 0.5 - (center_x + shadow_off_x);
                let rot_x = dx * cos_r - dy * sin_r;
                let rot_y = dx * sin_r + dy * cos_r;
                let u = rot_x / render_w + 0.5;
                let v = rot_y / render_h + 0.5;
                if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 {
                    continue;
                }
                let sx = (u * (asset_w - 1.0)).clamp(0.0, asset_w - 1.0);
                let sy = (v * (asset_h - 1.0)).clamp(0.0, asset_h - 1.0);
                let sx0 = sx.floor() as u32;
                let sx1 = (sx0 + 1).min(asset.width() - 1);
                let sy0 = sy.floor() as u32;
                let sy1 = (sy0 + 1).min(asset.height() - 1);
                let fx = sx - sx0 as f32;
                let fy = sy - sy0 as f32;

                let a00 = asset.get_pixel(sx0, sy0)[3] as f32;
                let a10 = asset.get_pixel(sx1, sy0)[3] as f32;
                let a01 = asset.get_pixel(sx0, sy1)[3] as f32;
                let a11 = asset.get_pixel(sx1, sy1)[3] as f32;
                let a = (a00 * (1.0 - fx) + a10 * fx) * (1.0 - fy) + (a01 * (1.0 - fx) + a11 * fx) * fy;
                if a > 8.0 {
                    let s_a = ((a / 255.0) * shadow_alpha_factor * 255.0).clamp(0.0, 255.0) as u8;
                    blend_pixel(img, px, py, 0, 0, 0, s_a);
                }
            }
        }
    }

    for py in min_y..=max_y {
        let dy = py as f32 + 0.5 - center_y;
        for px in min_x..=max_x {
            let dx = px as f32 + 0.5 - center_x;
            let rot_x = dx * cos_r - dy * sin_r;
            let rot_y = dx * sin_r + dy * cos_r;
            let u = rot_x / render_w + 0.5;
            let v = rot_y / render_h + 0.5;
            if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 {
                continue;
            }

            let sx = (u * (asset_w - 1.0)).clamp(0.0, asset_w - 1.0);
            let sy = (v * (asset_h - 1.0)).clamp(0.0, asset_h - 1.0);
            let sx0 = sx.floor() as u32;
            let sx1 = (sx0 + 1).min(asset.width() - 1);
            let sy0 = sy.floor() as u32;
            let sy1 = (sy0 + 1).min(asset.height() - 1);
            let fx = sx - sx0 as f32;
            let fy = sy - sy0 as f32;

            let p00 = asset.get_pixel(sx0, sy0);
            let p10 = asset.get_pixel(sx1, sy0);
            let p01 = asset.get_pixel(sx0, sy1);
            let p11 = asset.get_pixel(sx1, sy1);

            let interp = |idx: usize| -> f32 {
                let top = p00[idx] as f32 * (1.0 - fx) + p10[idx] as f32 * fx;
                let bot = p01[idx] as f32 * (1.0 - fx) + p11[idx] as f32 * fx;
                top * (1.0 - fy) + bot * fy
            };

            let a = interp(3);
            if a < 2.0 {
                continue;
            }

            let r = interp(0);
            let g = interp(1);
            let b = interp(2);

            let (out_r, out_g, out_b, out_a) = apply_adjustments_to_color(r, g, b, a, adj, theme);
            if out_a > 0 {
                blend_pixel(img, px, py, out_r, out_g, out_b, out_a);
            }
        }
    }
}

#[allow(dead_code)]
fn render_asset_with_theme(
    img: &mut RgbaImage,
    asset: &RgbaImage,
    dest_x: i32,
    dest_y: i32,
    dest_w: i32,
    dest_h: i32,
    theme: LogoColorTheme,
) {
    let asset_w = asset.width() as f32;
    let asset_h = asset.height() as f32;
    if asset_w <= 0.0 || asset_h <= 0.0 {
        return;
    }

    let scale = (dest_w as f32 / asset_w).min(dest_h as f32 / asset_h);
    let render_w = (asset_w * scale).round() as i32;
    let render_h = (asset_h * scale).round() as i32;
    let offset_x = dest_x + (dest_w - render_w) / 2;
    let offset_y = dest_y + (dest_h - render_h) / 2;

    for dy in 0..render_h {
        let src_y = (dy as f32 / scale).clamp(0.0, asset_h - 1.0);
        let sy0 = src_y.floor() as u32;
        let sy1 = (sy0 + 1).min(asset.height() - 1);
        let fy = src_y - sy0 as f32;

        for dx in 0..render_w {
            let src_x = (dx as f32 / scale).clamp(0.0, asset_w - 1.0);
            let sx0 = src_x.floor() as u32;
            let sx1 = (sx0 + 1).min(asset.width() - 1);
            let fx = src_x - sx0 as f32;

            let p00 = asset.get_pixel(sx0, sy0);
            let p10 = asset.get_pixel(sx1, sy0);
            let p01 = asset.get_pixel(sx0, sy1);
            let p11 = asset.get_pixel(sx1, sy1);

            let interp = |idx: usize| -> f32 {
                let top = p00[idx] as f32 * (1.0 - fx) + p10[idx] as f32 * fx;
                let bot = p01[idx] as f32 * (1.0 - fx) + p11[idx] as f32 * fx;
                top * (1.0 - fy) + bot * fy
            };

            let a = interp(3);
            if a < 2.0 {
                continue;
            }

            let r = interp(0);
            let g = interp(1);
            let b = interp(2);

            let (out_r, out_g, out_b, out_a) = match theme {
                LogoColorTheme::Original => (
                    r.clamp(0.0, 255.0) as u8,
                    g.clamp(0.0, 255.0) as u8,
                    b.clamp(0.0, 255.0) as u8,
                    a.clamp(0.0, 255.0) as u8,
                ),
                LogoColorTheme::MonochromeWhite => (
                    255,
                    255,
                    255,
                    a.clamp(0.0, 255.0) as u8,
                ),
                LogoColorTheme::LuxuryGold => (
                    245,
                    205,
                    75,
                    a.clamp(0.0, 255.0) as u8,
                ),
                LogoColorTheme::SilverPlatinum => (
                    230,
                    235,
                    245,
                    a.clamp(0.0, 255.0) as u8,
                ),
                LogoColorTheme::StealthBlack => (
                    35,
                    38,
                    45,
                    a.clamp(0.0, 255.0) as u8,
                ),
            };

            blend_pixel(img, offset_x + dx, offset_y + dy, out_r, out_g, out_b, out_a);
        }
    }
}

#[allow(dead_code)]
fn render_asset_shadow(
    img: &mut RgbaImage,
    asset: &RgbaImage,
    dest_x: i32,
    dest_y: i32,
    dest_w: i32,
    dest_h: i32,
    shadow_opacity: u8,
) {
    let asset_w = asset.width() as f32;
    let asset_h = asset.height() as f32;
    if asset_w <= 0.0 || asset_h <= 0.0 {
        return;
    }

    let scale = (dest_w as f32 / asset_w).min(dest_h as f32 / asset_h);
    let render_w = (asset_w * scale).round() as i32;
    let render_h = (asset_h * scale).round() as i32;
    let offset_x = dest_x + (dest_w - render_w) / 2;
    let offset_y = dest_y + (dest_h - render_h) / 2;

    for dy in 0..render_h {
        let src_y = (dy as f32 / scale).clamp(0.0, asset_h - 1.0) as u32;
        for dx in 0..render_w {
            let src_x = (dx as f32 / scale).clamp(0.0, asset_w - 1.0) as u32;
            let p = asset.get_pixel(src_x, src_y);
            if p[3] > 8 {
                let alpha = ((p[3] as f32 / 255.0) * shadow_opacity as f32) as u8;
                blend_pixel(img, offset_x + dx, offset_y + dy, 0, 0, 0, alpha);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// EMV Gold / Custom Smart Chip
// ---------------------------------------------------------------------------
fn draw_emv_chip(
    img: &mut RgbaImage,
    custom_chip: Option<&RgbaImage>,
    chip_x: f32,
    chip_y: f32,
    chip_scale: f32,
    chip_adj: &LayerAdjustments,
) {
    let base_w = 205.0f32;
    let base_h = 155.0f32;
    if let Some(chip) = custom_chip {
        render_transformed_asset(
            img,
            chip,
            chip_x,
            chip_y,
            chip_scale,
            base_w,
            base_h,
            chip_adj,
            LogoColorTheme::Original,
            true,
        );
    } else if let Ok(dyn_img) = image::load_from_memory(ASSET_EMV_CHIP) {
        let chip = dyn_img.to_rgba8();
        render_transformed_asset(
            img,
            &chip,
            chip_x,
            chip_y,
            chip_scale,
            base_w,
            base_h,
            chip_adj,
            LogoColorTheme::Original,
            true,
        );
    }
}

// ---------------------------------------------------------------------------
// Contactless Waves Indicator (Shockwave) with position, scale, rotation, color
// ---------------------------------------------------------------------------
fn draw_contactless_wave(
    img: &mut RgbaImage,
    wave_x: f32,
    wave_y: f32,
    wave_scale: f32,
    wave_adj: &LayerAdjustments,
) {
    if let Ok(dyn_img) = image::load_from_memory(ASSET_CONTACTLESS) {
        let wave = dyn_img.to_rgba8();
        render_transformed_asset(
            img,
            &wave,
            wave_x,
            wave_y,
            wave_scale,
            75.0,
            95.0,
            wave_adj,
            LogoColorTheme::MonochromeWhite,
            true,
        );
    }
}

// ---------------------------------------------------------------------------
// Payment Network Logos & Badges
// ---------------------------------------------------------------------------
fn draw_payment_network(
    img: &mut RgbaImage,
    network: PaymentNetwork,
    badge_style: LogoBadgeStyle,
    color_theme: LogoColorTheme,
    custom_logo: Option<&RgbaImage>,
    logo_x: f32,
    logo_y: f32,
    logo_scale: f32,
    logo_adj: &LayerAdjustments,
) {
    let scale = logo_scale.clamp(0.2, 5.0);
    let bw = (245.0 * scale).round() as i32;
    let bh = (94.0 * scale).round() as i32;
    let bx0 = logo_x.round() as i32;
    let by0 = logo_y.round() as i32;
    let radius = (18.0 * scale).max(4.0);

    draw_logo_badge_frame(img, bx0, by0, bw, bh, radius, badge_style);

    let (_dest_x, _dest_y, _dest_w, _dest_h) = if badge_style == LogoBadgeStyle::Transparent {
        (bx0 - (10.0 * scale) as i32, by0 - (8.0 * scale) as i32, bw + (20.0 * scale) as i32, bh + (16.0 * scale) as i32)
    } else {
        (bx0 + (14.0 * scale) as i32, by0 + (10.0 * scale) as i32, bw - (28.0 * scale) as i32, bh - (20.0 * scale) as i32)
    };

    let maybe_logo_img = match network {
        PaymentNetwork::None => None,
        PaymentNetwork::Visa => image::load_from_memory(ASSET_VISA).ok().map(|d| d.to_rgba8()),
        PaymentNetwork::Mastercard => image::load_from_memory(ASSET_MASTERCARD).ok().map(|d| d.to_rgba8()),
        PaymentNetwork::Napas => image::load_from_memory(ASSET_NAPAS).ok().map(|d| d.to_rgba8()),
        PaymentNetwork::Jcb => image::load_from_memory(ASSET_JCB).ok().map(|d| d.to_rgba8()),
        PaymentNetwork::Custom => custom_logo.cloned(),
    };

    if let Some(logo) = maybe_logo_img {
        let has_shadow = badge_style == LogoBadgeStyle::Transparent || badge_style == LogoBadgeStyle::ThinOutline;
        render_transformed_asset(
            img,
            &logo,
            logo_x,
            logo_y,
            logo_scale,
            245.0,
            94.0,
            logo_adj,
            color_theme,
            has_shadow,
        );
    }
}

fn draw_logo_badge_frame(
    img: &mut RgbaImage,
    x0: i32,
    y0: i32,
    w: i32,
    h: i32,
    radius: f32,
    style: LogoBadgeStyle,
) {
    match style {
        LogoBadgeStyle::Transparent => {}
        LogoBadgeStyle::ThinOutline => {
            draw_rounded_rect_outline(img, x0, y0, w, h, radius, 2.0, [255, 255, 255, 130]);
        }
        LogoBadgeStyle::FrostedGlass => {
            draw_rounded_rect(img, x0 + 3, y0 + 4, w, h, radius, [0, 0, 0, 95]);
            draw_rounded_rect(img, x0, y0, w, h, radius, [255, 255, 255, 42]);
            draw_rounded_rect_outline(img, x0, y0, w, h, radius, 1.5, [255, 255, 255, 125]);
        }
        LogoBadgeStyle::SolidDark => {
            draw_rounded_rect(img, x0 + 3, y0 + 4, w, h, radius, [0, 0, 0, 130]);
            draw_rounded_rect(img, x0, y0, w, h, radius, [22, 24, 28, 235]);
            draw_rounded_rect_outline(img, x0, y0, w, h, radius, 1.0, [75, 80, 90, 150]);
        }
        LogoBadgeStyle::SolidLight => {
            draw_rounded_rect(img, x0 + 3, y0 + 4, w, h, radius, [0, 0, 0, 90]);
            draw_rounded_rect(img, x0, y0, w, h, radius, [248, 249, 252, 245]);
            draw_rounded_rect_outline(img, x0, y0, w, h, radius, 1.0, [210, 215, 225, 200]);
        }
        LogoBadgeStyle::SubtleGlow => {
            let cx = x0 + w / 2;
            let cy = y0 + h / 2;
            let max_r = 95.0f32;
            let r2 = max_r * max_r;
            for dy in -95..=95 {
                for dx in -120..=120 {
                    let d2 = (dx * dx + dy * dy) as f32;
                    if d2 <= r2 {
                        let factor = ((max_r - d2.sqrt()) / max_r).powi(2) * 45.0;
                        blend_pixel(img, cx + dx, cy + dy, 255, 255, 255, factor as u8);
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// High-Precision Vector Typography & Grounded Card Details Engine
// ---------------------------------------------------------------------------
fn render_text_subpixel(
    img: &mut RgbaImage,
    font: &FontRef,
    text: &str,
    start_x: f32,
    start_y: f32,
    scale_px: f32,
    color: [u8; 4],
    letter_spacing: f32,
) -> f32 {
    let scale = PxScale::from(scale_px);
    let scaled = font.as_scaled(scale);
    let mut cur_x = start_x;

    for c in text.chars() {
        if c == ' ' {
            cur_x += scaled.h_advance(font.glyph_id(' ')) + letter_spacing;
            continue;
        }
        let id = font.glyph_id(c);
        let advance = scaled.h_advance(id);
        let glyph = id.with_scale_and_position(scale, point(cur_x, start_y));
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|gx, gy, cov| {
                if cov > 0.01 {
                    let px = bounds.min.x as i32 + gx as i32;
                    let py = bounds.min.y as i32 + gy as i32;
                    let a = ((color[3] as f32) * cov).round() as u8;
                    blend_pixel(img, px, py, color[0], color[1], color[2], a);
                }
            });
        }
        cur_x += advance + letter_spacing;
    }
    cur_x
}

fn draw_embossed_string_vector(
    img: &mut RgbaImage,
    font: &FontRef,
    start_x: f32,
    start_y: f32,
    text: &str,
    scale_px: f32,
    style: EmbossStyle,
    letter_spacing: f32,
) -> f32 {
    let (shadow_col, highlight_col, face_col, is_3d) = match style {
        EmbossStyle::EmbossedSilver => (
            [10, 10, 15, 230],
            [255, 255, 255, 220],
            [225, 232, 240, 255],
            true,
        ),
        EmbossStyle::EmbossedGold => (
            [35, 25, 5, 230],
            [255, 250, 215, 220],
            [240, 198, 68, 255],
            true,
        ),
        EmbossStyle::CrispWhite => (
            [0, 0, 0, 210],
            [0, 0, 0, 0],
            [255, 255, 255, 255],
            false,
        ),
        EmbossStyle::StealthDark => (
            [80, 85, 95, 160],
            [200, 205, 215, 100],
            [35, 38, 45, 255],
            false,
        ),
    };

    // 1. Ambient Contact Shadow (Multi-tap diffuse shadow for deep grounding)
    if shadow_col[3] > 0 {
        let soft_alpha = (shadow_col[3] as f32 * 0.35) as u8;
        render_text_subpixel(
            img,
            font,
            text,
            start_x + 3.0,
            start_y + 4.0,
            scale_px,
            [shadow_col[0], shadow_col[1], shadow_col[2], soft_alpha],
            letter_spacing,
        );
        render_text_subpixel(
            img,
            font,
            text,
            start_x + 1.0,
            start_y + 3.0,
            scale_px,
            [shadow_col[0], shadow_col[1], shadow_col[2], soft_alpha],
            letter_spacing,
        );
        let mid_alpha = (shadow_col[3] as f32 * 0.70) as u8;
        render_text_subpixel(
            img,
            font,
            text,
            start_x + 2.0,
            start_y + 2.0,
            scale_px,
            [shadow_col[0], shadow_col[1], shadow_col[2], mid_alpha],
            letter_spacing,
        );
    }

    // 2. 3D Specular Highlight layer (Light catch on top-left)
    if is_3d && highlight_col[3] > 0 {
        render_text_subpixel(
            img,
            font,
            text,
            start_x - 1.5,
            start_y - 1.5,
            scale_px,
            highlight_col,
            letter_spacing,
        );
    }

    // 3. Crisp Face layer
    render_text_subpixel(
        img,
        font,
        text,
        start_x,
        start_y,
        scale_px,
        face_col,
        letter_spacing,
    )
}

fn draw_text_backdrop(img: &mut RgbaImage, backdrop: TextBackdropStyle, vertical_offset: i32) {
    match backdrop {
        TextBackdropStyle::None => {}
        TextBackdropStyle::FrostedGlassStrip => {
            let y0 = (530 + vertical_offset).clamp(100, 750);
            let h = 330;
            // Translucent dark frosted glass strip
            draw_rounded_rect(img, 80, y0, 1376, h, 24.0, [15, 18, 26, 125]);
            // Subtle frosted border
            draw_rounded_rect_outline(img, 80, y0, 1376, h, 24.0, 1.5, [255, 255, 255, 45]);
        }
        TextBackdropStyle::SubtleDarkGradient => {
            let y_start = (480 + vertical_offset).clamp(200, 700) as u32;
            for y in y_start..CARD_HEIGHT {
                let factor = ((y - y_start) as f32 / (CARD_HEIGHT - y_start) as f32).powf(1.4);
                let alpha = (factor * 125.0) as u8; // up to ~50% dark
                for x in 0..CARD_WIDTH {
                    blend_pixel(img, x as i32, y as i32, 12, 14, 20, alpha);
                }
            }
        }
        TextBackdropStyle::FrostedPills => {
            let vo = vertical_offset;
            // Pill for card number
            let y_num = (575 + vo - 12).clamp(100, 850);
            draw_rounded_rect(img, 115, y_num, 930, 80, 18.0, [15, 18, 26, 115]);
            draw_rounded_rect_outline(img, 115, y_num, 930, 80, 18.0, 1.2, [255, 255, 255, 38]);

            // Pill for expiry
            let y_exp = (670 + vo - 8).clamp(100, 900);
            draw_rounded_rect(img, 520, y_exp, 320, 56, 14.0, [15, 18, 26, 115]);
            draw_rounded_rect_outline(img, 520, y_exp, 320, 56, 14.0, 1.2, [255, 255, 255, 38]);

            // Pill for cardholder name
            let y_name = (765 + vo - 8).clamp(100, 920);
            draw_rounded_rect(img, 115, y_name, 650, 64, 16.0, [15, 18, 26, 115]);
            draw_rounded_rect_outline(img, 115, y_name, 650, 64, 16.0, 1.2, [255, 255, 255, 38]);
        }
    }
}

fn draw_card_details(img: &mut RgbaImage, details: &CardDetails, _details_adj: &LayerAdjustments) {
    let ocr_font = match FontRef::try_from_slice(ASSET_CARD_OCR) {
        Ok(f) => f,
        Err(_) => return,
    };
    let sans_font = match FontRef::try_from_slice(ASSET_CARD_SANS) {
        Ok(f) => f,
        Err(_) => return,
    };

    let vo = details.vertical_offset as f32;

    // 0. Optional Text Grounding Backdrop
    draw_text_backdrop(img, details.backdrop, details.vertical_offset);

    // 1. Bank Name / Card Type (Top row, clean modern sans)
    if !details.card_type_or_bank.is_empty() {
        draw_embossed_string_vector(
            img,
            &sans_font,
            140.0,
            125.0,
            &details.card_type_or_bank.to_uppercase(),
            44.0,
            details.emboss_style,
            1.5,
        );
    }

    // 2. Card Number (Official ISO 7813 embossed OCR font)
    if !details.card_number.is_empty() {
        draw_embossed_string_vector(
            img,
            &ocr_font,
            140.0,
            580.0 + vo,
            &details.card_number,
            58.0,
            details.emboss_style,
            3.0,
        );
    }

    // 3. Expiry label & date (e.g. "VALID THRU 09/29")
    if !details.card_expiry.is_empty() {
        // Small "VALID THRU" label using clean sans font
        draw_embossed_string_vector(
            img,
            &sans_font,
            535.0,
            672.0 + vo,
            "VALID THRU",
            22.0,
            details.emboss_style,
            1.0,
        );
        // Expiry date numbers using OCR font
        draw_embossed_string_vector(
            img,
            &ocr_font,
            680.0,
            670.0 + vo,
            &details.card_expiry,
            38.0,
            details.emboss_style,
            2.0,
        );
    }

    // 4. Cardholder Name (Bottom row, official OCR font)
    if !details.card_holder.is_empty() {
        draw_embossed_string_vector(
            img,
            &ocr_font,
            140.0,
            775.0 + vo,
            &details.card_holder.to_uppercase(),
            42.0,
            details.emboss_style,
            2.0,
        );
    }
}

// ---------------------------------------------------------------------------
// Card Surface Finishes
// ---------------------------------------------------------------------------
fn apply_custom_finish_texture(
    img: &mut RgbaImage,
    texture: &RgbaImage,
    pan_x: f32,
    pan_y: f32,
    scale: f32,
    opacity: f32,
    finish_adj: &LayerAdjustments,
) {
    let mut adj = *finish_adj;
    adj.opacity *= opacity;
    render_transformed_asset(
        img,
        texture,
        CARD_WIDTH as f32 * 0.5 + pan_x,
        CARD_HEIGHT as f32 * 0.5 + pan_y,
        scale,
        CARD_WIDTH as f32,
        CARD_HEIGHT as f32,
        &adj,
        LogoColorTheme::Original,
        false,
    );
}

#[allow(dead_code)]
fn apply_custom_finish_texture_legacy(
    img: &mut RgbaImage,
    texture: &RgbaImage,
    pan_x: f32,
    pan_y: f32,
    scale: f32,
    opacity: f32,
) {
    let (tex_w, tex_h) = texture.dimensions();
    if tex_w == 0 || tex_h == 0 || opacity <= 0.001 {
        return;
    }

    let scale = scale.clamp(0.05, 50.0);
    let dest_w = tex_w as f32 * scale;
    let dest_h = tex_h as f32 * scale;

    let offset_x = (CARD_WIDTH as f32 - dest_w) * 0.5 + pan_x;
    let offset_y = (CARD_HEIGHT as f32 - dest_h) * 0.5 + pan_y;

    let x_min = (offset_x.floor() as i32).clamp(0, CARD_WIDTH as i32) as u32;
    let x_max = ((offset_x + dest_w).ceil() as i32).clamp(0, CARD_WIDTH as i32) as u32;
    let y_min = (offset_y.floor() as i32).clamp(0, CARD_HEIGHT as i32) as u32;
    let y_max = ((offset_y + dest_h).ceil() as i32).clamp(0, CARD_HEIGHT as i32) as u32;

    if x_min >= x_max || y_min >= y_max {
        return;
    }

    let inv_scale = 1.0 / scale;
    let tex_raw = texture.as_raw();
    let stride = tex_w as usize * 4;
    let max_x = tex_w - 1;
    let max_y = tex_h - 1;
    let opacity_factor = opacity.clamp(0.0, 1.0);

    for y in y_min..y_max {
        let dy = (y as f32 + 0.5) - offset_y;
        let sy = (dy * inv_scale - 0.5).clamp(0.0, max_y as f32);
        let y0 = sy.floor() as u32;
        let y1 = (y0 + 1).min(max_y);
        let fy = sy - y0 as f32;
        let inv_fy = 1.0 - fy;

        let row0 = y0 as usize * stride;
        let row1 = y1 as usize * stride;

        for x in x_min..x_max {
            let dx = (x as f32 + 0.5) - offset_x;
            let sx = (dx * inv_scale - 0.5).clamp(0.0, max_x as f32);
            let x0 = sx.floor() as u32;
            let x1 = (x0 + 1).min(max_x);
            let fx = sx - x0 as f32;
            let inv_fx = 1.0 - fx;

            let w00 = inv_fx * inv_fy;
            let w10 = fx * inv_fy;
            let w01 = inv_fx * fy;
            let w11 = fx * fy;

            let idx00 = row0 + x0 as usize * 4;
            let idx10 = row0 + x1 as usize * 4;
            let idx01 = row1 + x0 as usize * 4;
            let idx11 = row1 + x1 as usize * 4;

            let r = (tex_raw[idx00] as f32 * w00
                + tex_raw[idx10] as f32 * w10
                + tex_raw[idx01] as f32 * w01
                + tex_raw[idx11] as f32 * w11)
                .round() as u8;
            let g = (tex_raw[idx00 + 1] as f32 * w00
                + tex_raw[idx10 + 1] as f32 * w10
                + tex_raw[idx01 + 1] as f32 * w01
                + tex_raw[idx11 + 1] as f32 * w11)
                .round() as u8;
            let b = (tex_raw[idx00 + 2] as f32 * w00
                + tex_raw[idx10 + 2] as f32 * w10
                + tex_raw[idx01 + 2] as f32 * w01
                + tex_raw[idx11 + 2] as f32 * w11)
                .round() as u8;
            let a = (tex_raw[idx00 + 3] as f32 * w00
                + tex_raw[idx10 + 3] as f32 * w10
                + tex_raw[idx01 + 3] as f32 * w01
                + tex_raw[idx11 + 3] as f32 * w11)
                .round() as u8;

            let effective_alpha = ((a as f32 / 255.0) * opacity_factor * 255.0).round() as u8;
            blend_pixel(img, x as i32, y as i32, r, g, b, effective_alpha);
        }
    }
}

fn apply_surface_finish(
    img: &mut RgbaImage,
    finish: CardFinish,
    custom_finish: Option<&RgbaImage>,
    options: &CardOverlayOptions,
) {
    match finish {
        CardFinish::Standard => {}
        CardFinish::MetallicSheen => {
            // Diagonal specular reflection highlight band across the card
            let cx = CARD_WIDTH as f32 * 0.45;
            let cy = CARD_HEIGHT as f32 * 0.45;

            for y in 0..CARD_HEIGHT {
                for x in 0..CARD_WIDTH {
                    // Distance to diagonal line: (x + y) / sqrt(2)
                    let d = ((x as f32 + y as f32) - (cx + cy)).abs();
                    if d < 320.0 {
                        let factor = ((320.0 - d) / 320.0).powi(2) * 0.28;
                        let px = img.get_pixel_mut(x, y);
                        let r = (px[0] as f32 + (255.0 - px[0] as f32) * factor).min(255.0) as u8;
                        let g = (px[1] as f32 + (255.0 - px[1] as f32) * factor).min(255.0) as u8;
                        let b = (px[2] as f32 + (255.0 - px[2] as f32) * factor).min(255.0) as u8;
                        *px = Rgba([r, g, b, 255]);
                    }
                }
            }
        }
        CardFinish::CarbonWeave => {
            // Subtle carbon fiber weave texture
            for y in 0..CARD_HEIGHT {
                for x in 0..CARD_WIDTH {
                    let block_x = (x / 6) % 2;
                    let block_y = (y / 6) % 2;
                    let is_bright = block_x == block_y;
                    let delta = if is_bright { 18.0 } else { -18.0 };

                    let px = img.get_pixel_mut(x, y);
                    let r = (px[0] as f32 + delta).clamp(0.0, 255.0) as u8;
                    let g = (px[1] as f32 + delta).clamp(0.0, 255.0) as u8;
                    let b = (px[2] as f32 + delta).clamp(0.0, 255.0) as u8;
                    *px = Rgba([r, g, b, 255]);
                }
            }
        }
        CardFinish::CustomTexture => {
            if let Some(texture) = custom_finish {
                apply_custom_finish_texture(
                    img,
                    texture,
                    options.finish_x,
                    options.finish_y,
                    options.finish_scale,
                    options.finish_opacity,
                    &options.finish_adj,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Card Rounded Corner Clipping (ISO 7810 ID-1 Standard)
// ---------------------------------------------------------------------------
pub fn apply_card_rounded_corners(img: &mut RgbaImage, radius: f32) {
    let w = img.width() as f32;
    let h = img.height() as f32;
    let r2 = radius * radius;
    let r_inner = radius - 0.75;
    let r_outer = radius + 0.75;

    for y in 0..img.height() {
        let yf = y as f32 + 0.5;
        let cy = if yf < radius {
            radius
        } else if yf > h - radius {
            h - radius
        } else {
            continue;
        };

        for x in 0..img.width() {
            let xf = x as f32 + 0.5;
            let cx = if xf < radius {
                radius
            } else if xf > w - radius {
                w - radius
            } else {
                continue;
            };

            let dx = xf - cx;
            let dy = yf - cy;
            let dist2 = dx * dx + dy * dy;

            if dist2 > r2 {
                let dist = dist2.sqrt();
                if dist >= r_outer {
                    img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                } else {
                    let coverage = ((r_outer - dist) / (r_outer - r_inner)).clamp(0.0, 1.0);
                    let p = img.get_pixel_mut(x, y);
                    p[3] = (p[3] as f32 * coverage).round() as u8;
                }
            }
        }
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
        ).expect("Custom finish and chip rendering failed");

        assert_eq!(skin.preview.size, [CARD_WIDTH as usize, CARD_HEIGHT as usize]);
        assert!(!skin.png.is_empty());
        assert!(!skin.pdf.is_empty());
    }
}
