use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::thread;

use eframe::egui;
use image::GenericImageView;

use crate::apple;
use crate::device::{DeviceInfo, list_connected_devices};
use crate::flasher::{flash_passcode_theme, flash_wallet_skin};
use crate::card_studio::{
    CardBackgroundPreset, CardStudioState, ImageTransform,
    CARD_HEIGHT, CARD_WIDTH,
};
use crate::image_skin::PreparedSkin;
use crate::passthm::{PasscodeTheme, parse_passthm_file};
use crate::scanner::{SavedCard, load_saved_cards, scan_syslog_for_cards};

#[derive(PartialEq, Eq)]
enum AppTab {
    Wallet,
    Passcode,
    Help,
    Sources,
}

enum BackgroundTaskMessage {
    Progress { step: usize, total: usize, message: String },
    Log(String),
    CardFound { hash: String, name: String },
    Done(Result<String, String>),
}

#[cfg(windows)]
fn current_timestamp() -> String {
    #[repr(C)]
    struct SystemTime {
        w_year: u16,
        w_month: u16,
        w_day_of_week: u16,
        w_day: u16,
        w_hour: u16,
        w_minute: u16,
        w_second: u16,
        w_milliseconds: u16,
    }
    unsafe extern "system" {
        fn GetLocalTime(lpSystemTime: *mut SystemTime);
    }
    let mut st = std::mem::MaybeUninit::<SystemTime>::uninit();
    unsafe {
        GetLocalTime(st.as_mut_ptr());
        let st = st.assume_init();
        format!(
            "{:02}:{:02}:{:02}.{:03}",
            st.w_hour, st.w_minute, st.w_second, st.w_milliseconds
        )
    }
}

