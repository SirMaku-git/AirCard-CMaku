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
            .with_inner_size([1040.0, 700.0])
            .with_min_inner_size([850.0, 550.0])
            .with_title(&app_title),
        ..Default::default()
    };

    eframe::run_native(
        &app_title,
        options,
        Box::new(|cc| Ok(Box::new(app::AirCardApp::new(cc)))),
    )
}
