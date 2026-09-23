pub const CARD_WIDTH: u32 = 1_536;
pub const CARD_HEIGHT: u32 = 969;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ActiveTransformLayer {
    #[default]
    Background,
    Finish,
    Chip,
    Wave,
    Logo,
    Details,
    Widget(usize),
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum CardFontPreset {
    #[default]
    ClassicOcr,
    ModernSans,
    Monospace,
    SerifLuxury,
    Custom,
}

impl CardFontPreset {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ClassicOcr => "Classic Card OCR (Farrington 7B)",
            Self::ModernSans => "Modern Sans (Apple / SF style)",
            Self::Monospace => "Monospace Code (Consolas)",
            Self::SerifLuxury => "Serif Luxury (Georgia)",
            Self::Custom => "Custom Font (User Loaded...)",
        }
    }

    pub fn display_name_lang(&self, is_vi: bool) -> &'static str {
        if !is_vi { return self.display_name(); }
        match self {
            Self::ClassicOcr => "OCR Thẻ kinh điển (Farrington 7B)",
            Self::ModernSans => "Sans Hiện đại (Apple / SF Pro)",
            Self::Monospace => "Monospace (Mã lập trình)",
            Self::SerifLuxury => "Serif Sang trọng (Thẻ VIP / Georgia)",
            Self::Custom => "Font Tùy Chọn (Đã nạp file)",
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CardDetails {
    pub show_details: bool,
    pub card_number: String,
    pub card_holder: String,
    pub card_expiry: String,
    pub card_type_or_bank: String,
    pub emboss_style: EmbossStyle,
    pub backdrop: TextBackdropStyle,
    pub vertical_offset: i32,
    pub horizontal_offset: i32,
    pub scale: f32,
    #[serde(default)]
    pub number_font: CardFontPreset,
    #[serde(default)]
    pub text_font: CardFontPreset,
    #[serde(default)]
    pub custom_font_path: Option<String>,
    #[serde(default)]
    pub custom_font_name: Option<String>,
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
            horizontal_offset: 0,
            scale: 1.0,
            number_font: CardFontPreset::ClassicOcr,
            text_font: CardFontPreset::ModernSans,
            custom_font_path: None,
            custom_font_name: None,
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