#[cfg(not(windows))]
fn current_timestamp() -> String {
    "00:00:00.000".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiLanguage {
    #[default]
    English,
    Vietnamese,
}

#[allow(unused_imports)]
pub use crate::card_studio::{ActiveTransformLayer, CustomWidget};

pub struct AirCardApp {
    current_tab: AppTab,
    apple_status: String,
    apple_ready: bool,

    // Device management
    devices: Vec<DeviceInfo>,
    selected_udid: Option<String>,

    // Wallet tab
    card_hash: String,
    saved_cards: Vec<SavedCard>,
    source_path: Option<PathBuf>,
    raw_image: Option<image::DynamicImage>,
    studio: CardStudioState,
    ui_language: UiLanguage,
    skin: Option<PreparedSkin>,
    skin_texture: Option<egui::TextureHandle>,
    scanning_syslog: bool,
    scan_stop_flag: Option<Arc<AtomicBool>>,

    // Passcode tab
    theme_path: Option<PathBuf>,
    loaded_theme: Option<PasscodeTheme>,
    forced_telephony_ver: String,
    keypad_language: String,
    passcode_bold: bool,
    keypad_textures: Vec<(String, egui::TextureHandle)>,

    // Worker thread & progress
    is_busy: bool,
    progress_step: usize,
    progress_total: usize,
    progress_msg: String,
    status_msg: String,
    task_rx: Option<Receiver<BackgroundTaskMessage>>,
    logs: Vec<String>,
    show_logs_window: bool,
}

impl AirCardApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        setup_custom_theme(&cc.egui_ctx);

        let (apple_ready, apple_status) = match apple::verify_support() {
            Ok(msg) => (true, msg),
            Err(err) => (false, err.to_string()),
        };

        let mut app = Self {
            current_tab: AppTab::Wallet,
            apple_status,
            apple_ready,

            devices: Vec::new(),
            selected_udid: None,

            card_hash: String::new(),
            saved_cards: load_saved_cards(),
            source_path: None,
            raw_image: None,
            studio: CardStudioState::default(),
            ui_language: UiLanguage::Vietnamese,
            skin: None,
            skin_texture: None,
            scanning_syslog: false,
            scan_stop_flag: None,

            theme_path: None,
            loaded_theme: None,
            forced_telephony_ver: "Auto (TelephonyUI-10)".to_string(),
            keypad_language: "English".to_string(),
            passcode_bold: false,
            keypad_textures: Vec::new(),

            is_busy: false,
            progress_step: 0,
            progress_total: 0,
            progress_msg: String::new(),
            status_msg: "Ready. Connect iPhone via USB and unlock it.".to_string(),
            task_rx: None,
            logs: Vec::new(),
            show_logs_window: false,
        };

        app.add_log(format!("AirCard-CMaku Windows v{} initialized", env!("CARGO_PKG_VERSION")));
        app.add_log(format!("Apple Support Runtime: {}", if app.apple_ready { "Loaded and operational" } else { "Not found (iTunes required)" }));
        app.add_log(format!("Loaded {} saved card(s) from database", app.saved_cards.len()));

        if app.apple_ready {
            app.refresh_devices();
        }

        app
    }

    fn add_log(&mut self, text: impl AsRef<str>) {
        let ts = current_timestamp();
        self.logs.push(format!("[{}] {}", ts, text.as_ref()));
        if self.logs.len() > 1000 {
            self.logs.remove(0);
        }
    }

    fn refresh_devices(&mut self) {
        self.add_log("Scanning for connected iOS devices via usbmuxd...");
        match list_connected_devices() {
            Ok(devs) => {
                self.devices = devs;
                if self.selected_udid.is_none() && !self.devices.is_empty() {
                    self.selected_udid = Some(self.devices[0].udid.clone());
                }
                if self.devices.is_empty() {
                    self.add_log("No devices detected. Please plug in your iPhone and tap 'Trust this Computer'.");
                    self.status_msg = "No devices connected via USB.".to_string();
                } else {
                    let dev_logs: Vec<String> = self.devices.iter().enumerate().map(|(i, d)| {
                        format!("Device #{}: {} - UDID: {}", i + 1, d, d.udid)
                    }).collect();
                    for line in dev_logs {
                        self.add_log(line);
                    }
                    self.status_msg = format!("Found {} connected device(s)", self.devices.len());
                }
            }
            Err(err) => {
                self.add_log(format!("Device scan error: {}", err));
                self.status_msg = format!("Could not enumerate devices: {}", err);
            }
        }
    }

    fn select_skin(&mut self, ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
            .pick_file()
        else {
            return;
        };

        self.add_log(format!("Opening skin image: {}", path.display()));
        match image::open(&path) {
            Ok(img) => {
                self.raw_image = Some(img.clone());
                self.source_path = Some(path.clone());
                self.studio.overlay_options.bg_preset = CardBackgroundPreset::CustomImage;
                self.studio.overlay_options.transform = ImageTransform::default();
                self.recompute_skin(ctx);
                if let Some((w, h)) = self.skin.as_ref().map(|s| (s.source_width, s.source_height)) {
                    self.add_log(format!(
                        "Skin processed: source {}x{} resampled to 1536x969",
                        w, h,
                    ));
                    self.status_msg = format!(
                        "Prepared {} ({}x{} -> 1536x969)",
                        path.file_name().and_then(|n| n.to_str()).unwrap_or("image"),
                        w, h,
                    );
                }
            }
            Err(error) => {
                self.add_log(format!("Failed to open image {}: {error:#}", path.display()));
                self.status_msg = format!("Could not open image: {error:#}");
            }
        }
    }

    fn recompute_skin(&mut self, ctx: &egui::Context) {
        if self.raw_image.is_none() && self.studio.overlay_options.bg_preset == CardBackgroundPreset::CustomImage {
            return;
        }
        match self.studio.render_preview_canvas(self.raw_image.as_ref()) {
            Ok((rgba, preview)) => {
                self.skin_texture = Some(ctx.load_texture(
                    "card-skin-preview",
                    preview.clone(),
                    egui::TextureOptions::LINEAR,
                ));
                let (source_width, source_height) = if let Some(ref img) = self.raw_image {
                    img.dimensions()
                } else {
                    (CARD_WIDTH, CARD_HEIGHT)
                };
                self.skin = Some(PreparedSkin {
                    png: Vec::new(),
                    pdf: Vec::new(),
                    preview,
                    source_width,
                    source_height,
                    rgba,
                });
            }
            Err(err) => {
                self.add_log(format!("Failed to re-render card preview: {err:#}"));
            }
        }
    }

    fn ensure_skin_encoded(&mut self) -> Result<(), String> {
        let Some(skin) = self.skin.as_mut() else {
            return Err("No skin prepared".to_string());
        };
        if skin.png.is_empty() || skin.pdf.is_empty() {
            let (png, pdf) = PreparedSkin::encode_skin_png_and_pdf(&skin.rgba)
                .map_err(|e| format!("{e:#}"))?;
            skin.png = png;
            skin.pdf = pdf;
        }
        Ok(())
    }

    fn save_prepared_png(&mut self) {
        if let Err(err) = self.ensure_skin_encoded() {
            self.add_log(format!("Export failed: {err}"));
            self.status_msg = format!("Could not encode PNG: {err}");
            return;
        }
        let Some(skin) = &self.skin else {
            return;
        };
        let Some(path) = rfd::FileDialog::new()
            .set_file_name("aircard-skin.png")
            .save_file()
        else {
            return;
        };
        match std::fs::write(&path, &skin.png) {
            Ok(()) => {
                self.add_log(format!("Exported prepared card skin PNG: {}", path.display()));
                self.status_msg = format!("Saved prepared PNG: {}", path.display());
            }
            Err(err) => {
                self.add_log(format!("Failed to save PNG: {err}"));
                self.status_msg = format!("Could not save PNG: {err}");
            }
        }
    }

    fn toggle_syslog_scan(&mut self) {
        if self.scanning_syslog {
            if let Some(flag) = self.scan_stop_flag.take() {
                flag.store(true, Ordering::Relaxed);
            }
            self.scanning_syslog = false;
            self.add_log("Syslog scanning stopped by user.");
            self.status_msg = "Syslog scanning stopped.".to_string();
            return;
        }

        let stop_flag = Arc::new(AtomicBool::new(false));
        self.scan_stop_flag = Some(Arc::clone(&stop_flag));
        self.scanning_syslog = true;
        self.add_log("Initiating syslog monitor session...");
        self.status_msg = "Scanning syslog... Open Wallet or tap your card on iPhone.".to_string();

        let (tx, rx) = channel();
        self.task_rx = Some(rx);
        let udid = self.selected_udid.clone();

        thread::spawn(move || {
            let tx_card = tx.clone();
            let tx_log = tx.clone();
            let res = scan_syslog_for_cards(
                udid.as_deref(),
                stop_flag,
                move |hash, name| {
                    let _ = tx_card.send(BackgroundTaskMessage::CardFound { hash, name });
                },
                move |msg| {
                    let _ = tx_log.send(BackgroundTaskMessage::Log(msg));
                },
            );
            match res {
                Ok(()) => {
                    let _ = tx.send(BackgroundTaskMessage::Done(Ok("Syslog scan finished".into())));
                }
                Err(e) => {
                    let _ = tx.send(BackgroundTaskMessage::Done(Err(e.to_string())));
                }
            }
        });
    }

    fn flash_card(&mut self) {
        let Some(udid) = self.selected_udid.clone() else {
            self.add_log("Flash failed: No connected iPhone selected.");
            self.status_msg = "Please select a connected iPhone.".to_string();
            return;
        };
        let hash = self.card_hash.trim().to_string();
        if hash.is_empty() {
            self.add_log("Flash failed: Target card hash is empty.");
            self.status_msg = "Please enter or scan a target card hash.".to_string();
            return;
        }
        if let Err(err) = self.ensure_skin_encoded() {
            self.add_log(format!("Flash failed: Could not encode skin assets: {err}"));
            self.status_msg = format!("Could not encode skin assets: {err}");
            return;
        }
        let Some(skin) = self.skin.as_ref() else {
            self.add_log("Flash failed: No skin image prepared.");
            self.status_msg = "Please choose a card skin image first.".to_string();
            return;
        };

        let png_bytes = skin.png.clone();
        let pdf_bytes = skin.pdf.clone();
        if let Some(ref flag) = self.scan_stop_flag {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        self.scanning_syslog = false;
        self.is_busy = true;
        self.progress_step = 0;
        self.progress_total = 3;
        self.progress_msg = "Initiating card flash...".to_string();
        self.status_msg = "Writing card skin to iPhone...".to_string();
        self.add_log(format!("Starting card skin flash for hash: {} (UDID: {})", hash, udid));

        let (tx, rx) = channel();
        self.task_rx = Some(rx);

        thread::spawn(move || {
            let tx_progress = tx.clone();
            let tx_log = tx.clone();
            let res = flash_wallet_skin(
                &udid,
                &hash,
                &png_bytes,
                &pdf_bytes,
                move |step, total, msg| {
                    let _ = tx_progress.send(BackgroundTaskMessage::Progress {
                        step,
                        total,
                        message: msg.to_string(),
                    });
                },
                move |msg| {
                    let _ = tx_log.send(BackgroundTaskMessage::Log(msg.to_string()));
                },
            );

            match res {
                Ok(()) => {
                    let _ = tx.send(BackgroundTaskMessage::Done(Ok(
                        "Card skin successfully flashed! Force quit Wallet on iPhone and reopen it.".into(),
                    )));
                }
                Err(e) => {
                    let _ = tx.send(BackgroundTaskMessage::Done(Err(format!("{:#}", e))));
                }
            }
        });
    }

    fn select_theme_file(&mut self, ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Passcode Theme", &["passthm", "passtheme", "zip"])
            .pick_file()
        else {
            return;
        };

        self.load_theme_from_path(ctx, &path);
    }

    fn load_theme_from_path(&mut self, ctx: &egui::Context, path: &Path) {
        self.add_log(format!("Opening passcode theme package: {}", path.display()));
        let target_ver = match self.forced_telephony_ver.as_str() {
            "TelephonyUI-10" => Some("TelephonyUI-10"),
            "TelephonyUI-9" => Some("TelephonyUI-9"),
            "TelephonyUI-8" => Some("TelephonyUI-8"),
            _ => Some("TelephonyUI-10"),
        };

        match parse_passthm_file(path, target_ver, &self.keypad_language, self.passcode_bold) {
            Ok(theme) => {
                self.keypad_textures.clear();
                for (digit, bytes) in &theme.key_previews {
                    if let Ok(img) = image::load_from_memory(bytes) {
                        let rgba = img.to_rgba8();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            [rgba.width() as usize, rgba.height() as usize],
                            &rgba,
                        );
                        let tex = ctx.load_texture(
                            format!("keypad-{}", digit),
                            color_image,
                            egui::TextureOptions::LINEAR,
                        );
                        self.keypad_textures.push((digit.clone(), tex));
                    }
                }
                self.keypad_textures.sort_by(|a, b| a.0.cmp(&b.0));

                self.add_log(format!(
                    "Passcode theme loaded: '{}' (telephony: {}, lang: {}, bold: {}, {} assets)",
                    theme.name,
                    theme.detected_version,
                    self.keypad_language,
                    self.passcode_bold,
                    theme.items.len()
                ));
                self.status_msg = format!(
                    "Loaded '{}' with {} assets (target: {}, lang: {}, bold: {})",
                    theme.name,
                    theme.items.len(),
                    theme.detected_version,
                    self.keypad_language,
                    if self.passcode_bold { "ON" } else { "OFF" }
                );
                self.theme_path = Some(path.to_path_buf());
                self.loaded_theme = Some(theme);
            }
            Err(err) => {
                self.add_log(format!("Failed to parse theme: {err:#}"));
                self.status_msg = format!("Failed to parse theme: {err:#}");
            }
        }
    }

    fn flash_theme(&mut self) {
        let Some(udid) = self.selected_udid.clone() else {
            self.add_log("Theme flash failed: No connected iPhone selected.");
            self.status_msg = "Please select a connected iPhone.".to_string();
            return;
        };
        let Some(theme) = self.loaded_theme.as_ref() else {
            self.add_log("Theme flash failed: No .passthm theme loaded.");
            self.status_msg = "Please select a .passthm theme file first.".to_string();
            return;
        };

        let items = theme.items.clone();
        self.is_busy = true;
        self.progress_step = 0;
        self.progress_total = items.len();
        self.progress_msg = "Starting passcode theme flash...".to_string();
        self.status_msg = "Writing passcode theme buttons...".to_string();
        self.add_log(format!("Flashing passcode theme '{}' ({} button assets) to device {}", theme.name, items.len(), udid));

        let (tx, rx) = channel();
        self.task_rx = Some(rx);

        thread::spawn(move || {
            let tx_progress = tx.clone();
            let tx_log = tx.clone();
            let res = flash_passcode_theme(
                &udid,
                &items,
                move |step, total, msg| {
                    let _ = tx_progress.send(BackgroundTaskMessage::Progress {
                        step,
                        total,
                        message: msg.to_string(),
                    });
                },
                move |msg| {
                    let _ = tx_log.send(BackgroundTaskMessage::Log(msg.to_string()));
                },
            );

            match res {
                Ok(()) => {
                    let _ = tx.send(BackgroundTaskMessage::Done(Ok(
                        "Passcode theme applied! Lock your iPhone to view the new keypad.".into(),
                    )));
                }
                Err(e) => {
                    let _ = tx.send(BackgroundTaskMessage::Done(Err(format!("{:#}", e))));
                }
            }
        });
    }

    fn handle_messages(&mut self) {
        let mut messages = Vec::new();
        if let Some(ref rx) = self.task_rx {
            while let Ok(msg) = rx.try_recv() {
                messages.push(msg);
            }
        }

        let mut finished = false;
        for msg in messages {
            match msg {
                BackgroundTaskMessage::Progress { step, total, message } => {
                    self.progress_step = step;
                    self.progress_total = total;
                    self.progress_msg = message.clone();
                    let msg_str = format!("[{}/{}] {}", step, total, message);
                    self.add_log(&msg_str);
                    self.status_msg = msg_str;
                }
                BackgroundTaskMessage::Log(log_line) => {
                    self.add_log(log_line);
                }
                BackgroundTaskMessage::CardFound { hash, name } => {
                    self.card_hash = hash.clone();
                    self.saved_cards = load_saved_cards();
                    let msg_str = format!("Card captured: {} ({})", name, hash);
                    self.add_log(&msg_str);
                    self.status_msg = msg_str;
                }
                BackgroundTaskMessage::Done(res) => {
                    self.is_busy = false;
                    self.scanning_syslog = false;
                    finished = true;
                    match res {
                        Ok(ok_msg) => {
                            self.add_log(format!("Operation completed: {}", ok_msg));
                            self.status_msg = ok_msg;
                        }
                        Err(err_msg) => {
                            self.add_log(format!("Operation failed: {}", err_msg));
                            self.status_msg = format!("Error: {}", err_msg);
                        }
                    }
                }
            }
        }
        if finished {
            self.task_rx = None;
        }
    }
}

