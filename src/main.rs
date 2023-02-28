// Hide console window on Windows platform, never remove it
// #![windows_subsystem = "windows"]

mod config;
mod entity;
mod filemanager;
mod global;
mod media;
mod app;
mod ui;
mod util;
mod sound;

use log::error;

use entity::EventMessage;
use app::NtApp;
use ui::components::dialog::show_error;
use util::{error::SuperError, log_builder};

// Four threads, one for decoding, one for playing audio,
// one for playing video, maint thread for rendering audio and video
fn main() -> Result<(), SuperError> {
    log_builder::load_logger(log::LevelFilter::Debug);

    match NtApp::new() {
        Ok(mut app) => {
            // The error occurred while app running, should be logged and shown
            if let Err(err) = app.run() {
                error!("{}", err);
                show_error(err.to_string().as_str());
            }
        }
        Err(err) => {
            error!("{}", err);
            show_error(err.to_string().as_str());
            // App init error, it makes no sence to continue
            panic!()
        }
    }

    Ok(())
}
