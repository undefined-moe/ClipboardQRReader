use eframe::egui;
use anyhow::Result;
use tracing::{info, error};

mod app;
mod qr_generator;
mod qr_scanner;
mod clipboard_handler;
mod cli;

use app::ClipboardQRApp;
use cli::ClipboardQRCLI;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("Starting Clipboard QR Application");

    // Check if we're in a headless environment (only on Unix-like systems)
    #[cfg(unix)]
    {
        if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
            error!("No display server detected. Starting CLI mode.");
            return run_cli_mode();
        }
    }

    // Try to run GUI mode
    match run_gui_mode() {
        Ok(()) => Ok(()),
        Err(e) => {
            error!("GUI mode failed: {}", e);
            error!("Falling back to CLI mode...");
            run_cli_mode()
        }
    }
}

fn run_gui_mode() -> Result<()> {
    info!("Starting GUI mode...");
    
    // Set up the native options
    let options = eframe::NativeOptions::default();

    // Run the application
    eframe::run_native(
        "Clipboard QR",
        options,
        Box::new(|_cc| {
            Box::new(ClipboardQRApp::new())
        }),
    ).map_err(|e| {
        error!("Failed to start GUI application: {}", e);
        anyhow::anyhow!("GUI initialization failed: {}. Please ensure you have a display server running.", e)
    })?;

    Ok(())
}

fn run_cli_mode() -> Result<()> {
    info!("Starting CLI mode...");
    
    let mut cli = ClipboardQRCLI::new();
    cli.run()?;
    
    Ok(())
} 