pub mod md3 {
    use eframe::egui::Color32;

    // M3 Dark scheme
    pub const SURFACE: Color32 = Color32::from_rgb(18, 18, 20);
    pub const SURFACE_CONTAINER_LOW: Color32 = Color32::from_rgb(26, 25, 29);
    pub const SURFACE_CONTAINER: Color32 = Color32::from_rgb(33, 31, 36);
    pub const SURFACE_CONTAINER_HIGH: Color32 = Color32::from_rgb(43, 41, 48);
    pub const SURFACE_CONTAINER_HIGHEST: Color32 = Color32::from_rgb(54, 52, 59);
    pub const ON_SURFACE: Color32 = Color32::from_rgb(230, 225, 229);
    pub const ON_SURFACE_VARIANT: Color32 = Color32::from_rgb(196, 199, 197);
    pub const OUTLINE: Color32 = Color32::from_rgb(147, 143, 153);
    pub const OUTLINE_VARIANT: Color32 = Color32::from_rgb(73, 69, 79);

    // Primary
    pub const PRIMARY: Color32 = Color32::from_rgb(208, 188, 255);
    pub const ON_PRIMARY: Color32 = Color32::from_rgb(56, 30, 114);
    pub const PRIMARY_CONTAINER: Color32 = Color32::from_rgb(79, 55, 139);
    pub const ON_PRIMARY_CONTAINER: Color32 = Color32::from_rgb(234, 221, 255);

    // Secondary
    pub const SECONDARY_CONTAINER: Color32 = Color32::from_rgb(74, 68, 88);
    pub const ON_SECONDARY_CONTAINER: Color32 = Color32::from_rgb(232, 222, 248);

    // Tertiary
    pub const TERTIARY_CONTAINER: Color32 = Color32::from_rgb(99, 59, 72);
    pub const ON_TERTIARY_CONTAINER: Color32 = Color32::from_rgb(255, 216, 228);

    // Error
    pub const ERROR: Color32 = Color32::from_rgb(242, 184, 181);
    pub const ERROR_CONTAINER: Color32 = Color32::from_rgb(140, 29, 24);

    // Extra
    pub const SUCCESS: Color32 = Color32::from_rgb(120, 220, 120);
}

fn draw_status_dot(ui: &mut egui::Ui, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 4.0, color);
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    #[cfg(windows)]
    {
        // 1. Primary UI font with full Vietnamese support: Segoe UI
        if let Ok(bytes) = std::fs::read("C:/Windows/Fonts/segoeui.ttf") {
            fonts.font_data.insert(
                "segoeui".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );

            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "segoeui".to_owned());

            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("segoeui".to_owned());
        }

        // 2. Fallback UI font: Arial
        if let Ok(bytes) = std::fs::read("C:/Windows/Fonts/arial.ttf") {
            fonts.font_data.insert(
                "arial".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );

            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("arial".to_owned());
        }

        // 3. Fallback for symbols & emojis: Segoe UI Emoji
        if let Ok(bytes) = std::fs::read("C:/Windows/Fonts/seguiemj.ttf") {
            fonts.font_data.insert(
                "seguiemj".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );

            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("seguiemj".to_owned());
        }
    }

    ctx.set_fonts(fonts);
}

fn setup_custom_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    visuals.panel_fill = md3::SURFACE;
    visuals.window_fill = md3::SURFACE;
    visuals.extreme_bg_color = md3::SURFACE_CONTAINER;
    visuals.faint_bg_color = md3::SURFACE_CONTAINER;

    visuals.window_corner_radius = 16.into();
    visuals.menu_corner_radius = 12.into();

    visuals.widgets.noninteractive.corner_radius = 12.into();
    visuals.widgets.noninteractive.bg_fill = md3::SURFACE_CONTAINER;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, md3::ON_SURFACE);

    visuals.widgets.inactive.bg_fill = md3::SURFACE_CONTAINER_HIGH;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, md3::ON_SURFACE_VARIANT);
    visuals.widgets.inactive.corner_radius = 12.into();

    visuals.widgets.hovered.bg_fill = md3::SURFACE_CONTAINER_HIGHEST;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, md3::ON_SURFACE);
    visuals.widgets.hovered.corner_radius = 12.into();

    visuals.widgets.active.bg_fill = md3::PRIMARY_CONTAINER;
    visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, md3::ON_PRIMARY_CONTAINER);
    visuals.widgets.active.corner_radius = 12.into();

    visuals.widgets.open.bg_fill = md3::SURFACE_CONTAINER_HIGHEST;
    visuals.widgets.open.corner_radius = 12.into();
    visuals.widgets.open.bg_stroke = egui::Stroke::NONE;

    visuals.selection.bg_fill = md3::PRIMARY_CONTAINER;
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, md3::PRIMARY);

    ctx.set_visuals(visuals);

    ctx.style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(16.0, 8.0);
    });
}

