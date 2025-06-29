use anyhow::Result;
use tracing::{info, error};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod qr_generator;
mod qr_scanner;
mod clipboard_handler;
mod cli;
mod global_state;

use cli::ClipboardQRCLI;
use clipboard_handler::ClipboardHandler;
use global_state::GlobalClipboardState;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("Starting Clipboard QR Application");

    // Create global clipboard state
    let clipboard_state = Arc::new(Mutex::new(GlobalClipboardState::new()));
    let clipboard_state_clone = clipboard_state.clone();

    // Start background clipboard monitoring thread
    let _background_thread = thread::spawn(move || {
        let mut clipboard_handler = ClipboardHandler::new();
        info!("Background clipboard monitoring thread started");

        loop {
            // Check for clipboard changes
            match clipboard_handler.get_data_if_changed() {
                Ok(Some(new_data)) => {
                    // Update global state
                    if let Ok(mut state) = clipboard_state_clone.lock() {
                        state.last_data = Some(new_data);
                        state.has_changed = true;
                    }
                    info!("Clipboard data updated in background thread");
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

    // Run CLI mode
    info!("Starting CLI mode...");
    run_cli_mode(clipboard_state)
}

fn run_cli_mode(clipboard_state: Arc<Mutex<GlobalClipboardState>>) -> Result<()> {
    info!("Starting CLI mode...");
    
    let mut cli = ClipboardQRCLI::new(clipboard_state);
    cli.run()?;
    
    Ok(())
} 