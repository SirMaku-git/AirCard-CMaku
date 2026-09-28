pub mod types;
pub mod widget;
pub mod render;
pub mod ui;
pub mod sources;
pub mod workspace;

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
    pub custom_logo_raw: Option<(Vec<u8>, String)>,
    pub custom_chip_image: Option<RgbaImage>,
    pub custom_chip_path: Option<PathBuf>,
    pub custom_chip_raw: Option<(Vec<u8>, String)>,
    pub custom_finish_image: Option<RgbaImage>,
    pub custom_finish_path: Option<PathBuf>,
    pub custom_finish_raw: Option<(Vec<u8>, String)>,
    pub custom_font_bytes: Option<Vec<u8>>,
    pub custom_font_path: Option<PathBuf>,
    pub custom_font_name: Option<String>,
    pub custom_widgets: Vec<CustomWidget>,
    pub selected_text_index: usize,
    pub save_workspace_requested: bool,
    pub load_workspace_requested: bool,
    pub select_skin_requested: bool,
    pub export_png_requested: bool,
    pub skin_info: Option<String>,
    pub can_export_png: bool,
    pub undo_stack: Vec<StudioSnapshot>,
    pub redo_stack: Vec<StudioSnapshot>,
    pub pre_interaction_snapshot: Option<StudioSnapshot>,
    pub last_interaction_time: f64,
}

pub const MAX_UNDO_STEPS: usize = 30;

#[derive(Clone)]
pub struct StudioSnapshot {
    pub overlay_options: CardOverlayOptions,
    pub custom_widgets: Vec<CustomWidget>,
    pub active_layer: ActiveTransformLayer,
    pub selected_text_index: usize,
    pub custom_logo_image: Option<RgbaImage>,
    pub custom_logo_path: Option<PathBuf>,
    pub custom_logo_raw: Option<(Vec<u8>, String)>,
    pub custom_chip_image: Option<RgbaImage>,
    pub custom_chip_path: Option<PathBuf>,
    pub custom_chip_raw: Option<(Vec<u8>, String)>,
    pub custom_finish_image: Option<RgbaImage>,
    pub custom_finish_path: Option<PathBuf>,
    pub custom_finish_raw: Option<(Vec<u8>, String)>,
    pub custom_font_bytes: Option<Vec<u8>>,
    pub custom_font_path: Option<PathBuf>,
    pub custom_font_name: Option<String>,
}