fn m3_card<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(md3::SURFACE_CONTAINER)
        .corner_radius(16)
        .inner_margin(egui::Margin::same(20))
        .show(ui, |ui| ui.vertical(add_contents).inner)
        .inner
}

fn m3_button_filled(ui: &mut egui::Ui, label: &str) -> bool {
    let btn = egui::Button::new(
        egui::RichText::new(label).size(13.0).color(md3::ON_PRIMARY),
    )
    .fill(md3::PRIMARY)
    .corner_radius(20)
    .stroke(egui::Stroke::NONE);
    ui.add(btn).clicked()
}

fn m3_button_tonal(ui: &mut egui::Ui, label: &str) -> bool {
    let btn = egui::Button::new(
        egui::RichText::new(label).size(13.0).color(md3::ON_SECONDARY_CONTAINER),
    )
    .fill(md3::SECONDARY_CONTAINER)
    .corner_radius(20)
    .stroke(egui::Stroke::NONE);
    ui.add(btn).clicked()
}

fn m3_button_outlined(ui: &mut egui::Ui, label: &str) -> bool {
    let btn = egui::Button::new(
        egui::RichText::new(label).size(13.0).color(md3::PRIMARY),
    )
    .fill(egui::Color32::TRANSPARENT)
    .corner_radius(20)
    .stroke(egui::Stroke::new(1.0_f32, md3::OUTLINE));
    ui.add(btn).clicked()
}

fn m3_tab(ui: &mut egui::Ui, current: &mut AppTab, target: AppTab, label: &str) {
    let selected = *current == target;
    let (bg, fg) = if selected {
        (md3::SECONDARY_CONTAINER, md3::ON_SECONDARY_CONTAINER)
    } else {
        (egui::Color32::TRANSPARENT, md3::ON_SURFACE_VARIANT)
    };
    let btn = egui::Button::new(
        egui::RichText::new(label).size(12.5).color(fg),
    )
    .fill(bg)
    .corner_radius(20)
    .stroke(egui::Stroke::NONE);
    if ui.add(btn).clicked() {
        *current = target;
    }
}

impl eframe::App for AirCardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_messages();

        if self.is_busy || self.scanning_syslog {
            ctx.request_repaint();
        }

        // Top bar
        egui::TopBottomPanel::top("header")
            .frame(
                egui::Frame::new()
                    .fill(md3::SURFACE)
                    .inner_margin(egui::Margin::symmetric(20, 10)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("AirCard-CMaku")
                            .strong()
                            .size(18.0)
                            .color(md3::ON_SURFACE),
                    );
                    ui.label(
                        egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                            .size(11.0)
                            .color(md3::ON_SURFACE_VARIANT),
                    );

                    ui.add_space(20.0);
                    let (tab_wallet, tab_passcode, tab_help, tab_sources) = match self.ui_language {
                        UiLanguage::English => ("Apple Pay", "Passcodes", "Manage", "Sources"),
                        UiLanguage::Vietnamese => ("Thẻ Apple Pay", "Mật mã", "Quản lý", "Nguồn & Bản quyền"),
                    };
                    m3_tab(ui, &mut self.current_tab, AppTab::Wallet, tab_wallet);
                    m3_tab(ui, &mut self.current_tab, AppTab::Passcode, tab_passcode);
                    m3_tab(ui, &mut self.current_tab, AppTab::Help, tab_help);
                    m3_tab(ui, &mut self.current_tab, AppTab::Sources, tab_sources);

                    ui.add_space(8.0);
                    let (lang_btn_text, lang_hover) = match self.ui_language {
                        UiLanguage::English => ("🌐 Tiếng Việt", "Chuyển giao diện sang Tiếng Việt"),
                        UiLanguage::Vietnamese => ("🌐 English", "Switch interface to English"),
                    };
                    let lang_btn = egui::Button::new(
                        egui::RichText::new(lang_btn_text).size(11.0).color(md3::PRIMARY).strong(),
                    )
                    .fill(md3::SURFACE_CONTAINER_HIGH)
                    .corner_radius(16)
                    .stroke(egui::Stroke::new(1.0_f32, md3::PRIMARY));
                    if ui.add(lang_btn).on_hover_text(lang_hover).clicked() {
                        self.ui_language = match self.ui_language {
                            UiLanguage::English => UiLanguage::Vietnamese,
                            UiLanguage::Vietnamese => UiLanguage::English,
                        };
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let is_vi = self.ui_language == UiLanguage::Vietnamese;
                        if m3_button_outlined(ui, if is_vi { "Làm mới" } else { "Refresh" }) {
                            self.refresh_devices();
                        }
                        ui.add_space(4.0);
                        let has_device = !self.devices.is_empty();
                        draw_status_dot(ui, if has_device { md3::SUCCESS } else { md3::ERROR });
                        if has_device {
                            let name = self.devices.iter()
                                .find(|d| Some(&d.udid) == self.selected_udid.as_ref())
                                .map(|d| d.name.clone())
                                .unwrap_or_else(|| "iPhone".into());
                            ui.label(egui::RichText::new(name).size(12.0).color(md3::ON_SURFACE))
                                .on_hover_text(&self.apple_status);
                        } else {
                            ui.label(egui::RichText::new(if is_vi { "Chưa kết nối" } else { "No device" }).size(12.0).color(md3::ON_SURFACE_VARIANT))
                                .on_hover_text(&self.apple_status);
                        }
                    });
                });
            });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar")
            .frame(
                egui::Frame::new()
                    .fill(md3::SURFACE)
                    .inner_margin(egui::Margin::symmetric(20, 8)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let dot_col = if self.is_busy || self.scanning_syslog {
                        md3::PRIMARY
                    } else if self.status_msg.starts_with("Error") || self.status_msg.starts_with("Failed") {
                        md3::ERROR
                    } else {
                        md3::SUCCESS
                    };
                    draw_status_dot(ui, dot_col);
                    if self.is_busy || self.scanning_syslog { ui.spinner(); }
                    let is_vi = self.ui_language == UiLanguage::Vietnamese;
                    let display_status = if is_vi {
                        if self.status_msg.starts_with("Ready") {
                            "Sẵn sàng. Kết nối iPhone qua cáp USB và mở khóa màn hình."
                        } else if self.status_msg.contains("No devices connected") {
                            "Chưa có thiết bị kết nối qua USB."
                        } else {
                            &self.status_msg
                        }
                    } else {
                        &self.status_msg
                    };
                    ui.label(egui::RichText::new(display_status).size(11.5).color(md3::ON_SURFACE_VARIANT));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let btn_text = if self.show_logs_window { "Logs [x]" } else { "Logs" };
                        let btn = egui::Button::new(
                            egui::RichText::new(btn_text).size(11.0).color(
                                if self.show_logs_window { md3::ON_PRIMARY_CONTAINER } else { md3::ON_SURFACE_VARIANT }
                            ),
                        )
                        .fill(if self.show_logs_window { md3::PRIMARY_CONTAINER } else { egui::Color32::TRANSPARENT })
                        .corner_radius(20)
                        .stroke(egui::Stroke::new(1.0_f32, if self.show_logs_window { md3::PRIMARY } else { md3::OUTLINE_VARIANT }));
                        if ui.add(btn).clicked() {
                            self.show_logs_window = !self.show_logs_window;
                        }
                    });
                });
            });

        // Central - same SURFACE fill as header/status for flat look
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(md3::SURFACE)
                    .inner_margin(egui::Margin::same(16)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    match self.current_tab {
                        AppTab::Wallet => self.show_wallet_tab(ctx, ui),
                        AppTab::Passcode => self.show_passcode_tab(ctx, ui),
                        AppTab::Help => self.show_help_tab(ui),
                        AppTab::Sources => self.show_sources_tab(ui),
                    }
                });
            });

        let mut show_logs = self.show_logs_window;
        let mut file_saved_msg: Option<String> = None;
        if show_logs {
            egui::Window::new("Logs")
                .open(&mut show_logs)
                .default_size([540.0, 300.0])
                .min_size([360.0, 180.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if m3_button_tonal(ui, "Copy Logs") {
                            ctx.copy_text(self.logs.join("\n"));
                        }
                        if m3_button_outlined(ui, "Save to File...") {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_file_name("aircard-diagnostics.log")
                                .add_filter("Log files", &["log", "txt"])
                                .save_file()
                            {
                                let content = self.logs.join("\r\n");
                                let _ = std::fs::write(&path, content);
                                file_saved_msg = Some(format!("Saved log file to {}", path.display()));
                            }
                        }
                        if m3_button_outlined(ui, "Clear") {
                            self.logs.clear();
                        }
                        ui.label(
                            egui::RichText::new(format!("{} entries", self.logs.len()))
                                .size(11.0)
                                .color(md3::ON_SURFACE_VARIANT),
                        );
                    });
                    ui.add_space(8.0);
                    egui::Frame::new()
                        .fill(md3::SURFACE_CONTAINER_HIGH)
                        .corner_radius(12)
                        .inner_margin(egui::Margin::same(10))
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .stick_to_bottom(true)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    if self.logs.is_empty() {
                                        ui.label(egui::RichText::new("No events logged yet.").size(11.0).color(md3::ON_SURFACE_VARIANT));
                                    } else {
                                        for line in &self.logs {
                                            ui.label(
                                                egui::RichText::new(line)
                                                    .size(10.5)
                                                    .monospace()
                                                    .color(md3::ON_SURFACE),
                                            );
                                        }
                                    }
                                });
                        });
                });
            self.show_logs_window = show_logs;
            if let Some(msg) = file_saved_msg {
                self.add_log(msg);
            }
        }
    }
}

