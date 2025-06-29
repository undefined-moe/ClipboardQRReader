use crate::clipboard_handler::ClipboardData;

// Global clipboard state shared between threads
#[derive(Clone)]
pub struct GlobalClipboardState {
    pub last_data: Option<ClipboardData>,
    pub has_changed: bool,
}

impl GlobalClipboardState {
    pub fn new() -> Self {
        Self {
            last_data: None,
            has_changed: false,
        }
    }
} 