impl StudioSnapshot {
    pub fn is_visually_equal(&self, other: &StudioSnapshot) -> bool {
        if self.overlay_options != other.overlay_options {
            return false;
        }
        if self.active_layer != other.active_layer {
            return false;
        }
        if self.selected_text_index != other.selected_text_index {
            return false;
        }
        if self.custom_widgets.len() != other.custom_widgets.len() {
            return false;
        }
        for (w1, w2) in self.custom_widgets.iter().zip(other.custom_widgets.iter()) {
            if w1.data != w2.data {
                return false;
            }
        }
        if self.custom_logo_path != other.custom_logo_path
            || self.custom_chip_path != other.custom_chip_path
            || self.custom_finish_path != other.custom_finish_path
            || self.custom_font_path != other.custom_font_path
            || self.custom_font_name != other.custom_font_name
        {
            return false;
        }
        true
    }
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
            custom_logo_raw: None,
            custom_chip_image: None,
            custom_chip_path: None,
            custom_chip_raw: None,
            custom_finish_image: None,
            custom_finish_path: None,
            custom_finish_raw: None,
            custom_font_bytes: None,
            custom_font_path: None,
            custom_font_name: None,
            custom_widgets: Vec::new(),
            selected_text_index: 0,
            save_workspace_requested: false,
            load_workspace_requested: false,
            select_skin_requested: false,
            export_png_requested: false,
            skin_info: None,
            can_export_png: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            pre_interaction_snapshot: None,
            last_interaction_time: 0.0,
        }
    }

    pub fn create_snapshot(&self) -> StudioSnapshot {
        StudioSnapshot {
            overlay_options: self.overlay_options.clone(),
            custom_widgets: self.custom_widgets.clone(),
            active_layer: self.active_layer,
            selected_text_index: self.selected_text_index,
            custom_logo_image: self.custom_logo_image.clone(),
            custom_logo_path: self.custom_logo_path.clone(),
            custom_logo_raw: self.custom_logo_raw.clone(),
            custom_chip_image: self.custom_chip_image.clone(),
            custom_chip_path: self.custom_chip_path.clone(),
            custom_chip_raw: self.custom_chip_raw.clone(),
            custom_finish_image: self.custom_finish_image.clone(),
            custom_finish_path: self.custom_finish_path.clone(),
            custom_finish_raw: self.custom_finish_raw.clone(),
            custom_font_bytes: self.custom_font_bytes.clone(),
            custom_font_path: self.custom_font_path.clone(),
            custom_font_name: self.custom_font_name.clone(),
        }
    }

    pub fn apply_snapshot(&mut self, snapshot: StudioSnapshot) {
        self.overlay_options = snapshot.overlay_options;
        self.custom_widgets = snapshot.custom_widgets;
        self.active_layer = snapshot.active_layer;
        self.selected_text_index = snapshot.selected_text_index;
        self.custom_logo_image = snapshot.custom_logo_image;
        self.custom_logo_path = snapshot.custom_logo_path;
        self.custom_logo_raw = snapshot.custom_logo_raw;
        self.custom_chip_image = snapshot.custom_chip_image;
        self.custom_chip_path = snapshot.custom_chip_path;
        self.custom_chip_raw = snapshot.custom_chip_raw;
        self.custom_finish_image = snapshot.custom_finish_image;
        self.custom_finish_path = snapshot.custom_finish_path;
        self.custom_finish_raw = snapshot.custom_finish_raw;
        self.custom_font_bytes = snapshot.custom_font_bytes;
        self.custom_font_path = snapshot.custom_font_path;
        self.custom_font_name = snapshot.custom_font_name;
    }

    pub fn push_undo_checkpoint(&mut self) {
        let current = self.create_snapshot();
        if let Some(last) = self.undo_stack.last() {
            if last.is_visually_equal(&current) {
                return;
            }
        }
        self.undo_stack.push(current);
        if self.undo_stack.len() > MAX_UNDO_STEPS {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
        self.pre_interaction_snapshot = None;
    }

    pub fn record_pre_interaction(&mut self) {
        if self.pre_interaction_snapshot.is_none() {
            self.pre_interaction_snapshot = Some(self.create_snapshot());
        }
    }

    pub fn commit_interaction(&mut self) {
        if let Some(prev) = self.pre_interaction_snapshot.take() {
            let current = self.create_snapshot();
            if !prev.is_visually_equal(&current) {
                self.undo_stack.push(prev);
                if self.undo_stack.len() > MAX_UNDO_STEPS {
                    self.undo_stack.remove(0);
                }
                self.redo_stack.clear();
            }
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some(snapshot) = self.undo_stack.pop() {
            let current = self.create_snapshot();
            self.redo_stack.push(current);
            self.apply_snapshot(snapshot);
            self.pre_interaction_snapshot = None;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(snapshot) = self.redo_stack.pop() {
            let current = self.create_snapshot();
            self.undo_stack.push(current);
            self.apply_snapshot(snapshot);
            self.pre_interaction_snapshot = None;
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn reset_to_defaults(&mut self) {
        self.push_undo_checkpoint();
        self.overlay_options = CardOverlayOptions::default();
        self.custom_logo_image = None;
        self.custom_logo_path = None;
        self.custom_logo_raw = None;
        self.custom_chip_image = None;
        self.custom_chip_path = None;
        self.custom_chip_raw = None;
        self.custom_finish_image = None;
        self.custom_finish_path = None;
        self.custom_finish_raw = None;
        self.custom_font_bytes = None;
        self.custom_font_path = None;
        self.custom_font_name = None;
        self.custom_widgets.clear();
        self.active_layer = ActiveTransformLayer::Background;
        self.selected_text_index = 0;
    }

    pub fn add_text_item(&mut self) -> usize {
        self.push_undo_checkpoint();
        self.overlay_options.details.ensure_items();
        let next_id = self.overlay_options.details.items.iter().map(|i| i.id).max().unwrap_or(0) + 1;
        let count = self.overlay_options.details.items.len();
        let new_y = 350.0 + (count as f32 * 65.0) % 450.0;
        let item = CardTextItem {
            id: next_id,
            label: format!("Dòng chữ #{next_id}"),
            content: "NỘI DUNG MỚI".to_string(),
            x: 140.0,
            y: new_y,
            font_size: 38.0,
            font: CardFontPreset::ModernSans,
            letter_spacing: 1.5,
            is_uppercase: false,
            has_backdrop: false,
            visible: true,
        };
        self.overlay_options.details.items.push(item);
        let idx = self.overlay_options.details.items.len() - 1;
        self.selected_text_index = idx;
        self.active_layer = ActiveTransformLayer::Details;
        idx
    }

    pub fn remove_text_item(&mut self, idx: usize) {
        self.push_undo_checkpoint();
        self.overlay_options.details.ensure_items();
        if idx < self.overlay_options.details.items.len() {
            self.overlay_options.details.items.remove(idx);
            if !self.overlay_options.details.items.is_empty() {
                self.selected_text_index = self.selected_text_index.min(self.overlay_options.details.items.len() - 1);
            } else {
                self.selected_text_index = 0;
            }
        }
    }

    pub fn add_custom_widget(&mut self) -> Result<String, String> {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Select Custom Widget Image (PNG/JPG/SVG/WebP)")
            .add_filter("Supported Images (*.png, *.jpg, *.svg, *.webp, *.bmp)", &["png", "jpg", "jpeg", "webp", "svg", "bmp", "gif", "ico", "tiff"])
            .pick_file()
        else {
            return Err("Cancelled by user".into());
        };

        let next_id = self.custom_widgets.iter().map(|w| w.data.id).max().unwrap_or(0) + 1;
        match CustomWidget::from_image_path(path.clone(), next_id, self.custom_widgets.len()) {
            Ok(widget) => {
                self.push_undo_checkpoint();
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
            self.push_undo_checkpoint();
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
            .set_title("Select Custom Logo Image (PNG/JPG/SVG/WebP)")
            .add_filter("Supported Images (*.png, *.jpg, *.svg, *.webp, *.bmp)", &["png", "jpg", "jpeg", "webp", "svg", "bmp", "gif", "ico", "tiff"])
            .pick_file()
        else {
            return Err("Cancelled by user".into());
        };

        match std::fs::read(&path) {
            Ok(bytes) => match crate::card_studio::workspace::load_image_any_format(&bytes, path.to_str()) {
                Ok(rgba) => {
                    self.push_undo_checkpoint();
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png").to_lowercase();
                    self.custom_logo_image = Some(rgba);
                    self.custom_logo_path = Some(path.clone());
                    self.custom_logo_raw = Some((bytes, ext));
                    self.overlay_options.network = PaymentNetwork::Custom;
                    self.active_layer = ActiveTransformLayer::Logo;
                    Ok(format!("Custom logo loaded: {}", path.display()))
                }
                Err(e) => Err(format!("Failed to parse logo {}: {e:#}", path.display())),
            },
            Err(e) => Err(format!("Failed to read file {}: {e:#}", path.display())),
        }
    }

    pub fn select_custom_chip(&mut self) -> Result<String, String> {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Select Custom EMV Chip Image (PNG/SVG with transparency)")
            .add_filter("Supported Images (*.png, *.svg, *.webp)", &["png", "webp", "svg", "ico"])
            .pick_file()
        else {
            return Err("Cancelled by user".into());
        };

        match std::fs::read(&path) {
            Ok(bytes) => match crate::card_studio::workspace::load_image_any_format(&bytes, path.to_str()) {
                Ok(rgba) => {
                    self.push_undo_checkpoint();
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png").to_lowercase();
                    self.custom_chip_image = Some(rgba);
                    self.custom_chip_path = Some(path.clone());
                    self.custom_chip_raw = Some((bytes, ext));
                    self.overlay_options.show_chip = true;
                    self.active_layer = ActiveTransformLayer::Chip;
                    Ok(format!("Custom chip loaded: {}", path.display()))
                }
                Err(e) => Err(format!("Failed to parse custom chip {}: {e:#}", path.display())),
            },
            Err(e) => Err(format!("Failed to read file {}: {e:#}", path.display())),
        }
    }

    pub fn select_custom_finish(&mut self) -> Result<String, String> {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Select Custom Texture / Foil Image (PNG/JPG/SVG/WebP)")
            .add_filter("Supported Images (*.png, *.jpg, *.svg, *.webp, *.bmp)", &["png", "jpg", "jpeg", "webp", "svg", "bmp", "gif", "ico", "tiff"])
            .pick_file()
        else {
            return Err("Cancelled by user".into());
        };

        match std::fs::read(&path) {
            Ok(bytes) => match crate::card_studio::workspace::load_image_any_format(&bytes, path.to_str()) {
                Ok(rgba) => {
                    self.push_undo_checkpoint();
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png").to_lowercase();
                    self.custom_finish_image = Some(rgba);
                    self.custom_finish_path = Some(path.clone());
                    self.custom_finish_raw = Some((bytes, ext));
                    self.overlay_options.finish = CardFinish::CustomTexture;
                    self.active_layer = ActiveTransformLayer::Finish;
                    Ok(format!("Custom finish texture loaded: {}", path.display()))
                }
                Err(e) => Err(format!("Failed to parse finish texture {}: {e:#}", path.display())),
            },
            Err(e) => Err(format!("Failed to read file {}: {e:#}", path.display())),
        }
    }

    pub fn save_workspace_dialog(
        &self,
        source_image: Option<&DynamicImage>,
        source_raw: Option<&(Vec<u8>, String)>,
    ) -> Result<String, String> {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Save AirCard CMaku Workspace (.wcm)")
            .add_filter("AirCard Workspace (*.wcm)", &["wcm", "WCM"])
            .set_file_name("card_workspace.wcm")
            .save_file()
        else {
            return Err("Cancelled by user".into());
        };

        let mut path = path;
        if path.extension().is_none() || path.extension().unwrap_or_default() != "wcm" {
            path.set_extension("wcm");
        }

        match crate::card_studio::workspace::save_workspace_wcm(self, source_image, source_raw, &path) {
            Ok(()) => Ok(format!("Đã lưu file workspace (.wcm) thành công: {}", path.display())),
            Err(e) => Err(format!("Lỗi khi lưu workspace: {e:#}")),
        }
    }

    pub fn load_workspace_dialog(&mut self) -> Result<(String, Option<DynamicImage>, Option<(Vec<u8>, String)>), String> {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Open AirCard CMaku Workspace (.wcm)")
            .add_filter("AirCard Workspace (*.wcm)", &["wcm", "WCM"])
            .pick_file()
        else {
            return Err("Cancelled by user".into());
        };

        match crate::card_studio::workspace::load_workspace_wcm(self, &path) {
            Ok((source_img, source_raw)) => Ok((format!("Đã mở workspace thành công: {}", path.display()), source_img, source_raw)),
            Err(e) => Err(format!("Lỗi khi nạp workspace: {e:#}")),
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
                self.push_undo_checkpoint();
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

                // Auto-apply custom font to all text items so the user immediately sees it
                for item in &mut self.overlay_options.details.items {
                    item.font = CardFontPreset::Custom;
                }

                Ok(format!("Đã tải font: {file_name}"))
            }
            Err(e) => Err(format!("Failed to read font file {}: {e:#}", path.display())),
        }
    }

    pub fn apply_custom_font_to_all_items(&mut self) {
        for item in &mut self.overlay_options.details.items {
            item.font = CardFontPreset::Custom;
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
        for item in &mut self.overlay_options.details.items {
            if item.font == CardFontPreset::Custom {
                item.font = CardFontPreset::ModernSans;
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_studio_undo_redo() {
        let mut state = CardStudioState::new();
        assert!(!state.can_undo());
        assert!(!state.can_redo());

        // Initial preset is CustomImage
        assert_eq!(state.overlay_options.bg_preset, CardBackgroundPreset::CustomImage);

        // Step 1: change bg_preset to MatteBlack
        state.push_undo_checkpoint();
        state.overlay_options.bg_preset = CardBackgroundPreset::MatteBlack;
        assert!(state.can_undo());
        assert!(!state.can_redo());

        // Step 2: change finish to MetallicSheen
        state.push_undo_checkpoint();
        state.overlay_options.finish = CardFinish::MetallicSheen;

        // Undo Step 2 -> finish should be Standard, bg should still be MatteBlack
        assert!(state.undo());
        assert_eq!(state.overlay_options.finish, CardFinish::Standard);
        assert_eq!(state.overlay_options.bg_preset, CardBackgroundPreset::MatteBlack);
        assert!(state.can_redo());

        // Undo Step 1 -> bg should be CustomImage
        assert!(state.undo());
        assert_eq!(state.overlay_options.bg_preset, CardBackgroundPreset::CustomImage);

        // Redo Step 1 -> bg should be MatteBlack
        assert!(state.redo());
        assert_eq!(state.overlay_options.bg_preset, CardBackgroundPreset::MatteBlack);

        // Redo Step 2 -> finish should be MetallicSheen
        assert!(state.redo());
        assert_eq!(state.overlay_options.finish, CardFinish::MetallicSheen);
        assert!(!state.can_redo());

        // Test Reset to defaults is undoable
        state.reset_to_defaults();
        assert_eq!(state.overlay_options.finish, CardFinish::Standard);
        assert!(state.can_undo());

        // Undo reset -> finish restored to MetallicSheen
        assert!(state.undo());
        assert_eq!(state.overlay_options.finish, CardFinish::MetallicSheen);
    }
}

