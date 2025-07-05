use anyhow::Result;
use tracing::{info, error};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[cfg(unix)]
use gtk;

mod qr_generator;
mod qr_scanner;
mod clipboard_handler;
mod global_state;
mod tray;

use clipboard_handler::ClipboardHandler;
use global_state::GlobalClipboardState;
use tray::SystemTray;
use qr_generator::QRGenerator;
use qr_scanner::QRScanner;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("Starting Clipboard QR Application");

    // Initialize GTK on Unix systems
    #[cfg(unix)]
    {
        if gtk::init().is_err() {
            error!("Failed to initialize GTK");
            return Err(anyhow::anyhow!("GTK initialization failed"));
        }
        info!("GTK initialized successfully");
    }

    // Create global clipboard state
    let clipboard_state = Arc::new(Mutex::new(GlobalClipboardState::new()));
    let clipboard_state_clone = clipboard_state.clone();

    // Create system tray
    let system_tray = match SystemTray::new(clipboard_state.clone()) {
        Ok(tray) => {
            info!("System tray created successfully");
            Some(tray)
        },
        Err(e) => {
            error!("Failed to create system tray: {}", e);
            None
        },
    };

    // Start background clipboard monitoring thread
    let _background_thread = thread::spawn(move || {
        let qr_generator = QRGenerator::new();
        let qr_scanner = QRScanner::new();
        let mut clipboard_handler = ClipboardHandler::new();
        info!("Background clipboard monitoring thread started");

        loop {
            // Check for clipboard changes
            match clipboard_handler.get_data_if_changed() {
                Ok(Some(new_data)) => {
                    // Update global state
                    if let Ok(mut state) = clipboard_state_clone.lock() {
                        state.last_data = Some(new_data.clone());
                        state.has_changed = true;
                    }
                    info!("Clipboard data updated in background thread");
                    
                    match &new_data {
                        crate::clipboard_handler::ClipboardData::Text(text) => {
                            println!("\n🔄 Clipboard text updated: {}", text);
                            println!("QR Code:");
                            if let Err(e) = qr_generator.print_qr_terminal(&text) {
                                println!("❌ Failed to generate QR code: {}", e);
                            }
                        },
                        crate::clipboard_handler::ClipboardData::Image(image) => {
                            println!("\n🔄 Clipboard image updated ({}x{})", image.width(), image.height());
                            println!("Scanning for QR codes...");
                            
                            match qr_scanner.scan_qr_from_rgba(&image) {
                                Ok(Some(content)) => {
                                    println!("✅ QR code detected in clipboard image!");
                                    println!("Content: {}", content);
                                    
                                    // Also display QR code for the detected content
                                    println!("QR Code for detected content:");
                                    if let Err(e) = qr_generator.print_qr_terminal(&content) {
                                        println!("❌ Failed to generate QR code: {}", e);
                                    }
                                },
                                Ok(None) => {
                                    println!("❌ No QR code found in clipboard image");
                                },
                                Err(e) => {
                                    println!("❌ Error scanning QR code: {}", e);
                                }
                            }
                        },
                        crate::clipboard_handler::ClipboardData::Empty => {
                            println!("\n🔄 Clipboard cleared");
                        },
                    }
                    
                }
                Ok(None) => {
                    // No change, continue monitoring
                }
                Err(e) => {
                    error!("Error checking clipboard: {}", e);
                }
            }

            // Sleep to avoid excessive CPU usage
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Main thread keeps running to handle tray events
    info!("Main thread running, waiting for tray events...");
    
    // Keep the system tray alive in the main thread
    loop {
        // Keep main thread alive and prevent tray icon from being dropped
        thread::sleep(Duration::from_secs(1));
    }
}
