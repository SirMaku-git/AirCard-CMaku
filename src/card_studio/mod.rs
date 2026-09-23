pub mod types;
pub mod widget;
pub mod render;
pub mod ui;
pub mod sources;

pub use types::*;
pub use widget::*;

use std::path::PathBuf;
use anyhow::Result;
use eframe::egui;
use image::{DynamicImage, RgbaImage};

pub struct CardStudioState {
    pub overlay_options: CardOverlayOptions,
    pub active_layer: ActiveTransformLayer,
    pub custom_logo_image: Option<RgbaImage>,
    pub custom_logo_path: Option<PathBuf>,
    pub custom_chip_image: Option<RgbaImage>,
    pub custom_chip_path: Option<PathBuf>,
    pub custom_finish_image: Option<RgbaImage>,
    pub custom_finish_path: Option<PathBuf>,
    pub custom_font_bytes: Option<Vec<u8>>,
    pub custom_font_path: Option<PathBuf>,
    pub custom_font_name: Option<String>,
    pub custom_widgets: Vec<CustomWidget>,
}

impl Default for CardStudioState {
    fn default() -> Self {
        Self::new()
    }
}

impl CardStudioState {
    pub fn new() -> Self {
        Self {
            overlay_options: CardOverlayOptions::default(),
            active_layer: ActiveTransformLayer::Background,
            custom_logo_image: None,
            custom_logo_path: None,
            custom_chip_image: None,
            custom_chip_path: None,
            custom_finish_image: None,
            custom_finish_path: None,
            custom_font_bytes: None,
            custom_font_path: None,
            custom_font_name: None,
            custom_widgets: Vec::new(),
        }
    }

    pub fn reset_to_defaults(&mut self) {
        self.overlay_options = CardOverlayOptions::default();
        self.custom_logo_image = None;
        self.custom_logo_path = None;
        self.custom_chip_image = None;
        self.custom_chip_path = None;
        self.custom_finish_image = None;
        self.custom_finish_path = None;
        self.custom_font_bytes = None;
        self.custom_font_path = None;
        self.custom_font_name = None;
        self.custom_widgets.clear();
        self.active_layer = ActiveTransformLayer::Background;
    }

    pub fn add_custom_widget(&mut self) -> Result<String, String> {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Select Custom Widget Image (PNG/JPG/WebP)")
            .add_filter("Image files", &["png", "jpg", "jpeg", "webp"])
            .pick_file()
        else {
            return Err("Cancelled by user".into());
        };

        let next_id = self.custom_widgets.iter().map(|w| w.data.id).max().unwrap_or(0) + 1;
        match CustomWidget::from_image_path(path.clone(), next_id, self.custom_widgets.len()) {
            Ok(widget) => {
                let name = widget.data.name.clone();
                let new_idx = self.custom_widgets.len();
                self.custom_widgets.push(widget);
                self.active_layer = ActiveTransformLayer::Widget(new_idx);
                Ok(format!("Added custom widget #{next_id}: {name}"))
            }
            Err(e) => Err(format!("Failed to open widget image {}: {e:#}", path.display())),
        }
    }

    pub fn remove_custom_widget(&mut self, idx: usize) {
        if idx < self.custom_widgets.len() {
            self.custom_widgets.remove(idx);
            if self.active_layer == ActiveTransformLayer::Widget(idx) {
                if !self.custom_widgets.is_empty() {
                    let new_idx = idx.min(self.custom_widgets.len() - 1);
                    self.active_layer = ActiveTransformLayer::Widget(new_idx);
                } else {
                    self.active_layer = ActiveTransformLayer::Background;
                }
            } else if let ActiveTransformLayer::Widget(curr_idx) = self.active_layer {
                if curr_idx > idx {
                    self.active_layer = ActiveTransformLayer::Widget(curr_idx - 1);
                }
            }
        }
    }

