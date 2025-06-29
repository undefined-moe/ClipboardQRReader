use arboard::Clipboard;
use anyhow::Result;
use tracing::{debug, warn};

pub struct ClipboardHandler {
    clipboard: Option<Clipboard>,
}

impl ClipboardHandler {
    pub fn new() -> Self {
        let clipboard = match Clipboard::new() {
            Ok(clipboard) => {
                debug!("Clipboard initialized successfully");
                Some(clipboard)
            },
            Err(e) => {
                warn!("Failed to initialize clipboard: {}", e);
                None
            },
        };

        Self { clipboard }
    }

    pub fn get_text(&mut self) -> Result<String> {
        match &mut self.clipboard {
            Some(clipboard) => {
                match clipboard.get_text() {
                    Ok(text) => {
                        debug!("Successfully read text from clipboard");
                        Ok(text)
                    },
                    Err(e) => {
                        warn!("Failed to read text from clipboard: {}", e);
                        Err(anyhow::anyhow!("Failed to read clipboard text: {}", e))
                    },
                }
            },
            None => {
                Err(anyhow::anyhow!("Clipboard not available"))
            },
        }
    }

    pub fn set_text(&mut self, text: &str) -> Result<()> {
        match &mut self.clipboard {
            Some(clipboard) => {
                match clipboard.set_text(text) {
                    Ok(()) => {
                        debug!("Successfully set text to clipboard");
                        Ok(())
                    },
                    Err(e) => {
                        warn!("Failed to set text to clipboard: {}", e);
                        Err(anyhow::anyhow!("Failed to set clipboard text: {}", e))
                    },
                }
            },
            None => {
                Err(anyhow::anyhow!("Clipboard not available"))
            },
        }
    }

    pub fn is_available(&self) -> bool {
        self.clipboard.is_some()
    }
} 