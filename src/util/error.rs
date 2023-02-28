use crossbeam::channel::{SendError};
// use tracing::error;
use log::error;
use std::{error::Error, fmt::Display};

use crate::{entity::EventMessage, global::EVENT_CHANNEL};

pub type SuperError = Box<dyn Error>;

/// Handle result function of methods, process exception if any, return content if there's no exception
pub fn handle_result<T>(result: Result<T, SuperError>) -> Option<T> {
    match result {
        Ok(t) => Some(t),
        Err(e) => {
            safe_send(EVENT_CHANNEL.0.send(EventMessage::ShowError(e.to_string())));
            None
        }
    }
}

/// Send event message, if any error occurred during sending, log the error
pub fn safe_send<T>(result: Result<(), SendError<T>>) {
    if let Err(err) = result {
        error!("{}", err);
    }
}

#[derive(Debug, Clone)]
pub struct CustomError{
    message: String,
}

impl CustomError {
    pub fn new<T:Into<String>>(message: T)->Self{
        Self { message: message.into() }
    }
}

impl Error for CustomError {}

impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}