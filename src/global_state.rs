use crate::clipboard_handler::ClipboardData;

// Global clipboard state shared between threads
#[derive(Clone)]
pub struct GlobalClipboardState {
    pub last_data: Option<ClipboardData>,
    pub last_hash: u64,
    pub has_changed: bool,
}

impl GlobalClipboardState {
    pub fn new() -> Self {
        Self {
            last_data: None,
            last_hash: 0,
            has_changed: false,
        }
    }
} 