use log::error;
use sdl2::messagebox::{show_simple_message_box, MessageBoxFlag};
// use tracing::error;

pub fn show_error(message: &str) {
    if let Err(err) = show_simple_message_box(MessageBoxFlag::ERROR, "Error", message, None){
        error!("{}", err);
    }
}
