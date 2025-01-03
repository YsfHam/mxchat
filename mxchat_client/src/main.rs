mod app;
mod auth_page;
mod chat_page;
mod gui_utils;
mod networking;
mod notifications_handler;

use app::ChatApp;
use eframe::egui::ViewportBuilder;

const WIDTH: f32 = 600.0;
const HEIGHT: f32 = 400.0;

fn main() {

    let size = (WIDTH, HEIGHT);
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
                .with_inner_size(size)
                .with_min_inner_size(size),

        ..Default::default()
    };

    eframe::run_native(
        "Chat Application",
         options,
          Box::new(|_| {
            Ok(Box::new(ChatApp::new()))
        }
    )).unwrap();

}