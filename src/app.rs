use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::thread;

use eframe::egui;

use crate::apple;
use crate::device::{DeviceInfo, list_connected_devices};
use crate::flasher::{flash_passcode_theme, flash_wallet_skin};
use crate::image_skin::{
    CardBackgroundPreset, CardFinish, CardOverlayOptions, EmbossStyle,
    ImageTransform, LogoBadgeStyle, LogoColorTheme, PaymentNetwork, PreparedSkin, TextBackdropStyle,
};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveTransformLayer {
    #[default]
    Background,
    Finish,
    Chip,
    Logo,
}

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
    custom_logo_image: Option<image::RgbaImage>,
    custom_logo_path: Option<PathBuf>,
    custom_chip_image: Option<image::RgbaImage>,
    custom_chip_path: Option<PathBuf>,
    custom_finish_image: Option<image::RgbaImage>,
    custom_finish_path: Option<PathBuf>,
    active_layer: ActiveTransformLayer,
    ui_language: UiLanguage,
    overlay_options: CardOverlayOptions,
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
            custom_logo_image: None,
            custom_logo_path: None,
            custom_chip_image: None,
            custom_chip_path: None,
            custom_finish_image: None,
            custom_finish_path: None,
            active_layer: ActiveTransformLayer::Background,
            ui_language: UiLanguage::English,
            overlay_options: CardOverlayOptions::default(),
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

        app.add_log("AIrCard-CMaku Windows v1.2.2 initialized");
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
                self.overlay_options.bg_preset = CardBackgroundPreset::CustomImage;
                self.overlay_options.transform = ImageTransform::default();
                match PreparedSkin::from_preset_or_image(Some(img), &self.overlay_options, self.custom_logo_image.as_ref()) {
                    Ok(skin) => {
                        self.add_log(format!(
                            "Skin processed: source {}x{} resampled to 1536x969 PNG ({:.1} KB)",
                            skin.source_width,
                            skin.source_height,
                            skin.png.len() as f32 / 1024.0,
                        ));
                        self.skin_texture = Some(ctx.load_texture(
                            "card-skin-preview",
                            skin.preview.clone(),
                            egui::TextureOptions::LINEAR,
                        ));
                        self.status_msg = format!(
                            "Prepared {} ({}x{} -> 1536x969 PNG, {:.1} KB)",
                            path.file_name().and_then(|n| n.to_str()).unwrap_or("image"),
                            skin.source_width,
                            skin.source_height,
                            skin.png.len() as f32 / 1024.0,
                        );
                        self.skin = Some(skin);
                    }
                    Err(error) => {
                        self.add_log(format!("Image preparation failed: {error:#}"));
                        self.status_msg = format!("Could not prepare image: {error:#}");
                    }
                }
            }
            Err(error) => {
                self.add_log(format!("Failed to open image {}: {error:#}", path.display()));
                self.status_msg = format!("Could not open image: {error:#}");
            }
        }
    }

    fn select_custom_logo(&mut self, ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Select Custom Logo Image (PNG/JPG)")
            .add_filter("Image files", &["png", "jpg", "jpeg", "webp"])
            .pick_file()
        else {
            return;
        };

        self.add_log(format!("Opening custom logo: {}", path.display()));
        match image::open(&path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                self.custom_logo_image = Some(rgba);
                self.custom_logo_path = Some(path.clone());
                self.overlay_options.network = PaymentNetwork::Custom;
                self.add_log(format!("Custom logo loaded: {}", path.display()));
                self.recompute_skin(ctx);
            }
            Err(err) => {
                self.add_log(format!("Failed to open custom logo {}: {err:#}", path.display()));
                self.status_msg = format!("Could not open custom logo: {err:#}");
            }
        }
    }

    fn select_custom_chip(&mut self, ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Select Custom EMV Chip Image (PNG with transparency)")
            .add_filter("Image files", &["png", "webp"])
            .pick_file()
        else {
            return;
        };

        self.add_log(format!("Opening custom chip: {}", path.display()));
        match image::open(&path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                self.custom_chip_image = Some(rgba);
                self.custom_chip_path = Some(path.clone());
                self.overlay_options.show_chip = true;
                self.active_layer = ActiveTransformLayer::Chip;
                self.add_log(format!("Custom chip loaded: {}", path.display()));
                self.recompute_skin(ctx);
            }
            Err(err) => {
                self.add_log(format!("Failed to open custom chip {}: {err:#}", path.display()));
                self.status_msg = format!("Could not open custom chip: {err:#}");
            }
        }
    }

    fn select_custom_finish(&mut self, ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Select Custom Texture / Foil Image (PNG/JPG/WebP)")
            .add_filter("Image files", &["png", "jpg", "jpeg", "webp"])
            .pick_file()
        else {
            return;
        };

        self.add_log(format!("Opening custom finish texture: {}", path.display()));
        match image::open(&path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                self.custom_finish_image = Some(rgba);
                self.custom_finish_path = Some(path.clone());
                self.overlay_options.finish = CardFinish::CustomTexture;
                self.active_layer = ActiveTransformLayer::Finish;
                self.add_log(format!("Custom finish texture loaded: {}", path.display()));
                self.recompute_skin(ctx);
            }
            Err(err) => {
                self.add_log(format!("Failed to open finish texture {}: {err:#}", path.display()));
                self.status_msg = format!("Could not open finish texture: {err:#}");
            }
        }
    }

    fn recompute_skin(&mut self, ctx: &egui::Context) {
        if self.raw_image.is_none() && self.overlay_options.bg_preset == CardBackgroundPreset::CustomImage {
            return;
        }
        match PreparedSkin::from_preset_or_image_with_assets(
            self.raw_image.clone(),
            &self.overlay_options,
            self.custom_logo_image.as_ref(),
            self.custom_chip_image.as_ref(),
            self.custom_finish_image.as_ref(),
        ) {
            Ok(skin) => {
                self.skin_texture = Some(ctx.load_texture(
                    "card-skin-preview",
                    skin.preview.clone(),
                    egui::TextureOptions::LINEAR,
                ));
                self.skin = Some(skin);
            }
            Err(err) => {
                self.add_log(format!("Failed to re-render card skin with overlays: {err:#}"));
            }
        }
    }

    fn save_prepared_png(&mut self) {
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
                        egui::RichText::new("AIrCard-CMaku")
                            .strong()
                            .size(18.0)
                            .color(md3::ON_SURFACE),
                    );
                    ui.label(
                        egui::RichText::new("v1.2.2")
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
                        UiLanguage::English => ("🌐 VI", "Chuyển giao diện sang Tiếng Việt"),
                        UiLanguage::Vietnamese => ("🌐 EN", "Switch interface to English"),
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
                        if m3_button_outlined(ui, "Refresh") {
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
                            ui.label(egui::RichText::new("No device").size(12.0).color(md3::ON_SURFACE_VARIANT))
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
                    ui.label(egui::RichText::new(&self.status_msg).size(11.5).color(md3::ON_SURFACE_VARIANT));

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
        if self.scanning_syslog {
            egui::Frame::new()
                .fill(md3::TERTIARY_CONTAINER)
                .corner_radius(16)
                .inner_margin(egui::Margin::same(16))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Scanning syslog...").strong().size(13.0).color(md3::ON_TERTIARY_CONTAINER));
                            ui.label(egui::RichText::new("Open Wallet on iPhone and tap your card").size(11.5).color(md3::ON_TERTIARY_CONTAINER));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let btn = egui::Button::new(egui::RichText::new("Stop").size(12.0).color(md3::ON_SURFACE))
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
                ui.label(egui::RichText::new("Card Configuration").strong().size(16.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Target your card and choose replacement artwork").size(12.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(16.0);

                // Target Card Hash
                ui.label(egui::RichText::new("Target Card Hash").strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let btn_w = 90.0;
                    let text_w = (ui.available_width() - btn_w - 12.0).max(150.0);
                    ui.add(egui::TextEdit::singleline(&mut self.card_hash).hint_text("Base64 pass hash...").desired_width(text_w));

                    let scan_label = if self.scanning_syslog { "Stop" } else { "Scan" };
                    let scan_bg = if self.scanning_syslog { md3::ERROR_CONTAINER } else { md3::PRIMARY_CONTAINER };
                    let scan_fg = if self.scanning_syslog { md3::ERROR } else { md3::ON_PRIMARY_CONTAINER };
                    let scan_btn = egui::Button::new(egui::RichText::new(scan_label).size(12.0).color(scan_fg))
                        .fill(scan_bg).corner_radius(20).stroke(egui::Stroke::NONE);
                    if ui.add(scan_btn).clicked() { self.toggle_syslog_scan(); }
                });

                if !self.saved_cards.is_empty() {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Saved cards").size(11.0).color(md3::ON_SURFACE_VARIANT));
                    ui.add_space(2.0);
                    let combo_w = (ui.available_width() - 4.0).max(150.0);
                    let sel_label = self.saved_cards.iter()
                        .find(|c| c.hash == self.card_hash)
                        .map(|c| format!("{} ({})", c.name, &c.hash[..8.min(c.hash.len())]))
                        .unwrap_or_else(|| "Select...".into());

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
                ui.label(egui::RichText::new("Card Skin Artwork").strong().size(12.0).color(md3::ON_SURFACE));
                ui.label(egui::RichText::new("PNG, JPG, WebP - auto-scaled to 1536x969").size(11.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if m3_button_filled(ui, "Choose Image...") { self.select_skin(ctx); }
                    if self.skin.is_some() {
                        if m3_button_tonal(ui, "Export PNG") { self.save_prepared_png(); }
                    }
                });

                if let Some(skin) = &self.skin {
                    ui.add_space(4.0);
                    let fname = self.source_path.as_ref()
                        .and_then(|p| p.file_name()).and_then(|n| n.to_str()).unwrap_or("image");
                    ui.label(egui::RichText::new(format!("{} - 1536x969 - {:.0} KB", fname, skin.png.len() as f32 / 1024.0)).size(11.0).color(md3::PRIMARY));
                }

                let mut overlay_changed = false;

                if self.raw_image.is_some() || self.overlay_options.bg_preset == CardBackgroundPreset::CustomImage {
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Zoom:").size(11.0).color(md3::ON_SURFACE_VARIANT));
                        let mut zoom_percent = (self.overlay_options.transform.zoom * 100.0).round() as i32;
                        if ui.add(egui::Slider::new(&mut zoom_percent, 20..=500).suffix("%").show_value(true)).changed() {
                            self.overlay_options.transform.zoom = (zoom_percent as f32 / 100.0).clamp(0.05, 50.0);
                            overlay_changed = true;
                        }

                        if ui.button(egui::RichText::new("↺ Reset Fit").size(10.5).color(md3::PRIMARY))
                            .on_hover_text("Reset zoom to 100% and center image")
                            .clicked()
                        {
                            self.overlay_options.transform = ImageTransform::default();
                            overlay_changed = true;
                        }
                    });
                }

                ui.add_space(14.0);

                // Card Overlays & Studio Customization
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Card Studio & Customization").strong().size(12.0).color(md3::ON_SURFACE));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(egui::RichText::new("↺ Reset to Default").size(10.5).color(md3::PRIMARY))
                            .on_hover_text("Reset all card customizations, overlays, and logos to default")
                            .clicked()
                        {
                            self.overlay_options = CardOverlayOptions::default();
                            self.custom_logo_image = None;
                            self.custom_logo_path = None;
                            self.custom_chip_image = None;
                            self.custom_chip_path = None;
                            self.custom_finish_image = None;
                            self.custom_finish_path = None;
                            self.active_layer = ActiveTransformLayer::Background;
                            overlay_changed = true;
                        }
                    });
                });
                ui.add_space(4.0);

                // Row 1: Background Preset & Finish
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Background:").size(11.0).color(md3::ON_SURFACE_VARIANT));
                    let cur_bg = self.overlay_options.bg_preset;
                    egui::ComboBox::from_id_salt("bg_preset_select")
                        .width(160.0)
                        .selected_text(egui::RichText::new(cur_bg.display_name()).size(11.0).color(md3::ON_SURFACE))
                        .show_ui(ui, |ui| {
                            for preset in [
                                CardBackgroundPreset::CustomImage,
                                CardBackgroundPreset::MatteBlack,
                                CardBackgroundPreset::OceanNavy,
                                CardBackgroundPreset::BrushedGold,
                                CardBackgroundPreset::EmeraldLuxury,
                                CardBackgroundPreset::TitaniumMinimal,
                                CardBackgroundPreset::CrimsonVelvet,
                                CardBackgroundPreset::DeepCyberViolet,
                            ] {
                                let is_sel = cur_bg == preset;
                                if ui.selectable_label(is_sel, preset.display_name()).clicked() {
                                    if self.overlay_options.bg_preset != preset {
                                        self.overlay_options.bg_preset = preset;
                                        overlay_changed = true;
                                    }
                                }
                            }
                        });

                    ui.label(egui::RichText::new("Finish:").size(11.0).color(md3::ON_SURFACE_VARIANT));
                    let cur_finish = self.overlay_options.finish;
                    egui::ComboBox::from_id_salt("finish_select")
                        .width(135.0)
                        .selected_text(egui::RichText::new(cur_finish.display_name()).size(11.0).color(md3::ON_SURFACE))
                        .show_ui(ui, |ui| {
                            for finish in [
                                CardFinish::Standard,
                                CardFinish::MetallicSheen,
                                CardFinish::CarbonWeave,
                                CardFinish::CustomTexture,
                            ] {
                                let is_sel = cur_finish == finish;
                                if ui.selectable_label(is_sel, finish.display_name()).clicked() {
                                    if self.overlay_options.finish != finish {
                                        self.overlay_options.finish = finish;
                                        if finish == CardFinish::CustomTexture {
                                            self.active_layer = ActiveTransformLayer::Finish;
                                        }
                                        overlay_changed = true;
                                    }
                                }
                            }
                        });
                });

                // Row 1b: Custom Texture / Foil Controls (when CustomTexture finish is selected or texture loaded)
                if self.overlay_options.finish == CardFinish::CustomTexture || self.custom_finish_image.is_some() {
                    ui.add_space(3.0);
                    ui.horizontal(|ui| {
                        let tex_title = if let Some(path) = &self.custom_finish_path {
                            format!("Texture: {}", path.file_name().and_then(|n| n.to_str()).unwrap_or("texture.png"))
                        } else {
                            "Upload Texture / Foil...".to_string()
                        };
                        if m3_button_tonal(ui, &tex_title) {
                            self.select_custom_finish(ctx);
                        }
                        if self.custom_finish_image.is_some() {
                            if ui.button(egui::RichText::new("Clear").size(11.0).color(md3::ERROR)).clicked() {
                                self.custom_finish_image = None;
                                self.custom_finish_path = None;
                                self.overlay_options.finish = CardFinish::Standard;
                                overlay_changed = true;
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Opacity:").size(10.5).color(md3::ON_SURFACE_VARIANT));
                        let mut op_pct = (self.overlay_options.finish_opacity * 100.0).round() as i32;
                        if ui.add(egui::Slider::new(&mut op_pct, 5..=100).suffix("%")).changed() {
                            self.overlay_options.finish_opacity = (op_pct as f32 / 100.0).clamp(0.05, 1.0);
                            overlay_changed = true;
                        }

                        ui.label(egui::RichText::new("Scale:").size(10.5).color(md3::ON_SURFACE_VARIANT));
                        let mut sc_pct = (self.overlay_options.finish_scale * 100.0).round() as i32;
                        if ui.add(egui::Slider::new(&mut sc_pct, 20..=500).suffix("%")).changed() {
                            self.overlay_options.finish_scale = (sc_pct as f32 / 100.0).clamp(0.1, 10.0);
                            overlay_changed = true;
                        }

                        if ui.button(egui::RichText::new("↺ Reset").size(10.5).color(md3::PRIMARY))
                            .on_hover_text("Reset texture pan, scale, and opacity")
                            .clicked()
                        {
                            self.overlay_options.finish_x = 0.0;
                            self.overlay_options.finish_y = 0.0;
                            self.overlay_options.finish_scale = 1.0;
                            self.overlay_options.finish_opacity = 0.60;
                            overlay_changed = true;
                        }
                    });
                }

                ui.add_space(4.0);

                // Row 2: Brand, Logo Frame/Outline, Logo Color
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Brand:").size(11.0).color(md3::ON_SURFACE_VARIANT));
                    let cur_net = self.overlay_options.network;
                    egui::ComboBox::from_id_salt("network_select")
                        .width(95.0)
                        .selected_text(egui::RichText::new(cur_net.display_name()).size(11.0).color(md3::ON_SURFACE))
                        .show_ui(ui, |ui| {
                            for net in [
                                PaymentNetwork::None,
                                PaymentNetwork::Visa,
                                PaymentNetwork::Mastercard,
                                PaymentNetwork::Napas,
                                PaymentNetwork::Jcb,
                                PaymentNetwork::Custom,
                            ] {
                                let is_sel = cur_net == net;
                                if ui.selectable_label(is_sel, net.display_name()).clicked() {
                                    if self.overlay_options.network != net {
                                        self.overlay_options.network = net;
                                        if net != PaymentNetwork::None {
                                            self.active_layer = ActiveTransformLayer::Logo;
                                        }
                                        overlay_changed = true;
                                    }
                                }
                            }
                        });

                    ui.label(egui::RichText::new("Frame:").size(11.0).color(md3::ON_SURFACE_VARIANT));
                    let cur_badge = self.overlay_options.logo_style;
                    egui::ComboBox::from_id_salt("logo_badge_select")
                        .width(115.0)
                        .selected_text(egui::RichText::new(cur_badge.display_name()).size(11.0).color(md3::ON_SURFACE))
                        .show_ui(ui, |ui| {
                            for style in [
                                LogoBadgeStyle::Transparent,
                                LogoBadgeStyle::ThinOutline,
                                LogoBadgeStyle::FrostedGlass,
                                LogoBadgeStyle::SolidDark,
                                LogoBadgeStyle::SolidLight,
                                LogoBadgeStyle::SubtleGlow,
                            ] {
                                let is_sel = cur_badge == style;
                                if ui.selectable_label(is_sel, style.display_name()).clicked() {
                                    if self.overlay_options.logo_style != style {
                                        self.overlay_options.logo_style = style;
                                        overlay_changed = true;
                                    }
                                }
                            }
                        });

                    ui.label(egui::RichText::new("Color:").size(11.0).color(md3::ON_SURFACE_VARIANT));
                    let cur_color = self.overlay_options.logo_color;
                    egui::ComboBox::from_id_salt("logo_color_select")
                        .width(110.0)
                        .selected_text(egui::RichText::new(cur_color.display_name()).size(11.0).color(md3::ON_SURFACE))
                        .show_ui(ui, |ui| {
                            for theme in [
                                LogoColorTheme::Original,
                                LogoColorTheme::MonochromeWhite,
                                LogoColorTheme::LuxuryGold,
                                LogoColorTheme::SilverPlatinum,
                                LogoColorTheme::StealthBlack,
                            ] {
                                let is_sel = cur_color == theme;
                                if ui.selectable_label(is_sel, theme.display_name()).clicked() {
                                    if self.overlay_options.logo_color != theme {
                                        self.overlay_options.logo_color = theme;
                                        overlay_changed = true;
                                    }
                                }
                            }
                        });
                });

                // Row 2b: Custom Logo Upload (when Custom brand is selected or logo is loaded)
                if self.overlay_options.network == PaymentNetwork::Custom || self.custom_logo_image.is_some() {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let btn_title = if let Some(path) = &self.custom_logo_path {
                            format!("Custom: {}", path.file_name().and_then(|n| n.to_str()).unwrap_or("logo.png"))
                        } else {
                            "Upload Custom Logo (PNG)...".to_string()
                        };
                        if m3_button_tonal(ui, &btn_title) {
                            self.select_custom_logo(ctx);
                        }
                        if self.custom_logo_image.is_some() {
                            if ui.button(egui::RichText::new("Clear").size(11.0).color(md3::ERROR)).clicked() {
                                self.custom_logo_image = None;
                                self.custom_logo_path = None;
                                self.overlay_options.network = PaymentNetwork::None;
                                overlay_changed = true;
                            }
                        }
                    });
                }

                // Row 2c: Logo Transform Controls (when a network brand or custom logo is active)
                if self.overlay_options.network != PaymentNetwork::None {
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Logo Scale:").size(10.5).color(md3::ON_SURFACE_VARIANT));
                        let mut logo_sc_pct = (self.overlay_options.logo_scale * 100.0).round() as i32;
                        if ui.add(egui::Slider::new(&mut logo_sc_pct, 30..=250).suffix("%")).changed() {
                            self.overlay_options.logo_scale = (logo_sc_pct as f32 / 100.0).clamp(0.2, 5.0);
                            overlay_changed = true;
                        }

                        ui.label(egui::RichText::new("X:").size(10.5).color(md3::ON_SURFACE_VARIANT));
                        if ui.add(egui::DragValue::new(&mut self.overlay_options.logo_x).range(0.0..=1500.0).speed(1.0)).changed() {
                            overlay_changed = true;
                        }

                        ui.label(egui::RichText::new("Y:").size(10.5).color(md3::ON_SURFACE_VARIANT));
                        if ui.add(egui::DragValue::new(&mut self.overlay_options.logo_y).range(0.0..=1000.0).speed(1.0)).changed() {
                            overlay_changed = true;
                        }

                        if ui.button(egui::RichText::new("↺ Reset").size(10.5).color(md3::PRIMARY))
                            .on_hover_text("Reset logo position and scale")
                            .clicked()
                        {
                            self.overlay_options.logo_x = 1205.0;
                            self.overlay_options.logo_y = 800.0;
                            self.overlay_options.logo_scale = 1.0;
                            overlay_changed = true;
                        }
                    });
                }

                ui.add_space(4.0);

                // Row 3: Hardware & Details Toggles
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.overlay_options.show_chip, egui::RichText::new("EMV Gold Chip").size(11.5).color(md3::ON_SURFACE)).changed() {
                        if self.overlay_options.show_chip {
                            self.active_layer = ActiveTransformLayer::Chip;
                        }
                        overlay_changed = true;
                    }
                    if ui.checkbox(&mut self.overlay_options.show_contactless, egui::RichText::new("Contactless Wave").size(11.5).color(md3::ON_SURFACE)).changed() {
                        overlay_changed = true;
                    }
                    if ui.checkbox(&mut self.overlay_options.details.show_details, egui::RichText::new("Emboss Card Details").size(11.5).color(md3::PRIMARY)).changed() {
                        overlay_changed = true;
                    }
                });

                // Row 3b: Custom Chip Upload & Transform Controls (when chip is visible)
                if self.overlay_options.show_chip {
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        let chip_title = if let Some(path) = &self.custom_chip_path {
                            format!("Chip: {}", path.file_name().and_then(|n| n.to_str()).unwrap_or("chip.png"))
                        } else {
                            "Upload Custom Chip (PNG)...".to_string()
                        };
                        if m3_button_tonal(ui, &chip_title) {
                            self.select_custom_chip(ctx);
                        }
                        if self.custom_chip_image.is_some() {
                            if ui.button(egui::RichText::new("Use Default Gold").size(11.0).color(md3::ON_SURFACE_VARIANT))
                                .on_hover_text("Revert to default procedural EMV gold chip")
                                .clicked()
                            {
                                self.custom_chip_image = None;
                                self.custom_chip_path = None;
                                overlay_changed = true;
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Chip Scale:").size(10.5).color(md3::ON_SURFACE_VARIANT));
                        let mut chip_sc_pct = (self.overlay_options.chip_scale * 100.0).round() as i32;
                        if ui.add(egui::Slider::new(&mut chip_sc_pct, 30..=250).suffix("%")).changed() {
                            self.overlay_options.chip_scale = (chip_sc_pct as f32 / 100.0).clamp(0.2, 5.0);
                            overlay_changed = true;
                        }

                        ui.label(egui::RichText::new("X:").size(10.5).color(md3::ON_SURFACE_VARIANT));
                        if ui.add(egui::DragValue::new(&mut self.overlay_options.chip_x).range(0.0..=1500.0).speed(1.0)).changed() {
                            overlay_changed = true;
                        }

                        ui.label(egui::RichText::new("Y:").size(10.5).color(md3::ON_SURFACE_VARIANT));
                        if ui.add(egui::DragValue::new(&mut self.overlay_options.chip_y).range(0.0..=1000.0).speed(1.0)).changed() {
                            overlay_changed = true;
                        }

                        if ui.button(egui::RichText::new("↺ Reset").size(10.5).color(md3::PRIMARY))
                            .on_hover_text("Reset chip to standard ISO card position and size")
                            .clicked()
                        {
                            self.overlay_options.chip_x = 188.0;
                            self.overlay_options.chip_y = 398.0;
                            self.overlay_options.chip_scale = 1.0;
                            overlay_changed = true;
                        }
                    });
                }

                // Row 4: Card Details Inputs (when show_details is true)
                if self.overlay_options.details.show_details {
                    ui.add_space(4.0);
                    egui::Frame::NONE
                        .fill(md3::SURFACE_CONTAINER_HIGH)
                        .corner_radius(6.0)
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Style:").size(11.0).color(md3::ON_SURFACE_VARIANT));
                                let cur_emboss = self.overlay_options.details.emboss_style;
                                egui::ComboBox::from_id_salt("emboss_style_select")
                                    .width(125.0)
                                    .selected_text(egui::RichText::new(cur_emboss.display_name()).size(11.0).color(md3::ON_SURFACE))
                                    .show_ui(ui, |ui| {
                                        for style in [
                                            EmbossStyle::EmbossedSilver,
                                            EmbossStyle::EmbossedGold,
                                            EmbossStyle::CrispWhite,
                                            EmbossStyle::StealthDark,
                                        ] {
                                            let is_sel = cur_emboss == style;
                                            if ui.selectable_label(is_sel, style.display_name()).clicked() {
                                                if self.overlay_options.details.emboss_style != style {
                                                    self.overlay_options.details.emboss_style = style;
                                                    overlay_changed = true;
                                                }
                                            }
                                        }
                                    });

                                ui.label(egui::RichText::new("Backdrop:").size(11.0).color(md3::ON_SURFACE_VARIANT));
                                let cur_backdrop = self.overlay_options.details.backdrop;
                                egui::ComboBox::from_id_salt("text_backdrop_select")
                                    .width(135.0)
                                    .selected_text(egui::RichText::new(cur_backdrop.display_name()).size(11.0).color(md3::ON_SURFACE))
                                    .show_ui(ui, |ui| {
                                        for b in [
                                            TextBackdropStyle::None,
                                            TextBackdropStyle::FrostedGlassStrip,
                                            TextBackdropStyle::SubtleDarkGradient,
                                            TextBackdropStyle::FrostedPills,
                                        ] {
                                            let is_sel = cur_backdrop == b;
                                            if ui.selectable_label(is_sel, b.display_name()).clicked() {
                                                if self.overlay_options.details.backdrop != b {
                                                    self.overlay_options.details.backdrop = b;
                                                    overlay_changed = true;
                                                }
                                            }
                                        }
                                    });

                                ui.label(egui::RichText::new("Y-Pos:").size(11.0).color(md3::ON_SURFACE_VARIANT));
                                if ui.add(egui::DragValue::new(&mut self.overlay_options.details.vertical_offset).range(-80..=80).speed(1.0)).changed() {
                                    overlay_changed = true;
                                }
                            });

                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Card Number:").size(11.0).color(md3::ON_SURFACE_VARIANT));
                                if ui.add(egui::TextEdit::singleline(&mut self.overlay_options.details.card_number).desired_width(140.0)).changed() {
                                    overlay_changed = true;
                                }

                                ui.label(egui::RichText::new("Valid Thru:").size(11.0).color(md3::ON_SURFACE_VARIANT));
                                if ui.add(egui::TextEdit::singleline(&mut self.overlay_options.details.card_expiry).desired_width(48.0)).changed() {
                                    overlay_changed = true;
                                }
                            });

                            ui.add_space(3.0);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Cardholder:").size(11.0).color(md3::ON_SURFACE_VARIANT));
                                if ui.add(egui::TextEdit::singleline(&mut self.overlay_options.details.card_holder).desired_width(135.0)).changed() {
                                    overlay_changed = true;
                                }

                                ui.label(egui::RichText::new("Bank / Title:").size(11.0).color(md3::ON_SURFACE_VARIANT));
                                if ui.add(egui::TextEdit::singleline(&mut self.overlay_options.details.card_type_or_bank).desired_width(95.0)).changed() {
                                    overlay_changed = true;
                                }
                            });
                        });
                }

                if overlay_changed {
                    self.recompute_skin(ctx);
                }

                ui.add_space(16.0);

                // Apply
                ui.label(egui::RichText::new("Write to iPhone").strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);

                let can_flash = !self.is_busy && self.selected_udid.is_some() && !self.card_hash.trim().is_empty() && self.skin.is_some();
                let flash_btn = egui::Button::new(
                    egui::RichText::new("Apply Card Skin").strong().size(14.0)
                        .color(if can_flash { md3::ON_PRIMARY } else { md3::ON_SURFACE_VARIANT }),
                )
                .fill(if can_flash { md3::PRIMARY } else { md3::SURFACE_CONTAINER_HIGH })
                .corner_radius(20).stroke(egui::Stroke::NONE)
                .min_size(egui::vec2(ui.available_width(), 40.0));

                let resp = ui.add_enabled(can_flash, flash_btn);
                if resp.clicked() { self.flash_card(); }
                if !can_flash {
                    let mut r = Vec::new();
                    if self.selected_udid.is_none() { r.push("connect iPhone"); }
                    if self.card_hash.trim().is_empty() { r.push("enter card hash"); }
                    if self.skin.is_none() { r.push("choose image"); }
                    if !r.is_empty() { resp.on_disabled_hover_text(format!("Need: {}", r.join(", "))); }
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
                ui.label(egui::RichText::new("Wallet Preview").strong().size(16.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new("1536 x 969 px pass canvas").size(12.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(8.0);

                // Multi-layer control: Layer Selector Bar
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Layer:").strong().size(11.0).color(md3::ON_SURFACE));
                    for (layer, icon_label) in [
                        (ActiveTransformLayer::Background, "🖼️ Background"),
                        (ActiveTransformLayer::Finish, "✨ Finish"),
                        (ActiveTransformLayer::Chip, "💳 Chip"),
                        (ActiveTransformLayer::Logo, "🏷️ Logo"),
                    ] {
                        let is_active = self.active_layer == layer;
                        let btn = egui::Button::new(
                            egui::RichText::new(icon_label)
                                .size(11.0)
                                .color(if is_active { md3::ON_PRIMARY_CONTAINER } else { md3::ON_SURFACE_VARIANT })
                                .strong(),
                        )
                        .fill(if is_active { md3::PRIMARY_CONTAINER } else { egui::Color32::TRANSPARENT })
                        .corner_radius(12)
                        .stroke(egui::Stroke::new(1.0_f32, if is_active { md3::PRIMARY } else { md3::OUTLINE_VARIANT }));

                        if ui.add(btn).clicked() {
                            self.active_layer = layer;
                        }
                    }
                });

                ui.add_space(10.0);

                let pass_w = (ui.available_width() - 8.0).clamp(250.0, 400.0);
                let pass_h = pass_w * (969.0 / 1536.0);
                let corner_r = pass_w * (58.0 / 1536.0);
                let mut preview_changed = false;

                ui.vertical_centered(|ui| {
                    let (rect, response) = ui.allocate_exact_size(egui::vec2(pass_w, pass_h), egui::Sense::click_and_drag());
                    let canvas_w = 1536.0_f32;
                    let canvas_to_preview = pass_w / canvas_w;

                    // Direct hit-testing: click directly on chip or logo to select that layer
                    if let Some(pos) = response.hover_pos() {
                        if rect.contains(pos) && (response.clicked() || response.drag_started()) {
                            let cx = (pos.x - rect.left()) / canvas_to_preview;
                            let cy = (pos.y - rect.top()) / canvas_to_preview;

                            let chip_w = 205.0 * self.overlay_options.chip_scale;
                            let chip_h = 155.0 * self.overlay_options.chip_scale;
                            let chip_rect = egui::Rect::from_min_size(
                                egui::pos2(self.overlay_options.chip_x, self.overlay_options.chip_y),
                                egui::vec2(chip_w, chip_h),
                            );

                            let logo_w = 245.0 * self.overlay_options.logo_scale;
                            let logo_h = 94.0 * self.overlay_options.logo_scale;
                            let logo_rect = egui::Rect::from_min_size(
                                egui::pos2(self.overlay_options.logo_x, self.overlay_options.logo_y),
                                egui::vec2(logo_w, logo_h),
                            );

                            if self.overlay_options.show_chip && chip_rect.contains(egui::pos2(cx, cy)) {
                                self.active_layer = ActiveTransformLayer::Chip;
                            } else if self.overlay_options.network != PaymentNetwork::None && logo_rect.contains(egui::pos2(cx, cy)) {
                                self.active_layer = ActiveTransformLayer::Logo;
                            } else if self.overlay_options.finish == CardFinish::CustomTexture && self.active_layer == ActiveTransformLayer::Finish {
                                // Keep finish selected if clicking empty background while finish layer is active
                            } else if self.active_layer != ActiveTransformLayer::Finish {
                                self.active_layer = ActiveTransformLayer::Background;
                            }
                        }
                    }

                    if response.dragged_by(egui::PointerButton::Primary) {
                        let delta = response.drag_delta();
                        if delta.x != 0.0 || delta.y != 0.0 {
                            let scale_factor = 1536.0 / pass_w;
                            match self.active_layer {
                                ActiveTransformLayer::Background => {
                                    self.overlay_options.transform.pan_x += delta.x * scale_factor;
                                    self.overlay_options.transform.pan_y += delta.y * scale_factor;
                                }
                                ActiveTransformLayer::Finish => {
                                    self.overlay_options.finish_x += delta.x * scale_factor;
                                    self.overlay_options.finish_y += delta.y * scale_factor;
                                }
                                ActiveTransformLayer::Chip => {
                                    self.overlay_options.chip_x += delta.x * scale_factor;
                                    self.overlay_options.chip_y += delta.y * scale_factor;
                                }
                                ActiveTransformLayer::Logo => {
                                    self.overlay_options.logo_x += delta.x * scale_factor;
                                    self.overlay_options.logo_y += delta.y * scale_factor;
                                }
                            }
                            preview_changed = true;
                        }
                    }

                    if response.hovered() {
                        let scroll_y = ui.input(|i| {
                            if i.smooth_scroll_delta.y != 0.0 {
                                i.smooth_scroll_delta.y
                            } else {
                                i.raw_scroll_delta.y
                            }
                        });
                        if scroll_y != 0.0 {
                            let factor = if scroll_y > 0.0 { 1.08 } else { 1.0 / 1.08 };
                            match self.active_layer {
                                ActiveTransformLayer::Background => {
                                    let old_zoom = self.overlay_options.transform.zoom;
                                    let new_zoom = (old_zoom * factor).clamp(0.05, 50.0);
                                    if (new_zoom - old_zoom).abs() > 0.001 {
                                        self.overlay_options.transform.zoom = new_zoom;
                                        preview_changed = true;
                                    }
                                }
                                ActiveTransformLayer::Finish => {
                                    let old_scale = self.overlay_options.finish_scale;
                                    let new_scale = (old_scale * factor).clamp(0.1, 10.0);
                                    if (new_scale - old_scale).abs() > 0.001 {
                                        self.overlay_options.finish_scale = new_scale;
                                        preview_changed = true;
                                    }
                                }
                                ActiveTransformLayer::Chip => {
                                    let old_scale = self.overlay_options.chip_scale;
                                    let new_scale = (old_scale * factor).clamp(0.2, 5.0);
                                    if (new_scale - old_scale).abs() > 0.001 {
                                        self.overlay_options.chip_scale = new_scale;
                                        preview_changed = true;
                                    }
                                }
                                ActiveTransformLayer::Logo => {
                                    let old_scale = self.overlay_options.logo_scale;
                                    let new_scale = (old_scale * factor).clamp(0.2, 5.0);
                                    if (new_scale - old_scale).abs() > 0.001 {
                                        self.overlay_options.logo_scale = new_scale;
                                        preview_changed = true;
                                    }
                                }
                            }
                        }
                    }

                    if response.dragged() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                        ui.ctx().request_repaint();
                    } else if response.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                    }

                    let painter = ui.painter();
                    if let Some(tex) = self.skin_texture.as_ref() {
                        // Ambient card drop shadows matching card corner radius
                        painter.rect_filled(rect.translate(egui::vec2(2.0, 5.0)), corner_r, egui::Color32::from_black_alpha(100));
                        painter.rect_filled(rect.translate(egui::vec2(1.0, 2.0)), corner_r, egui::Color32::from_black_alpha(60));

                        painter.image(tex.id(), rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE);
                        painter.rect_stroke(rect, corner_r,
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_premultiplied(255, 255, 255, 45)),
                            egui::StrokeKind::Inside);

                        // Visual Figma-style bounding box indicators for the active layer
                        match self.active_layer {
                            ActiveTransformLayer::Chip if self.overlay_options.show_chip => {
                                let cx = rect.left() + self.overlay_options.chip_x * canvas_to_preview;
                                let cy = rect.top() + self.overlay_options.chip_y * canvas_to_preview;
                                let cw = 205.0 * self.overlay_options.chip_scale * canvas_to_preview;
                                let ch = 155.0 * self.overlay_options.chip_scale * canvas_to_preview;
                                let chip_box = egui::Rect::from_min_size(egui::pos2(cx, cy), egui::vec2(cw, ch));
                                painter.rect_stroke(
                                    chip_box,
                                    4.0,
                                    egui::Stroke::new(1.5_f32, md3::PRIMARY),
                                    egui::StrokeKind::Outside,
                                );
                            }
                            ActiveTransformLayer::Logo if self.overlay_options.network != PaymentNetwork::None => {
                                let lx = rect.left() + self.overlay_options.logo_x * canvas_to_preview;
                                let ly = rect.top() + self.overlay_options.logo_y * canvas_to_preview;
                                let lw = 245.0 * self.overlay_options.logo_scale * canvas_to_preview;
                                let lh = 94.0 * self.overlay_options.logo_scale * canvas_to_preview;
                                let logo_box = egui::Rect::from_min_size(egui::pos2(lx, ly), egui::vec2(lw, lh));
                                painter.rect_stroke(
                                    logo_box,
                                    4.0,
                                    egui::Stroke::new(1.5_f32, md3::PRIMARY),
                                    egui::StrokeKind::Outside,
                                );
                            }
                            ActiveTransformLayer::Finish if self.overlay_options.finish == CardFinish::CustomTexture => {
                                painter.rect_stroke(
                                    rect,
                                    corner_r,
                                    egui::Stroke::new(2.0_f32, md3::PRIMARY),
                                    egui::StrokeKind::Inside,
                                );
                            }
                            _ => {}
                        }
                    } else {
                        painter.rect_filled(rect.translate(egui::vec2(1.0, 3.0)), corner_r, egui::Color32::from_black_alpha(50));
                        painter.rect_filled(rect, corner_r, md3::SURFACE_CONTAINER_HIGH);
                        painter.text(rect.center(), egui::Align2::CENTER_CENTER,
                            "No artwork loaded", egui::FontId::proportional(14.0), md3::ON_SURFACE_VARIANT);
                    }
                });

                if preview_changed {
                    self.recompute_skin(ctx);
                }

                ui.add_space(6.0);
                let layer_hint = match self.active_layer {
                    ActiveTransformLayer::Background => "Active Layer: Background (Drag card to pan • Scroll to zoom)",
                    ActiveTransformLayer::Finish => "Active Layer: Finish Texture (Drag card to pan • Scroll to zoom)",
                    ActiveTransformLayer::Chip => "Active Layer: EMV Chip (Drag to reposition • Scroll to resize)",
                    ActiveTransformLayer::Logo => "Active Layer: Brand Logo (Drag to reposition • Scroll to resize)",
                };
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("💡 {}", layer_hint)).size(10.5).color(md3::PRIMARY));
                });

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("1536x969").size(11.0).color(md3::ON_SURFACE_VARIANT));
                    ui.label(egui::RichText::new("|").size(11.0).color(md3::OUTLINE_VARIANT));
                    ui.label(egui::RichText::new("1.585 ratio").size(11.0).color(md3::ON_SURFACE_VARIANT));
                    ui.label(egui::RichText::new("|").size(11.0).color(md3::OUTLINE_VARIANT));
                    if self.skin.is_some() {
                        ui.label(egui::RichText::new("Ready").size(11.0).color(md3::SUCCESS));
                    } else {
                        ui.label(egui::RichText::new("No image").size(11.0).color(md3::ON_SURFACE_VARIANT));
                    }
                });
                ui.add_space(8.0);
                ui.label(egui::RichText::new("After applying, force close Apple Wallet and reopen it.").size(11.0).color(md3::ON_SURFACE_VARIANT));
            });
        });
    }

    fn show_passcode_tab(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.columns(2, |cols| {
            // Left: config
            let left = &mut cols[0];
            m3_card(left, |ui| {
                ui.label(egui::RichText::new("Passcode Theme").strong().size(16.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Custom lockscreen keypad from Cowabunga or Nugget").size(12.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(16.0);

                // Theme file
                ui.label(egui::RichText::new("Theme Package").strong().size(12.0).color(md3::ON_SURFACE));
                ui.label(egui::RichText::new("Choose a .passthm archive containing dialer artwork").size(11.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(4.0);
                if m3_button_filled(ui, "Choose .passthm...") { self.select_theme_file(ctx); }

                if let Some(theme) = &self.loaded_theme {
                    let fname = self.theme_path.as_ref()
                        .and_then(|p| p.file_name()).and_then(|n| n.to_str()).unwrap_or("theme.passthm");
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(format!("{} - {} assets", fname, theme.items.len())).size(11.0).color(md3::PRIMARY));
                }

                ui.add_space(16.0);

                // iOS version
                ui.label(egui::RichText::new("Target iOS Cache").strong().size(12.0).color(md3::ON_SURFACE));
                ui.label(egui::RichText::new("Select cache format based on connected iOS version").size(11.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(4.0);
                let combo_w = (ui.available_width() - 4.0).max(150.0);
                let mut ver_changed = false;
                egui::ComboBox::from_id_salt("telephony_combo")
                    .width(combo_w)
                    .selected_text(&self.forced_telephony_ver)
                    .show_ui(ui, |ui| {
                        ver_changed |= ui.selectable_value(&mut self.forced_telephony_ver, "Auto (TelephonyUI-10)".into(), "Auto (TelephonyUI-10)").clicked();
                        ver_changed |= ui.selectable_value(&mut self.forced_telephony_ver, "TelephonyUI-10".into(), "TelephonyUI-10 (iOS 18+)").clicked();
                        ver_changed |= ui.selectable_value(&mut self.forced_telephony_ver, "TelephonyUI-9".into(), "TelephonyUI-9 (iOS 16-17)").clicked();
                        ver_changed |= ui.selectable_value(&mut self.forced_telephony_ver, "TelephonyUI-8".into(), "TelephonyUI-8 (Legacy)").clicked();
                    });

                if ver_changed {
                    if let Some(path) = self.theme_path.clone() {
                        self.load_theme_from_path(ctx, &path);
                    }
                }

                ui.add_space(16.0);

                // Keypad Language
                ui.label(egui::RichText::new("Keypad Language").strong().size(12.0).color(md3::ON_SURFACE));
                ui.label(egui::RichText::new("Subtext alphabet layout (English, Russian, Ukrainian, Japanese, or Universal)").size(11.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(4.0);
                let mut lang_changed = false;
                egui::ComboBox::from_id_salt("keypad_lang_combo")
                    .width(combo_w)
                    .selected_text(egui::RichText::new(&self.keypad_language).color(md3::ON_SURFACE))
                    .show_ui(ui, |ui| {
                        lang_changed |= ui.selectable_value(&mut self.keypad_language, "English".into(), "English").clicked();
                        lang_changed |= ui.selectable_value(&mut self.keypad_language, "Russian".into(), "Russian").clicked();
                        lang_changed |= ui.selectable_value(&mut self.keypad_language, "Ukrainian".into(), "Ukrainian").clicked();
                        lang_changed |= ui.selectable_value(&mut self.keypad_language, "Japanese".into(), "Japanese").clicked();
                        lang_changed |= ui.selectable_value(&mut self.keypad_language, "All Languages (Universal)".into(), "All Languages (Universal)").clicked();
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
                    if ui.checkbox(&mut self.passcode_bold, egui::RichText::new("Bold Text (iOS Accessibility)").strong().size(12.0).color(md3::ON_SURFACE)).changed() {
                        bold_changed = true;
                    }
                });
                ui.label(
                    egui::RichText::new("Generates *-bold.png for devices with Bold Text turned ON in iPhone Settings -> Display")
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
                ui.label(egui::RichText::new("Write to iPhone").strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);

                let can_flash = !self.is_busy && self.selected_udid.is_some() && self.loaded_theme.is_some();
                let flash_btn = egui::Button::new(
                    egui::RichText::new("Apply Passcode Theme").strong().size(14.0)
                        .color(if can_flash { md3::ON_PRIMARY } else { md3::ON_SURFACE_VARIANT }),
                )
                .fill(if can_flash { md3::PRIMARY } else { md3::SURFACE_CONTAINER_HIGH })
                .corner_radius(20).stroke(egui::Stroke::NONE)
                .min_size(egui::vec2(ui.available_width(), 40.0));

                let resp = ui.add_enabled(can_flash, flash_btn);
                if resp.clicked() { self.flash_theme(); }
                if !can_flash {
                    let mut r = Vec::new();
                    if self.selected_udid.is_none() { r.push("connect iPhone"); }
                    if self.loaded_theme.is_none() { r.push("select theme"); }
                    if !r.is_empty() { resp.on_disabled_hover_text(format!("Need: {}", r.join(", "))); }
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
                ui.label(egui::RichText::new("Keypad Preview").strong().size(16.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Dialer button artwork").size(12.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(12.0);

                let pass_w = (ui.available_width() - 8.0).clamp(240.0, 360.0);
                let pass_h = 265.0;

                ui.vertical_centered(|ui| {
                    if self.keypad_textures.is_empty() {
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(pass_w, pass_h), egui::Sense::hover());
                        let painter = ui.painter();
                        painter.rect_filled(rect, 16.0, md3::SURFACE_CONTAINER_HIGH);
                        painter.text(rect.center(), egui::Align2::CENTER_CENTER,
                            "No theme loaded", egui::FontId::proportional(14.0), md3::ON_SURFACE_VARIANT);
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
                    ui.label(egui::RichText::new("3x4 Keypad").size(11.0).color(md3::ON_SURFACE_VARIANT));
                    ui.label(egui::RichText::new("|").size(11.0).color(md3::OUTLINE_VARIANT));
                    ui.label(egui::RichText::new("TelephonyUI").size(11.0).color(md3::ON_SURFACE_VARIANT));
                    ui.label(egui::RichText::new("|").size(11.0).color(md3::OUTLINE_VARIANT));
                    if self.loaded_theme.is_some() {
                        ui.label(egui::RichText::new("Ready").size(11.0).color(md3::SUCCESS));
                    } else {
                        ui.label(egui::RichText::new("No theme").size(11.0).color(md3::ON_SURFACE_VARIANT));
                    }
                });
                ui.add_space(8.0);
                ui.label(egui::RichText::new("After applying, lock your iPhone to see the new keypad.").size(11.0).color(md3::ON_SURFACE_VARIANT));
            });
        });
    }

    fn show_help_tab(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            m3_card(left, |ui| {
                ui.label(egui::RichText::new("Setup & Card Hash Guide").strong().size(16.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Everything you need to connect and capture your card").size(12.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(16.0);

                ui.label(egui::RichText::new("Prerequisites").strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(6.0);
                ui.label(egui::RichText::new("- 64-bit iTunes or Apple Mobile Device Support installed").size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new("- Connect iPhone via USB-C or Lightning cable").size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new("- Unlock iPhone and tap \"Trust this Computer\"").size(11.5).color(md3::ON_SURFACE_VARIANT));

                ui.add_space(18.0);

                ui.label(egui::RichText::new("Finding Your Card Hash").strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(6.0);
                ui.label(egui::RichText::new("1. Click \"Scan\" in the Wallet tab").size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new("2. Open Apple Wallet on your iPhone").size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new("3. Tap the card you want to customize").size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new("4. AirCard captures the pass hash automatically").size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new("5. Click \"Stop\" once detected").size(11.5).color(md3::ON_SURFACE_VARIANT));
            });

            let right = &mut cols[1];
            m3_card(right, |ui| {
                ui.label(egui::RichText::new("Activation & Theme Guide").strong().size(16.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Applying skins and dialer keypad packages").size(12.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(16.0);

                ui.label(egui::RichText::new("Activating Apple Wallet Skin").strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(6.0);
                ui.label(egui::RichText::new("1. Click \"Apply Card Skin\" and wait for completion").size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new("2. Open App Switcher on iPhone (swipe up from bottom)").size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new("3. Force close Apple Wallet by swiping up on it").size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new("4. Reopen Wallet - your new skin appears!").size(11.5).color(md3::ON_SURFACE_VARIANT));

                ui.add_space(18.0);

                ui.label(egui::RichText::new("Passcode Themes (.passthm)").strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(6.0);
                ui.label(egui::RichText::new("- Compatible with Cowabunga & Nugget theme packages").size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new("- iOS 18+: Select \"Auto (TelephonyUI-10)\"").size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new("- iOS 16-17: Select \"TelephonyUI-9\"").size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new("- Lock screen to verify your updated keypad artwork").size(11.5).color(md3::ON_SURFACE_VARIANT));
            });
        });
    }

        fn show_sources_tab(&mut self, ui: &mut egui::Ui) {
        let is_vi = self.ui_language == UiLanguage::Vietnamese;
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            m3_card(left, |ui| {
                let title = if is_vi { "AIrCard-CMaku (Phiên bản Windows)" } else { "AIrCard-CMaku (Windows Edition)" };
                let subtitle = if is_vi {
                    "Bản mod tùy biến dành cho Windows của AirCard Apple Wallet & Passcode Studio"
                } else {
                    "Customized Native Windows port of the AirCard Apple Wallet & Passcode studio"
                };
                ui.label(egui::RichText::new(title).strong().size(16.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(subtitle).size(12.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(14.0);

                // Author & GitHub
                let orig_title = if is_vi { "Tác giả gốc & Kho lưu trữ chính thức" } else { "Original Author & Official Repository" };
                ui.label(egui::RichText::new(orig_title).strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    let author_lbl = if is_vi { "Tác giả:" } else { "Author:" };
                    ui.label(egui::RichText::new(author_lbl).size(11.5).color(md3::ON_SURFACE_VARIANT));
                    ui.label(egui::RichText::new("Lumid-Off").strong().size(11.5).color(md3::PRIMARY));
                });
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("GitHub:").size(11.5).color(md3::ON_SURFACE_VARIANT));
                    ui.hyperlink_to(
                        egui::RichText::new("Lumid-Off/AirCard-Windows").size(11.5).color(md3::PRIMARY).underline(),
                        "https://github.com/Lumid-Off/AirCard-Windows",
                    );
                });

                ui.add_space(14.0);

                // Customizer / Modder
                let mod_title = if is_vi { "Người tùy biến & Mod lại" } else { "Customized & Modded By" };
                ui.label(egui::RichText::new(mod_title).strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    let mod_lbl = if is_vi { "Người mod / Customizer:" } else { "Modder / Customizer:" };
                    ui.label(egui::RichText::new(mod_lbl).size(11.5).color(md3::ON_SURFACE_VARIANT));
                    ui.label(egui::RichText::new("SirMaku").strong().size(11.5).color(md3::PRIMARY));
                });
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("GitHub:").size(11.5).color(md3::ON_SURFACE_VARIANT));
                    ui.hyperlink_to(
                        egui::RichText::new("SirMaku-git").size(11.5).color(md3::PRIMARY).underline(),
                        "https://github.com/SirMaku-git",
                    );
                });
                ui.add_space(2.0);
                let mod_features = if is_vi {
                    "Mở rộng với Card Studio tương tác đa lớp, tùy biến chất liệu Foil/Texture, bộ engine gắn & định vị chip EMV, hệ thống căn chỉnh layer tự do và xem trước trực tiếp."
                } else {
                    "Enhanced with Interactive Multi-Layer Card Studio, Custom Texture/Foil overlays, Custom EMV Chip engine, layer alignment controls, and UI improvements."
                };
                ui.label(
                    egui::RichText::new(mod_features)
                        .size(11.0)
                        .color(md3::ON_SURFACE_VARIANT),
                );

                ui.add_space(12.0);

                // EXPLICIT SCOPE DISCLAIMER FRAME
                egui::Frame::new()
                    .fill(md3::SURFACE_CONTAINER)
                    .corner_radius(10)
                    .inner_margin(egui::Margin::same(10))
                    .stroke(egui::Stroke::new(1.0_f32, md3::PRIMARY))
                    .show(ui, |ui| {
                        let scope_header = if is_vi {
                            "⚠️ LƯU Ý QUAN TRỌNG VỀ PHẠM VI TÙY BIẾN:"
                        } else {
                            "⚠️ IMPORTANT CUSTOMIZATION SCOPE NOTICE:"
                        };
                        ui.label(
                            egui::RichText::new(scope_header)
                                .strong()
                                .size(11.5)
                                .color(md3::PRIMARY),
                        );
                        ui.add_space(4.0);
                        let scope_body = if is_vi {
                            "• SirMaku CHỈ tập trung tùy biến và mở rộng các tính năng liên quan đến Apple Pay / Apple Wallet (Card Studio).
• Tất cả các cơ chế kỹ thuật cốt lõi khác — bao gồm cơ chế exploit AirTraffic sync (airlift), kết nối thiết bị USB (usbmuxd), sao lưu phục hồi Books snapshot, và tính năng Passcode — KHÔNG bị can thiệp nhiều và hoàn toàn mang tính kế thừa nguyên bản từ dự án gốc của tác giả @Lumid-Off và các tác giả tiền nhiệm (@mak5er, 0xjohnny)."
                        } else {
                            "• SirMaku ONLY customized and extended features related to Apple Pay / Apple Wallet (Card Studio).
• All other core technical mechanisms — including the AirTraffic sync exploit (airlift), USB usbmuxd communication, Books snapshot restore, and Passcode theming — were NOT heavily modified and are directly inherited from the original project by @Lumid-Off and upstream authors (@mak5er, 0xjohnny)."
                        };
                        ui.label(
                            egui::RichText::new(scope_body)
                                .size(10.5)
                                .color(md3::ON_SURFACE),
                        );
                    });

                ui.add_space(14.0);

                // Open-source license & attribution notice
                egui::Frame::new()
                    .fill(md3::SURFACE_CONTAINER_HIGH)
                    .corner_radius(12)
                    .inner_margin(egui::Margin::same(12))
                    .stroke(egui::Stroke::new(1.0_f32, md3::OUTLINE))
                    .show(ui, |ui| {
                        let notice_header = if is_vi {
                            "📜 Quy định Bản quyền & Ghi công Mã nguồn mở"
                        } else {
                            "📜 Open Source & Attribution Notice"
                        };
                        ui.label(
                            egui::RichText::new(notice_header)
                                .strong()
                                .size(11.5)
                                .color(md3::PRIMARY),
                        );
                        ui.add_space(6.0);
                        let notice_rules = if is_vi {
                            "• Đây là dự án mã nguồn mở, hoàn toàn miễn phí sử dụng (free to use / for fun).
• Người dùng có quyền tự do sử dụng, chỉnh sửa và đóng góp cho mã nguồn.
• QUY ĐỊNH BẮT BUỘC: Khi sử dụng, tùy biến hoặc phân phối lại dự án này, BẮT BUỘC phải giữ và ghi nhận đầy đủ tên tác giả gốc (@Lumid-Off) và người tùy biến/chỉnh sửa (@SirMaku).
• Dự án có sự hỗ trợ tham khảo và tinh chỉnh từ AI (Google Antigravity / Gemini)."
                        } else {
                            "• This is an open-source, non-commercial (for fun) project, free to use.
• Users are free to use, modify, and contribute to the source code.
• MANDATORY REQUIREMENT: Any redistribution or modification MUST retain full attribution to both the original author (@Lumid-Off) and customizer (@SirMaku).
• AI Assistance: AI (Google Antigravity / Gemini) was utilized during modding and customization."
                        };
                        ui.label(
                            egui::RichText::new(notice_rules)
                                .size(10.5)
                                .color(md3::ON_SURFACE),
                        );
                    });
            });

            let right = &mut cols[1];
            m3_card(right, |ui| {
                let r_title = if is_vi { "Người đóng góp & Lời cảm ơn" } else { "Contributors & Credits" };
                let r_subtitle = if is_vi {
                    "Ghi nhận công lao của các tác giả và nhà nghiên cứu bảo mật"
                } else {
                    "Credits to the original creators and security researchers"
                };
                ui.label(egui::RichText::new(r_title).strong().size(16.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(r_subtitle).size(12.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(14.0);

                let contrib_hdr = if is_vi { "Người đóng góp" } else { "Contributors" };
                ui.label(egui::RichText::new(contrib_hdr).strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(6.0);

                // Lumid-Off
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("• @Lumid-Off").strong().size(11.5).color(md3::ON_SURFACE));
                    ui.label(egui::RichText::new(if is_vi { "(Windows Native Rust Port & Duy trì)" } else { "(Windows Native Rust Port & Maintainer)" }).size(11.0).color(md3::ON_SURFACE_VARIANT));
                });
                ui.horizontal(|ui| {
                    ui.add_space(14.0);
                    ui.hyperlink_to(egui::RichText::new("GitHub").size(11.0).color(md3::PRIMARY).underline(), "https://github.com/Lumid-Off");
                    ui.label(egui::RichText::new("·").size(11.0).color(md3::OUTLINE_VARIANT));
                    ui.hyperlink_to(egui::RichText::new("Twitter / X").size(11.0).color(md3::PRIMARY).underline(), "https://x.com/LumidOff");
                });

                ui.add_space(8.0);

                // mak5er
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("• @mak5er").strong().size(11.5).color(md3::ON_SURFACE));
                    ui.label(egui::RichText::new(if is_vi { "(Ứng dụng macOS gốc & Nghiên cứu exploit)" } else { "(Original macOS App & Exploit Research)" }).size(11.0).color(md3::ON_SURFACE_VARIANT));
                });
                ui.horizontal(|ui| {
                    ui.add_space(14.0);
                    ui.hyperlink_to(egui::RichText::new("GitHub").size(11.0).color(md3::PRIMARY).underline(), "https://github.com/mak5er");
                    ui.label(egui::RichText::new("·").size(11.0).color(md3::OUTLINE_VARIANT));
                    ui.hyperlink_to(egui::RichText::new("Twitter / X").size(11.0).color(md3::PRIMARY).underline(), "https://x.com/mak5er");
                });

                ui.add_space(8.0);

                // AirLift / 0xjohnny
                ui.horizontal(|ui| {
                    ui.hyperlink_to(egui::RichText::new("• AirLift").strong().size(11.5).color(md3::PRIMARY).underline(), "https://github.com/0xjohnnydev/airlift");
                    ui.label(egui::RichText::new(if is_vi { "bởi" } else { "by" }).size(11.0).color(md3::ON_SURFACE_VARIANT));
                    ui.hyperlink_to(egui::RichText::new("0xjohnny (@0xjohnnydev)").strong().size(11.0).color(md3::PRIMARY).underline(), "https://github.com/0xjohnnydev");
                });
                let poc_desc = if is_vi {
                    "  Mã nguồn gốc vượt sandbox AirTraffic/ATAirlock nền tảng cho AirliftFFI."
                } else {
                    "  Original AirTraffic/ATAirlock sandbox escape and proof of concept underlying AirliftFFI."
                };
                ui.label(
                    egui::RichText::new(poc_desc)
                        .size(10.5)
                        .color(md3::ON_SURFACE_VARIANT),
                );

                ui.add_space(16.0);

                let credits_hdr = if is_vi { "Ghi nhận khác" } else { "Credits" };
                ui.label(egui::RichText::new(credits_hdr).strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(6.0);
                let cred1 = if is_vi {
                    "• Exploit cốt lõi dựa trên airlift (vượt cơ chế đồng bộ AirTraffic)."
                } else {
                    "• Core exploit based on airlift (AirTraffic sync escape)."
                };
                ui.label(egui::RichText::new(cred1).size(11.0).color(md3::ON_SURFACE_VARIANT));
                ui.horizontal(|ui| {
                    let cred2 = if is_vi { "• Định dạng theme lấy cảm hứng từ" } else { "• Theme format inspired by" };
                    ui.label(egui::RichText::new(cred2).size(11.0).color(md3::ON_SURFACE_VARIANT));
                    ui.hyperlink_to(egui::RichText::new("Cowabunga").size(11.0).color(md3::PRIMARY).underline(), "https://github.com/leminlimez/Cowabunga");
                    ui.label(egui::RichText::new(if is_vi { "và" } else { "and" }).size(11.0).color(md3::ON_SURFACE_VARIANT));
                    ui.hyperlink_to(egui::RichText::new("Nugget").size(11.0).color(md3::PRIMARY).underline(), "https://github.com/leminlimez/Nugget");
                    ui.label(egui::RichText::new(".").size(11.0).color(md3::ON_SURFACE_VARIANT));
                });
            });
        });
    }
}
