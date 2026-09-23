use std::path::PathBuf;
use crate::card_studio::types::{CARD_WIDTH, CARD_HEIGHT, LayerAdjustments};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CustomWidgetData {
    pub id: u64,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub base_w: f32,
    pub base_h: f32,
    pub adjustments: LayerAdjustments,
    pub visible: bool,
    pub has_shadow: bool,
}

impl Default for CustomWidgetData {
    fn default() -> Self {
        Self {
            id: 0,
            name: "Widget".to_string(),
            x: CARD_WIDTH as f32 * 0.5,
            y: CARD_HEIGHT as f32 * 0.5,
            scale: 1.0,
            base_w: 160.0,
            base_h: 160.0,
            adjustments: LayerAdjustments::default(),
            visible: true,
            has_shadow: true,
        }
    }
}

pub struct CustomWidget {
    pub data: CustomWidgetData,
    pub image: image::RgbaImage,
    #[allow(dead_code)]
    pub path: Option<PathBuf>,
}

impl CustomWidget {
    pub fn from_image_path(path: PathBuf, next_id: u64, existing_count: usize) -> Result<Self, anyhow::Error> {
        let img = image::open(&path)?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        let file_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Widget")
            .to_string();

        let aspect = if h > 0 { w as f32 / h as f32 } else { 1.0 };
        let (base_w, base_h) = if aspect >= 1.0 {
            (200.0, (200.0 / aspect).max(30.0))
        } else {
            ((200.0 * aspect).max(30.0), 200.0)
        };

        let offset = (existing_count as f32 * 35.0) % 250.0;
        let data = CustomWidgetData {
            id: next_id,
            name: file_name,
            x: (CARD_WIDTH as f32 * 0.5) + offset - 100.0,
            y: (CARD_HEIGHT as f32 * 0.5) + offset - 80.0,
            scale: 1.0,
            base_w,
            base_h,
            adjustments: LayerAdjustments::default(),
            visible: true,
            has_shadow: true,
        };

        Ok(Self {
            data,
            image: rgba,
            path: Some(path),
        })
    }
}