impl AirCardApp {
    fn show_wallet_tab(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let is_vi = self.ui_language == UiLanguage::Vietnamese;
        if self.scanning_syslog {
            egui::Frame::new()
                .fill(md3::TERTIARY_CONTAINER)
                .corner_radius(16)
                .inner_margin(egui::Margin::same(16))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.vertical(|ui| {
                            let scan_title = if is_vi { "Đang quét syslog..." } else { "Scanning syslog..." };
                            let scan_sub = if is_vi { "Mở ứng dụng Wallet trên iPhone và chạm vào thẻ của bạn" } else { "Open Wallet on iPhone and tap your card" };
                            ui.label(egui::RichText::new(scan_title).strong().size(13.0).color(md3::ON_TERTIARY_CONTAINER));
                            ui.label(egui::RichText::new(scan_sub).size(11.5).color(md3::ON_TERTIARY_CONTAINER));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let stop_txt = if is_vi { "Dừng" } else { "Stop" };
                            let btn = egui::Button::new(egui::RichText::new(stop_txt).size(12.0).color(md3::ON_SURFACE))
                                .fill(md3::ERROR_CONTAINER).corner_radius(20).stroke(egui::Stroke::NONE);
                            if ui.add(btn).clicked() { self.toggle_syslog_scan(); }
                        });
                    });
                });
            ui.add_space(8.0);
        }

        ui.columns(2, |cols| {
            let left = &mut cols[0];
            m3_card(left, |ui| {
                let cfg_title = if is_vi { "Cấu hình thẻ" } else { "Card Configuration" };
                let cfg_sub = if is_vi { "Chọn thẻ mục tiêu và ảnh thay thế" } else { "Target your card and choose replacement artwork" };
                ui.label(egui::RichText::new(cfg_title).strong().size(16.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(cfg_sub).size(12.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(16.0);

                // Target Card Hash
                let hash_title = if is_vi { "Mã Hash thẻ mục tiêu" } else { "Target Card Hash" };
                let hash_hint = if is_vi { "Mã pass hash Base64..." } else { "Base64 pass hash..." };
                ui.label(egui::RichText::new(hash_title).strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let btn_w = 90.0;
                    let text_w = (ui.available_width() - btn_w - 12.0).max(150.0);
                    ui.add(egui::TextEdit::singleline(&mut self.card_hash).hint_text(hash_hint).desired_width(text_w));

                    let scan_label = if self.scanning_syslog { if is_vi { "Dừng" } else { "Stop" } } else { if is_vi { "Quét" } else { "Scan" } };
                    let scan_bg = if self.scanning_syslog { md3::ERROR_CONTAINER } else { md3::PRIMARY_CONTAINER };
                    let scan_fg = if self.scanning_syslog { md3::ERROR } else { md3::ON_PRIMARY_CONTAINER };
                    let scan_btn = egui::Button::new(egui::RichText::new(scan_label).size(12.0).color(scan_fg))
                        .fill(scan_bg).corner_radius(20).stroke(egui::Stroke::NONE);
                    if ui.add(scan_btn).clicked() { self.toggle_syslog_scan(); }
                });

                if !self.saved_cards.is_empty() {
                    ui.add_space(8.0);
                    let saved_lbl = if is_vi { "Thẻ đã lưu" } else { "Saved cards" };
                    let default_sel = if is_vi { "Chọn thẻ..." } else { "Select..." };
                    ui.label(egui::RichText::new(saved_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
                    ui.add_space(2.0);
                    let combo_w = (ui.available_width() - 4.0).max(150.0);
                    let sel_label = self.saved_cards.iter()
                        .find(|c| c.hash == self.card_hash)
                        .map(|c| format!("{} ({})", c.name, &c.hash[..8.min(c.hash.len())]))
                        .unwrap_or_else(|| default_sel.into());

                    egui::ComboBox::from_id_salt("saved_cards_box")
                        .width(combo_w)
                        .selected_text(egui::RichText::new(sel_label).color(md3::ON_SURFACE))
                        .show_ui(ui, |ui| {
                            for card in &self.saved_cards {
                                let is_selected = self.card_hash == card.hash;
                                let label = format!("{} ({}...)", card.name, &card.hash[..8.min(card.hash.len())]);
                                let text = egui::RichText::new(label)
                                    .color(if is_selected { md3::ON_PRIMARY_CONTAINER } else { md3::ON_SURFACE })
                                    .strong();
                                if ui.selectable_label(is_selected, text).clicked() {
                                    self.card_hash = card.hash.clone();
                                }
                            }
                        });
                }

                ui.add_space(16.0);

                // Card Skin
                let skin_title = if is_vi { "Ảnh giao diện thẻ" } else { "Card Skin Artwork" };
                let skin_sub = if is_vi { "PNG, JPG, WebP - tự động co giãn về 1536x969" } else { "PNG, JPG, WebP - auto-scaled to 1536x969" };
                ui.label(egui::RichText::new(skin_title).strong().size(12.0).color(md3::ON_SURFACE));
                ui.label(egui::RichText::new(skin_sub).size(11.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let choose_lbl = if is_vi { "Chọn ảnh..." } else { "Choose Image..." };
                    if m3_button_filled(ui, choose_lbl) { self.select_skin(ctx); }
                    if self.skin.is_some() {
                        let export_lbl = if is_vi { "Xuất PNG" } else { "Export PNG" };
                        if m3_button_tonal(ui, export_lbl) { self.save_prepared_png(); }
                    }
                });

                if let Some(skin) = &self.skin {
                    ui.add_space(4.0);
                    let fname = self.source_path.as_ref()
                        .and_then(|p| p.file_name()).and_then(|n| n.to_str()).unwrap_or("image");
                    ui.label(egui::RichText::new(format!("{} - 1536x969 - {:.0} KB", fname, skin.png.len() as f32 / 1024.0)).size(11.0).color(md3::PRIMARY));
                }

                let mut overlay_changed = false;

                if self.raw_image.is_some() || self.studio.overlay_options.bg_preset == CardBackgroundPreset::CustomImage {
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        let zoom_lbl = if is_vi { "Thu phóng:" } else { "Zoom:" };
                        ui.label(egui::RichText::new(zoom_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
                        let mut zoom_percent = (self.studio.overlay_options.transform.zoom * 100.0).round() as i32;
                        if ui.add(egui::Slider::new(&mut zoom_percent, 20..=500).suffix("%").show_value(true)).changed() {
                            self.studio.overlay_options.transform.zoom = (zoom_percent as f32 / 100.0).clamp(0.05, 50.0);
                            overlay_changed = true;
                        }

                        let reset_fit_lbl = if is_vi { "↺ Vừa khung" } else { "↺ Reset Fit" };
                        let reset_fit_hover = if is_vi { "Khôi phục thu phóng 100% và căn giữa ảnh" } else { "Reset zoom to 100% and center image" };
                        if ui.button(egui::RichText::new(reset_fit_lbl).size(10.5).color(md3::PRIMARY))
                            .on_hover_text(reset_fit_hover)
                            .clicked()
                        {
                            self.studio.overlay_options.transform = ImageTransform::default();
                            overlay_changed = true;
                        }
                    });
                }

                ui.add_space(14.0);

                if self.studio.draw_sidebar(ui, is_vi) {
                    overlay_changed = true;
                }

                if overlay_changed {
                    self.recompute_skin(ctx);
                }

                ui.add_space(16.0);

                // Apply
                let write_lbl = if is_vi { "Ghi vào iPhone" } else { "Write to iPhone" };
                ui.label(egui::RichText::new(write_lbl).strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);

                let can_flash = !self.is_busy && self.selected_udid.is_some() && !self.card_hash.trim().is_empty() && self.skin.is_some();
                let apply_lbl = if is_vi { "Áp dụng giao diện thẻ" } else { "Apply Card Skin" };
                let flash_btn = egui::Button::new(
                    egui::RichText::new(apply_lbl).strong().size(14.0)
                        .color(if can_flash { md3::ON_PRIMARY } else { md3::ON_SURFACE_VARIANT }),
                )
                .fill(if can_flash { md3::PRIMARY } else { md3::SURFACE_CONTAINER_HIGH })
                .corner_radius(20).stroke(egui::Stroke::NONE)
                .min_size(egui::vec2(ui.available_width(), 40.0));

                let resp = ui.add_enabled(can_flash, flash_btn);
                if resp.clicked() { self.flash_card(); }
                if !can_flash {
                    let mut r = Vec::new();
                    if self.selected_udid.is_none() { r.push(if is_vi { "kết nối iPhone qua USB" } else { "connect iPhone" }); }
                    if self.card_hash.trim().is_empty() { r.push(if is_vi { "nhập mã pass hash" } else { "enter card hash" }); }
                    if self.skin.is_none() { r.push(if is_vi { "chọn ảnh thẻ" } else { "choose image" }); }
                    if !r.is_empty() { resp.on_disabled_hover_text(format!("{}: {}", if is_vi { "Cần" } else { "Need" }, r.join(", "))); }
                }

                if self.is_busy {
                    ui.add_space(8.0);
                    if self.progress_total > 0 {
                        ui.add(egui::ProgressBar::new(self.progress_step as f32 / self.progress_total as f32).animate(true));
                    }
                    ui.label(egui::RichText::new(&self.progress_msg).size(11.0).color(md3::PRIMARY));
                }
            });

            // Right: preview
            let right = &mut cols[1];
            m3_card(right, |ui| {
                let prev_title = if is_vi { "Xem trước Wallet" } else { "Wallet Preview" };
                let prev_sub = if is_vi { "Khung thẻ 1536 x 969 px" } else { "1536 x 969 px pass canvas" };
                ui.label(egui::RichText::new(prev_title).strong().size(16.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(prev_sub).size(12.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(8.0);

                if self.studio.draw_preview_card(ui, self.skin_texture.as_ref(), is_vi) {
                    self.recompute_skin(ctx);
                }
            });
        });
    }

    fn show_passcode_tab(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let is_vi = self.ui_language == UiLanguage::Vietnamese;
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            m3_card(left, |ui| {
                let thm_title = if is_vi { "Giao diện Mật mã (Passcode Theme)" } else { "Passcode Theme" };
                let thm_sub = if is_vi { "Tùy biến bàn phím quay số màn hình khóa từ Cowabunga hoặc Nugget" } else { "Custom lockscreen keypad from Cowabunga or Nugget" };
                ui.label(egui::RichText::new(thm_title).strong().size(16.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(thm_sub).size(12.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(16.0);

                // Theme file
                let pkg_hdr = if is_vi { "Gói theme (.passthm)" } else { "Theme Package" };
                let pkg_sub = if is_vi { "Chọn tệp nén .passthm chứa hình ảnh bàn phím quay số" } else { "Choose a .passthm archive containing dialer artwork" };
                ui.label(egui::RichText::new(pkg_hdr).strong().size(12.0).color(md3::ON_SURFACE));
                ui.label(egui::RichText::new(pkg_sub).size(11.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(4.0);
                let choose_thm_lbl = if is_vi { "Chọn .passthm..." } else { "Choose .passthm..." };
                if m3_button_filled(ui, choose_thm_lbl) { self.select_theme_file(ctx); }

                if let Some(theme) = &self.loaded_theme {
                    ui.add_space(4.0);
                    let fname = self.theme_path.as_ref()
                        .and_then(|p| p.file_name()).and_then(|n| n.to_str()).unwrap_or("theme");
                    let asset_txt = if is_vi { "tệp tài nguyên" } else { "assets" };
                    ui.label(egui::RichText::new(format!("{} - {} {}", fname, theme.items.len(), asset_txt)).size(11.0).color(md3::PRIMARY));
                }

                ui.add_space(16.0);

                // iOS version
                let cache_hdr = if is_vi { "Định dạng Cache iOS mục tiêu" } else { "Target iOS Cache" };
                let cache_sub = if is_vi { "Chọn cấu trúc cache tương ứng với phiên bản iOS đã kết nối" } else { "Select cache format based on connected iOS version" };
                ui.label(egui::RichText::new(cache_hdr).strong().size(12.0).color(md3::ON_SURFACE));
                ui.label(egui::RichText::new(cache_sub).size(11.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(4.0);
                let combo_w = (ui.available_width() - 4.0).max(150.0);
                let mut ver_changed = false;
                let auto_label = if is_vi { "Tự động (TelephonyUI-10)" } else { "Auto (TelephonyUI-10)" };
                let legacy_label = if is_vi { "TelephonyUI-8 (Cũ hơn)" } else { "TelephonyUI-8 (Legacy)" };
                let cur_telephony_display = if is_vi && self.forced_telephony_ver == "Auto (TelephonyUI-10)" {
                    auto_label
                } else if is_vi && self.forced_telephony_ver == "TelephonyUI-8" {
                    legacy_label
                } else {
                    &self.forced_telephony_ver
                };
                egui::ComboBox::from_id_salt("telephony_combo")
                    .width(combo_w)
                    .selected_text(cur_telephony_display)
                    .show_ui(ui, |ui| {
                        ver_changed |= ui.selectable_value(&mut self.forced_telephony_ver, "Auto (TelephonyUI-10)".into(), auto_label).clicked();
                        ver_changed |= ui.selectable_value(&mut self.forced_telephony_ver, "TelephonyUI-10".into(), "TelephonyUI-10 (iOS 18+)").clicked();
                        ver_changed |= ui.selectable_value(&mut self.forced_telephony_ver, "TelephonyUI-9".into(), "TelephonyUI-9 (iOS 16-17)").clicked();
                        ver_changed |= ui.selectable_value(&mut self.forced_telephony_ver, "TelephonyUI-8".into(), legacy_label).clicked();
                    });

                if ver_changed {
                    if let Some(path) = self.theme_path.clone() {
                        self.load_theme_from_path(ctx, &path);
                    }
                }

                ui.add_space(16.0);

                // Keypad Language
                let lang_hdr = if is_vi { "Ngôn ngữ bàn phím" } else { "Keypad Language" };
                let lang_sub = if is_vi { "Bố cục chữ cái phụ dưới số (English, Russian, Ukrainian, Japanese, hoặc Universal)" } else { "Subtext alphabet layout (English, Russian, Ukrainian, Japanese, or Universal)" };
                ui.label(egui::RichText::new(lang_hdr).strong().size(12.0).color(md3::ON_SURFACE));
                ui.label(egui::RichText::new(lang_sub).size(11.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(4.0);
                let mut lang_changed = false;
                let univ_label = if is_vi { "Tất cả ngôn ngữ (Universal)" } else { "All Languages (Universal)" };
                let cur_keypad_lang_display = if is_vi && self.keypad_language == "All Languages (Universal)" {
                    univ_label
                } else {
                    &self.keypad_language
                };
                egui::ComboBox::from_id_salt("keypad_lang_combo")
                    .width(combo_w)
                    .selected_text(egui::RichText::new(cur_keypad_lang_display).color(md3::ON_SURFACE))
                    .show_ui(ui, |ui| {
                        lang_changed |= ui.selectable_value(&mut self.keypad_language, "English".into(), "English").clicked();
                        lang_changed |= ui.selectable_value(&mut self.keypad_language, "Russian".into(), "Russian").clicked();
                        lang_changed |= ui.selectable_value(&mut self.keypad_language, "Ukrainian".into(), "Ukrainian").clicked();
                        lang_changed |= ui.selectable_value(&mut self.keypad_language, "Japanese".into(), "Japanese").clicked();
                        lang_changed |= ui.selectable_value(&mut self.keypad_language, "All Languages (Universal)".into(), univ_label).clicked();
                    });

                if lang_changed {
                    if let Some(path) = self.theme_path.clone() {
                        self.load_theme_from_path(ctx, &path);
                    }
                }

                ui.add_space(10.0);

                // Bold Font Toggle
                let mut bold_changed = false;
                ui.horizontal(|ui| {
                    let bold_lbl = if is_vi { "Chữ đậm (Trợ năng iOS - Bold Text)" } else { "Bold Text (iOS Accessibility)" };
                    if ui.checkbox(&mut self.passcode_bold, egui::RichText::new(bold_lbl).strong().size(12.0).color(md3::ON_SURFACE)).changed() {
                        bold_changed = true;
                    }
                });
                let bold_sub = if is_vi {
                    "Tạo ảnh *-bold.png cho thiết bị đã BẬT Chữ đậm trong Cài đặt iPhone ➔ Màn hình & Độ sáng"
                } else {
                    "Generates *-bold.png for devices with Bold Text turned ON in iPhone Settings -> Display"
                };
                ui.label(
                    egui::RichText::new(bold_sub)
                        .size(11.0)
                        .color(md3::ON_SURFACE_VARIANT),
                );

                if bold_changed {
                    if let Some(path) = self.theme_path.clone() {
                        self.load_theme_from_path(ctx, &path);
                    }
                }

                ui.add_space(16.0);

                // Apply
                let write_lbl = if is_vi { "Ghi dữ liệu vào iPhone" } else { "Write to iPhone" };
                ui.label(egui::RichText::new(write_lbl).strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);

                let can_flash = !self.is_busy && self.selected_udid.is_some() && self.loaded_theme.is_some();
                let apply_thm_lbl = if is_vi { "Áp dụng giao diện Mật mã" } else { "Apply Passcode Theme" };
                let flash_btn = egui::Button::new(
                    egui::RichText::new(apply_thm_lbl).strong().size(14.0)
                        .color(if can_flash { md3::ON_PRIMARY } else { md3::ON_SURFACE_VARIANT }),
                )
                .fill(if can_flash { md3::PRIMARY } else { md3::SURFACE_CONTAINER_HIGH })
                .corner_radius(20).stroke(egui::Stroke::NONE)
                .min_size(egui::vec2(ui.available_width(), 40.0));

                let resp = ui.add_enabled(can_flash, flash_btn);
                if resp.clicked() { self.flash_theme(); }
                if !can_flash {
                    let mut r = Vec::new();
                    if self.selected_udid.is_none() { r.push(if is_vi { "kết nối iPhone" } else { "connect iPhone" }); }
                    if self.loaded_theme.is_none() { r.push(if is_vi { "chọn theme" } else { "select theme" }); }
                    if !r.is_empty() { resp.on_disabled_hover_text(format!("{}: {}", if is_vi { "Cần" } else { "Need" }, r.join(", "))); }
                }

                if self.is_busy {
                    ui.add_space(8.0);
                    if self.progress_total > 0 {
                        ui.add(egui::ProgressBar::new(self.progress_step as f32 / self.progress_total as f32).animate(true));
                    }
                    ui.label(egui::RichText::new(&self.progress_msg).size(11.0).color(md3::PRIMARY));
                }
            });

            // Right: preview
            let right = &mut cols[1];
            m3_card(right, |ui| {
                let prev_title = if is_vi { "Xem trước bàn phím số" } else { "Keypad Preview" };
                let prev_sub = if is_vi { "Hình ảnh nút bấm quay số màn hình khóa" } else { "Dialer button artwork" };
                ui.label(egui::RichText::new(prev_title).strong().size(16.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(prev_sub).size(12.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(12.0);

                let pass_w = (ui.available_width() - 8.0).clamp(240.0, 360.0);
                let pass_h = 265.0;

                ui.vertical_centered(|ui| {
                    if self.keypad_textures.is_empty() {
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(pass_w, pass_h), egui::Sense::hover());
                        let painter = ui.painter();
                        painter.rect_filled(rect, 16.0, md3::SURFACE_CONTAINER_HIGH);
                        let no_thm = if is_vi { "Chưa nạp theme" } else { "No theme loaded" };
                        painter.text(rect.center(), egui::Align2::CENTER_CENTER,
                            no_thm, egui::FontId::proportional(14.0), md3::ON_SURFACE_VARIANT);
                    } else {
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(pass_w, pass_h), egui::Sense::hover());
                        let painter = ui.painter();
                        painter.rect_filled(rect, 16.0, md3::SURFACE_CONTAINER_HIGH);
                        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                            ui.vertical_centered(|ui| {
                                ui.add_space(10.0);
                                const DIALER_LAYOUT: &[&[&str]] = &[
                                    &["1", "2", "3"],
                                    &["4", "5", "6"],
                                    &["7", "8", "9"],
                                    &["", "0", ""],
                                ];
                                egui::Grid::new("keypad_grid")
                                    .spacing([18.0, 6.0])
                                    .show(ui, |ui| {
                                        for row in DIALER_LAYOUT {
                                            for &d in *row {
                                                if d.is_empty() {
                                                    ui.allocate_exact_size(egui::vec2(44.0, 50.0), egui::Sense::hover());
                                                } else if let Some((_, tex)) = self.keypad_textures.iter().find(|(k, _)| k == d) {
                                                    ui.vertical_centered(|ui| {
                                                        egui::Frame::new()
                                                            .fill(md3::SURFACE)
                                                            .corner_radius(12)
                                                            .inner_margin(3)
                                                            .show(ui, |ui| { ui.image((tex.id(), egui::vec2(40.0, 40.0))); });
                                                        ui.label(egui::RichText::new(d).size(9.5).color(md3::ON_SURFACE_VARIANT));
                                                    });
                                                } else {
                                                    ui.vertical_centered(|ui| {
                                                        egui::Frame::new()
                                                            .fill(md3::SURFACE)
                                                            .corner_radius(12)
                                                            .inner_margin(3)
                                                            .show(ui, |ui| {
                                                                let (btn_rect, _) = ui.allocate_exact_size(egui::vec2(40.0, 40.0), egui::Sense::hover());
                                                                ui.painter().rect_filled(btn_rect, 8.0, md3::SURFACE_CONTAINER);
                                                                ui.painter().text(btn_rect.center(), egui::Align2::CENTER_CENTER, d, egui::FontId::proportional(14.0), md3::ON_SURFACE_VARIANT);
                                                            });
                                                        ui.label(egui::RichText::new(d).size(9.5).color(md3::ON_SURFACE_VARIANT));
                                                    });
                                                }
                                            }
                                            ui.end_row();
                                        }
                                    });
                            });
                        });
                    }
                });

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    let pad_lbl = if is_vi { "Bàn phím 3x4" } else { "3x4 Keypad" };
                    ui.label(egui::RichText::new(pad_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
                    ui.label(egui::RichText::new("|").size(11.0).color(md3::OUTLINE_VARIANT));
                    ui.label(egui::RichText::new("TelephonyUI").size(11.0).color(md3::ON_SURFACE_VARIANT));
                    ui.label(egui::RichText::new("|").size(11.0).color(md3::OUTLINE_VARIANT));
                    if self.loaded_theme.is_some() {
                        let ready_lbl = if is_vi { "Sẵn sàng" } else { "Ready" };
                        ui.label(egui::RichText::new(ready_lbl).size(11.0).color(md3::SUCCESS));
                    } else {
                        let no_thm_lbl = if is_vi { "Chưa có theme" } else { "No theme" };
                        ui.label(egui::RichText::new(no_thm_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
                    }
                });
                ui.add_space(8.0);
                let lock_hint = if is_vi {
                    "Sau khi áp dụng, khóa màn hình iPhone để kiểm tra giao diện bàn phím mới."
                } else {
                    "After applying, lock your iPhone to see the new keypad."
                };
                ui.label(egui::RichText::new(lock_hint).size(11.0).color(md3::ON_SURFACE_VARIANT));
            });
        });
    }

    fn show_help_tab(&mut self, ui: &mut egui::Ui) {
        let is_vi = self.ui_language == UiLanguage::Vietnamese;
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            m3_card(left, |ui| {
                let title = if is_vi { "Hướng dẫn kết nối & Quét mã thẻ" } else { "Setup & Card Hash Guide" };
                let subtitle = if is_vi { "Tất cả những gì bạn cần để kết nối thiết bị và lấy mã thẻ" } else { "Everything you need to connect and capture your card" };
                ui.label(egui::RichText::new(title).strong().size(16.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(subtitle).size(12.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(16.0);

                let prereq_hdr = if is_vi { "Điều kiện cần" } else { "Prerequisites" };
                ui.label(egui::RichText::new(prereq_hdr).strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(6.0);
                let p1 = if is_vi { "- Đã cài đặt iTunes 64-bit hoặc Apple Mobile Device Support" } else { "- 64-bit iTunes or Apple Mobile Device Support installed" };
                let p2 = if is_vi { "- Kết nối iPhone qua cáp USB-C hoặc Lightning chính hãng" } else { "- Connect iPhone via USB-C or Lightning cable" };
                let p3 = if is_vi { "- Mở khóa iPhone và bấm 'Tin cậy máy tính này'" } else { "- Unlock iPhone and tap 'Trust this Computer'" };
                ui.label(egui::RichText::new(p1).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(p2).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(p3).size(11.5).color(md3::ON_SURFACE_VARIANT));

                ui.add_space(18.0);

                let find_hdr = if is_vi { "Cách lấy mã Hash thẻ của bạn" } else { "Finding Your Card Hash" };
                ui.label(egui::RichText::new(find_hdr).strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(6.0);
                let s1 = if is_vi { "1. Bấm nút 'Quét' trong tab Thẻ Apple Pay" } else { "1. Click 'Scan' in the Wallet tab" };
                let s2 = if is_vi { "2. Mở ứng dụng Apple Wallet trên iPhone" } else { "2. Open Apple Wallet on your iPhone" };
                let s3 = if is_vi { "3. Chạm vào thẻ bạn muốn tùy biến giao diện" } else { "3. Tap the card you want to customize" };
                let s4 = if is_vi { "4. AirCard sẽ tự động nhận diện mã pass hash qua syslog" } else { "4. AirCard captures the pass hash automatically" };
                let s5 = if is_vi { "5. Bấm 'Dừng' sau khi mã thẻ đã được nhận diện" } else { "5. Click 'Stop' once detected" };
                ui.label(egui::RichText::new(s1).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(s2).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(s3).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(s4).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(s5).size(11.5).color(md3::ON_SURFACE_VARIANT));
            });

            let right = &mut cols[1];
            m3_card(right, |ui| {
                let act_title = if is_vi { "Hướng dẫn kích hoạt & Cài đặt Theme" } else { "Activation & Theme Guide" };
                let act_sub = if is_vi { "Kích hoạt giao diện thẻ và gói bàn phím số" } else { "Applying skins and dialer keypad packages" };
                ui.label(egui::RichText::new(act_title).strong().size(16.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(act_sub).size(12.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(16.0);

                let act_hdr = if is_vi { "Kích hoạt giao diện thẻ Apple Wallet" } else { "Activating Apple Wallet Skin" };
                ui.label(egui::RichText::new(act_hdr).strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(6.0);
                let as1 = if is_vi { "1. Bấm 'Áp dụng giao diện thẻ' và chờ hệ thống hoàn tất" } else { "1. Click 'Apply Card Skin' and wait for completion" };
                let as2 = if is_vi { "2. Mở App Switcher trên iPhone (vuốt từ dưới đáy màn hình lên)" } else { "2. Open App Switcher on iPhone (swipe up from bottom)" };
                let as3 = if is_vi { "3. Vuốt tắt hoàn toàn ứng dụng Apple Wallet" } else { "3. Force close Apple Wallet by swiping up on it" };
                let as4 = if is_vi { "4. Mở lại Apple Wallet - giao diện thẻ mới sẽ xuất hiện ngay!" } else { "4. Reopen Wallet - your new skin appears!" };
                ui.label(egui::RichText::new(as1).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(as2).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(as3).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(as4).size(11.5).color(md3::ON_SURFACE_VARIANT));

                ui.add_space(18.0);

                let thm_hdr = if is_vi { "Gói theme bàn phím số (.passthm)" } else { "Passcode Themes (.passthm)" };
                ui.label(egui::RichText::new(thm_hdr).strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(6.0);
                let pt1 = if is_vi { "- Tương thích hoàn toàn với các gói theme Cowabunga & Nugget" } else { "- Compatible with Cowabunga & Nugget theme packages" };
                let pt2 = if is_vi { "- iOS 18+: Chọn 'Auto (TelephonyUI-10)'" } else { "- iOS 18+: Select 'Auto (TelephonyUI-10)'" };
                let pt3 = if is_vi { "- iOS 16-17: Chọn 'TelephonyUI-9'" } else { "- iOS 16-17: Select 'TelephonyUI-9'" };
                let pt4 = if is_vi { "- Khóa màn hình iPhone để kiểm tra giao diện bàn phím mới" } else { "- Lock screen to verify your updated keypad artwork" };
                ui.label(egui::RichText::new(pt1).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(pt2).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(pt3).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(pt4).size(11.5).color(md3::ON_SURFACE_VARIANT));
            });
        });
    }

    fn show_sources_tab(&mut self, ui: &mut egui::Ui) {
        let is_vi = self.ui_language == UiLanguage::Vietnamese;
        crate::card_studio::sources::draw_sources_tab(ui, is_vi);
    }
}
