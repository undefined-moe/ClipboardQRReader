use anyhow::Result;
use tracing::error;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::qr_generator::QRGenerator;
use crate::qr_scanner::QRScanner;
use crate::clipboard_handler::ClipboardHandler;
use crate::global_state::GlobalClipboardState;

pub struct ClipboardQRCLI {
    qr_generator: QRGenerator,
    qr_scanner: QRScanner,
    clipboard_handler: ClipboardHandler,
    global_clipboard_state: Arc<Mutex<GlobalClipboardState>>,
    last_processed_hash: u64,
}

impl ClipboardQRCLI {
    pub fn new(global_clipboard_state: Arc<Mutex<GlobalClipboardState>>) -> Self {
        Self {
            qr_generator: QRGenerator::new(),
            qr_scanner: QRScanner::new(),
            clipboard_handler: ClipboardHandler::new(),
            global_clipboard_state,
            last_processed_hash: 0,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        println!("🔗 Clipboard QR Code Generator (Background Mode)");
        println!("===============================================");
        println!("Background clipboard monitoring is active.");
        println!("Clipboard changes will be automatically detected and displayed.");
        println!("Press Ctrl+C to exit.");
        
        loop {
            // Check for clipboard changes from background thread
            self.check_clipboard_updates()?;
            
            // Small delay to avoid excessive CPU usage
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    fn check_clipboard_updates(&mut self) -> Result<()> {
        if let Ok(mut state) = self.global_clipboard_state.lock() {
            if state.has_changed {
                state.has_changed = false;
                
                if let Some(data) = &state.last_data {
                    match data {
                        crate::clipboard_handler::ClipboardData::Text(text) => {
                            println!("\n🔄 Clipboard text updated: {}", text);
                            println!("QR Code:");
                            if let Err(e) = self.qr_generator.print_qr_terminal(text) {
                                println!("❌ Failed to generate QR code: {}", e);
                            }
                        },
                        crate::clipboard_handler::ClipboardData::Image(image) => {
                            println!("\n🔄 Clipboard image updated ({}x{})", image.width(), image.height());
                            println!("Scanning for QR codes...");
                            
                            match self.qr_scanner.scan_qr_from_rgba(image)? {
                                Some(content) => {
                                    println!("✅ QR code detected in clipboard image!");
                                    println!("Content: {}", content);
                                    
                                    // Also display QR code for the detected content
                                    println!("QR Code for detected content:");
                                    if let Err(e) = self.qr_generator.print_qr_terminal(&content) {
                                        println!("❌ Failed to generate QR code: {}", e);
                                    }
                                },
                                None => {
                                    println!("❌ No QR code found in clipboard image");
                                },
                            }
                        },
                        crate::clipboard_handler::ClipboardData::Empty => {
                            println!("\n🔄 Clipboard cleared");
                        },
                    }
                }
            }
        }
        Ok(())
    }
} 