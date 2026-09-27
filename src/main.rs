#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod afc;
mod airlift;
mod airtraffic;
mod app;
mod apple;
pub mod card_studio;
mod device;
mod flasher;
mod image_skin;
mod i18n;
mod passthm;
mod scanner;
mod wallet_backup;

fn main() -> eframe::Result<()> {
    let app_title = format!("AirCard-CMaku v{}", env!("CARGO_PKG_VERSION"));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([980.0, 640.0])
            .with_min_inner_size([880.0, 580.0])
            .with_title(&app_title),
        ..Default::default()
    };

    eframe::run_native(
        &app_title,
        options,
        Box::new(|cc| Ok(Box::new(app::AirCardApp::new(cc)))),
    )
}
