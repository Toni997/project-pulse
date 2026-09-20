use log::warn;
use tauri::Emitter;

use crate::{app_handle, core::constants::NOTIFICATION_ERROR_EVENT};

pub fn log_and_notify_error(message: &str) {
    warn!("{}", message);
    let _ = app_handle().emit(NOTIFICATION_ERROR_EVENT, message);
}
