use arboard::Clipboard;
use anyhow::Result;
use tracing::{debug, warn, info};
use std::time::SystemTime;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use image::{ImageBuffer, Rgba};

use crate::qr_scanner::QRScanner;

#[derive(Debug, Clone)]
pub enum ClipboardData {
    Text(String),
    Image(ImageBuffer<Rgba<u8>, Vec<u8>>),
    Empty,
}

pub struct ClipboardHandler {
    clipboard: Option<Clipboard>,
    qr_scanner: QRScanner,
    last_hash: u64,
    last_check_time: SystemTime,
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

        Self { 
            clipboard,
            qr_scanner: QRScanner::new(),
            last_hash: 0,
            last_check_time: SystemTime::now(),
        }
    }

    pub fn get_data(&mut self) -> Result<ClipboardData> {
        match &mut self.clipboard {
            Some(clipboard) => {
                // Try to get image first
                if let Ok(image) = clipboard.get_image() {
                    debug!("Successfully read image from clipboard");
                    let img_buffer = ImageBuffer::from_raw(
                        image.width as u32,
                        image.height as u32,
                        image.bytes.into_owned(),
                    ).ok_or_else(|| anyhow::anyhow!("Failed to create image buffer"))?;
                    
                    return Ok(ClipboardData::Image(img_buffer));
                }
                
                // Try to get text
                match clipboard.get_text() {
                    Ok(text) => {
                        debug!("Successfully read text from clipboard");
                        if text.is_empty() {
                            Ok(ClipboardData::Empty)
                        } else {
                            Ok(ClipboardData::Text(text))
                        }
                    },
                    Err(e) => {
                        warn!("Failed to read text from clipboard: {}", e);
                        Ok(ClipboardData::Empty)
                    },
                }
            },
            None => {
                Err(anyhow::anyhow!("Clipboard not available"))
            },
        }
    }

    pub fn get_text(&mut self) -> Result<String> {
        match self.get_data()? {
            ClipboardData::Text(text) => Ok(text),
            ClipboardData::Image(_) => Ok("".to_string()),
            ClipboardData::Empty => Ok("".to_string()),
        }
    }

    pub fn detect_qr_in_image(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<Option<String>> {
        info!("Attempting to detect QR code in clipboard image");
        self.qr_scanner.scan_qr_from_rgba(image)
    }

    pub fn detect_multiple_qr_codes(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<Vec<String>> {
        info!("Attempting to detect multiple QR codes in clipboard image");
        self.qr_scanner.scan_multiple_qr_codes(image)
    }

    pub fn has_qr_code(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<bool> {
        self.qr_scanner.has_qr_code(image)
    }

    pub fn has_changed(&mut self) -> Result<bool> {
        let current_data = self.get_data()?;
        let mut hasher = DefaultHasher::new();
        
        match &current_data {
            ClipboardData::Text(text) => text.hash(&mut hasher),
            ClipboardData::Image(image) => {
                // Hash the image dimensions and first few pixels for change detection
                (image.width(), image.height()).hash(&mut hasher);
                if let Some(pixel) = image.get_pixel_checked(0, 0) {
                    pixel.hash(&mut hasher);
                }
            },
            ClipboardData::Empty => "empty".hash(&mut hasher),
        }
        
        let current_hash = hasher.finish();
        let changed = current_hash != self.last_hash;
        
        if changed {
            self.last_hash = current_hash;
            self.last_check_time = SystemTime::now();
        }
        
        Ok(changed)
    }

    pub fn get_data_if_changed(&mut self) -> Result<Option<ClipboardData>> {
        if self.has_changed()? {
            Ok(Some(self.get_data()?))
        } else {
            Ok(None)
        }
    }

    pub fn get_text_if_changed(&mut self) -> Result<Option<String>> {
        if self.has_changed()? {
            let data = self.get_data()?;
            match data {
                ClipboardData::Text(text) => Ok(Some(text)),
                ClipboardData::Image(image) => {
                    // Try to detect QR code in the image
                    if let Some(qr_content) = self.detect_qr_in_image(&image)? {
                        Ok(Some(qr_content))
                    } else {
                        Ok(Some("".to_string()))
                    }
                },
                ClipboardData::Empty => Ok(Some("".to_string())),
            }
        } else {
            Ok(None)
        }
    }

    pub fn set_text(&mut self, text: &str) -> Result<()> {
        match &mut self.clipboard {
            Some(clipboard) => {
                match clipboard.set_text(text) {
                    Ok(()) => {
                        debug!("Successfully set text to clipboard");
                        // Update hash to prevent immediate change detection
                        let mut hasher = DefaultHasher::new();
                        text.hash(&mut hasher);
                        self.last_hash = hasher.finish();
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

    pub fn get_last_check_time(&self) -> SystemTime {
        self.last_check_time
    }
} 