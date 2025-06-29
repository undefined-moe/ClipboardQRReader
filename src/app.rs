use eframe::egui;
use anyhow::Result;
use tracing::{info, warn};

use crate::qr_generator::QRGenerator;
use crate::clipboard_handler::ClipboardHandler;

pub struct ClipboardQRApp {
    qr_generator: QRGenerator,
    clipboard_handler: ClipboardHandler,
    clipboard_text: String,
    last_clipboard_text: String,
    auto_update: bool,
}

impl ClipboardQRApp {
    pub fn new() -> Self {
        info!("Initializing Clipboard QR Application");
        
        Self {
            qr_generator: QRGenerator::new(),
            clipboard_handler: ClipboardHandler::new(),
            clipboard_text: String::new(),
            last_clipboard_text: String::new(),
            auto_update: true,
        }
    }

    fn update_clipboard_content(&mut self) -> Result<()> {
        match self.clipboard_handler.get_text() {
            Ok(text) => {
                if text != self.last_clipboard_text {
                    self.clipboard_text = text.clone();
                    self.last_clipboard_text = text;
                    
                    if !self.clipboard_text.is_empty() {
                        info!("Generating QR code for clipboard content");
                        // TODO: Implement QR image generation
                    }
                }
            },
            Err(e) => {
                warn!("Failed to read clipboard: {}", e);
            },
        }
        Ok(())
    }
}

impl eframe::App for ClipboardQRApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Auto-update clipboard content
        if self.auto_update {
            if let Err(e) = self.update_clipboard_content() {
                warn!("Failed to update clipboard content: {}", e);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Clipboard QR Code Generator");
            
            // Controls section
            ui.group(|ui| {
                ui.label("Controls:");
                
                // Auto-update toggle
                ui.checkbox(&mut self.auto_update, "Auto-update from clipboard");
                
                // Manual update button
                if ui.button("Update from Clipboard").clicked() {
                    if let Err(e) = self.update_clipboard_content() {
                        warn!("Failed to update clipboard content: {}", e);
                    }
                }
                
                // Clear button
                if ui.button("Clear").clicked() {
                    self.clipboard_text.clear();
                    self.last_clipboard_text.clear();
                }
            });
            
            ui.add_space(10.0);
            
            // Clipboard content display
            ui.group(|ui| {
                ui.label("Clipboard Content:");
                ui.text_edit_multiline(&mut self.clipboard_text);
            });
            
            ui.add_space(10.0);
            
            // QR code display
            if !self.clipboard_text.is_empty() {
                ui.group(|ui| {
                    ui.label("QR Code Status:");
                    ui.label(format!("Content: {}", self.clipboard_text));
                    ui.label("QR code generation is working (image display coming soon)");
                    
                    // Save button
                    if ui.button("Save QR Code").clicked() {
                        if let Err(e) = self.qr_generator.save_qr_image(&self.clipboard_text) {
                            warn!("Failed to save QR code: {}", e);
                        } else {
                            info!("QR code saved successfully");
                        }
                    }
                });
            } else {
                ui.label("No QR code generated yet. Copy some text to clipboard or enter text above.");
            }
        });
    }
} 