    pub fn select_custom_logo(&mut self) -> Result<String, String> {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Select Custom Logo Image (PNG/JPG)")
            .add_filter("Image files", &["png", "jpg", "jpeg", "webp"])
            .pick_file()
        else {
            return Err("Cancelled by user".into());
        };

        match image::open(&path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                self.custom_logo_image = Some(rgba);
                self.custom_logo_path = Some(path.clone());
                self.overlay_options.network = PaymentNetwork::Custom;
                self.active_layer = ActiveTransformLayer::Logo;
                Ok(format!("Custom logo loaded: {}", path.display()))
            }
            Err(e) => Err(format!("Failed to open custom logo {}: {e:#}", path.display())),
        }
    }

    pub fn select_custom_chip(&mut self) -> Result<String, String> {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Select Custom EMV Chip Image (PNG with transparency)")
            .add_filter("Image files", &["png", "webp"])
            .pick_file()
        else {
            return Err("Cancelled by user".into());
        };

        match image::open(&path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                self.custom_chip_image = Some(rgba);
                self.custom_chip_path = Some(path.clone());
                self.overlay_options.show_chip = true;
                self.active_layer = ActiveTransformLayer::Chip;
                Ok(format!("Custom chip loaded: {}", path.display()))
            }
            Err(e) => Err(format!("Failed to open custom chip {}: {e:#}", path.display())),
        }
    }

    pub fn select_custom_finish(&mut self) -> Result<String, String> {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Select Custom Texture / Foil Image (PNG/JPG/WebP)")
            .add_filter("Image files", &["png", "jpg", "jpeg", "webp"])
            .pick_file()
        else {
            return Err("Cancelled by user".into());
        };

        match image::open(&path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                self.custom_finish_image = Some(rgba);
                self.custom_finish_path = Some(path.clone());
                self.overlay_options.finish = CardFinish::CustomTexture;
                self.active_layer = ActiveTransformLayer::Finish;
                Ok(format!("Custom finish texture loaded: {}", path.display()))
            }
            Err(e) => Err(format!("Failed to open finish texture {}: {e:#}", path.display())),
        }
    }

    pub fn load_custom_font(&mut self) -> Result<String, String> {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Select Custom Font File (.ttf / .otf)")
            .add_filter("Font files (*.ttf, *.otf)", &["ttf", "otf", "TTF", "OTF"])
            .pick_file()
        else {
            return Err("Cancelled by user".into());
        };

        match std::fs::read(&path) {
            Ok(bytes) => {
                if let Err(e) = ab_glyph::FontArc::try_from_vec(bytes.clone()) {
                    return Err(format!("Invalid font file (cannot parse TTF/OTF): {e}"));
                }
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("custom_font.ttf")
                    .to_string();

                self.custom_font_bytes = Some(bytes);
                self.custom_font_path = Some(path.clone());
                self.custom_font_name = Some(file_name.clone());

                self.overlay_options.details.custom_font_path = Some(path.display().to_string());
                self.overlay_options.details.custom_font_name = Some(file_name.clone());
                self.overlay_options.details.number_font = CardFontPreset::Custom;
                self.overlay_options.details.text_font = CardFontPreset::Custom;
                self.active_layer = ActiveTransformLayer::Details;

                Ok(format!("Custom font loaded: {file_name}"))
            }
            Err(e) => Err(format!("Failed to read font file {}: {e:#}", path.display())),
        }
    }

    pub fn clear_custom_font(&mut self) {
        self.custom_font_bytes = None;
        self.custom_font_path = None;
        self.custom_font_name = None;
        self.overlay_options.details.custom_font_path = None;
        self.overlay_options.details.custom_font_name = None;
        if self.overlay_options.details.number_font == CardFontPreset::Custom {
            self.overlay_options.details.number_font = CardFontPreset::ClassicOcr;
        }
        if self.overlay_options.details.text_font == CardFontPreset::Custom {
            self.overlay_options.details.text_font = CardFontPreset::ModernSans;
        }
    }

    pub fn render_preview_canvas(
        &self,
        raw_image: Option<&DynamicImage>,
    ) -> Result<(RgbaImage, egui::ColorImage)> {
        let widget_refs: Vec<(&CustomWidgetData, &RgbaImage)> = self
            .custom_widgets
            .iter()
            .map(|w| (&w.data, &w.image))
            .collect();
        crate::card_studio::render::render_preview_canvas(
            raw_image,
            &self.overlay_options,
            self.custom_logo_image.as_ref(),
            self.custom_chip_image.as_ref(),
            self.custom_finish_image.as_ref(),
            self.custom_font_bytes.as_deref(),
            &widget_refs,
        )
    }

    pub fn draw_sidebar(&mut self, ui: &mut egui::Ui, is_vi: bool) -> bool {
        crate::card_studio::ui::draw_studio_sidebar(self, ui, is_vi)
    }

    pub fn draw_preview_card(
        &mut self,
        ui: &mut egui::Ui,
        skin_texture: Option<&egui::TextureHandle>,
        is_vi: bool,
    ) -> bool {
        crate::card_studio::ui::draw_studio_preview(self, ui, skin_texture, is_vi)
    }
}
