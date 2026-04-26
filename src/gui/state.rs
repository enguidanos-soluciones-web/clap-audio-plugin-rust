use std::sync::Mutex;
use wry::WebView;

use crate::processors::send_ui_event::GUIRequest;

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub struct GUIState {
    pub window: Option<WebView>,
    pub width: u32,
    pub height: u32,
    pub message_queue: Mutex<Vec<GUIRequest>>,
}
