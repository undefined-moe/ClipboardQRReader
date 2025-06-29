use eframe::egui;
use anyhow::Result;
use tracing::{info, warn};

use crate::qr_generator::QRGenerator;
use crate::qr_scanner::QRScanner;
use crate::clipboard_handler::{ClipboardHandler, ClipboardData};

#[derive(PartialEq)]
enum Tab {
    Generator,
    Scanner,
}

pub struct ClipboardQRApp {
    qr_generator: QRGenerator,
    qr_scanner: QRScanner,
    clipboard_handler: ClipboardHandler,
    clipboard_text: String,
    last_clipboard_text: String,
    auto_update: bool,
    qr_texture: Option<egui::TextureId>,
    qr_image: Option<egui::ColorImage>,
    clipboard_data_type: String,
    qr_detection_status: String,
    file_path: String,
    scan_result: String,
    selected_tab: Tab,
}

impl ClipboardQRApp {
    pub fn new() -> Self {
        info!("Initializing Clipboard QR Application");
        
        Self {
            qr_generator: QRGenerator::new(),
            qr_scanner: QRScanner::new(),
            clipboard_handler: ClipboardHandler::new(),
            clipboard_text: String::new(),
            last_clipboard_text: String::new(),
            auto_update: true,
            qr_texture: None,
            qr_image: None,
            clipboard_data_type: String::new(),
            qr_detection_status: String::new(),
            file_path: String::new(),
            scan_result: String::new(),
            selected_tab: Tab::Generator,
        }
    }

    fn update_clipboard_content(&mut self) -> Result<()> {
        // Use event-based detection instead of polling
        match self.clipboard_handler.get_data_if_changed() {
            Ok(Some(data)) => {
                match data {
                    ClipboardData::Text(text) => {
                        self.clipboard_text = text.clone();
                        self.last_clipboard_text = text;
                        self.clipboard_data_type = "Text".to_string();
                        self.qr_detection_status = "".to_string();
                        
                        if !self.clipboard_text.is_empty() {
                            info!("Clipboard text changed, generating QR code");
                            self.generate_qr_code()?;
                        }
                    },
                    ClipboardData::Image(image) => {
                        self.clipboard_data_type = format!("Image ({}x{})", image.width(), image.height());
                        
                        // Try to detect QR code in the image
                        match self.clipboard_handler.detect_qr_in_image(&image) {
                            Ok(Some(qr_content)) => {
                                info!("QR code detected in clipboard image: {}", qr_content);
                                self.clipboard_text = qr_content.clone();
                                self.last_clipboard_text = qr_content;
                                self.qr_detection_status = "✅ QR code detected in image".to_string();
                                
                                self.generate_qr_code()?;
                            },
                            Ok(None) => {
                                self.clipboard_text = "".to_string();
                                self.qr_detection_status = "❌ No QR code found in image".to_string();
                                self.qr_image = None;
                                self.qr_texture = None;
                            },
                            Err(e) => {
                                self.clipboard_text = "".to_string();
                                self.qr_detection_status = format!("❌ Error detecting QR code: {}", e);
                                self.qr_image = None;
                                self.qr_texture = None;
                            },
                        }
                    },
                    ClipboardData::Empty => {
                        self.clipboard_text = "".to_string();
                        self.clipboard_data_type = "Empty".to_string();
                        self.qr_detection_status = "".to_string();
                        self.qr_image = None;
                        self.qr_texture = None;
                    },
                }
            },
            Ok(None) => {
                // No change detected, do nothing
            },
            Err(e) => {
                warn!("Failed to read clipboard: {}", e);
                self.clipboard_data_type = "Error".to_string();
                self.qr_detection_status = format!("❌ Error: {}", e);
            },
        }
        Ok(())
    }

    fn generate_qr_code(&mut self) -> Result<()> {
        if let Some(color_image) = self.qr_generator.generate_qr_image(&self.clipboard_text)? {
            self.qr_image = Some(color_image);
            info!("QR code generated successfully");
        }
        Ok(())
    }

    fn update_texture(&mut self, ctx: &egui::Context) {
        if let Some(qr_image) = &self.qr_image {
            // Always create a new texture for simplicity
            let texture_id = ctx.tex_manager().write().alloc(
                "qr_code".to_string(),
                qr_image.clone().into(),
                Default::default(),
            );
            self.qr_texture = Some(texture_id);
        }
    }

    fn scan_file(&mut self) -> Result<()> {
        if self.file_path.is_empty() {
            self.scan_result = "Please enter a file path".to_string();
            return Ok(());
        }

        match self.qr_scanner.scan_qr_from_file(&self.file_path)? {
            Some(content) => {
                self.scan_result = format!("✅ QR code detected!\nContent: {}", content);
                self.clipboard_text = content.clone();
                self.last_clipboard_text = content;
                self.generate_qr_code()?;
            },
            None => {
                self.scan_result = format!("❌ No QR code found in file: {}", self.file_path);
            },
        }
        Ok(())
    }
}

impl eframe::App for ClipboardQRApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check clipboard changes less frequently since we use event detection
        // Only check every 100ms to reduce CPU usage
        let current_time = std::time::SystemTime::now();
        let time_since_last_check = current_time.duration_since(self.clipboard_handler.get_last_check_time())
            .unwrap_or_default();
        
        if self.auto_update && time_since_last_check.as_millis() >= 100 {
            if let Err(e) = self.update_clipboard_content() {
                warn!("Failed to update clipboard content: {}", e);
            }
        }

        // Update texture if needed
        self.update_texture(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Clipboard QR Code Generator & Scanner");
            
            ui.add_space(10.0);
            
            // Create tabs for different functions
            egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.selected_tab, Tab::Generator, "QR Generator");
                    ui.selectable_value(&mut self.selected_tab, Tab::Scanner, "QR Scanner");
                });
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                match self.selected_tab {
                    Tab::Generator => self.show_generator_tab(ui),
                    Tab::Scanner => self.show_scanner_tab(ui),
                }
            });
        });
    }
}

impl ClipboardQRApp {
    fn show_generator_tab(&mut self, ui: &mut egui::Ui) {
        // Controls and clipboard content in the same row
        ui.horizontal(|ui| {
            // Controls section (left side)
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
                    self.clipboard_data_type.clear();
                    self.qr_detection_status.clear();
                    self.qr_image = None;
                    self.qr_texture = None;
                }
            });
            
            ui.add_space(10.0);
            
            // Clipboard content display (right side)
            ui.group(|ui| {
                ui.label("Clipboard Content:");
                
                // Limit the height of the text area
                let available_height = ui.available_height() - 30.0; // Reserve space for label and button
                ui.add_sized(
                    [ui.available_width(), available_height],
                    egui::TextEdit::multiline(&mut self.clipboard_text)
                        .desired_rows(5)
                );
                
                // Generate QR code button for manual input
                if ui.button("Generate QR Code").clicked() {
                    if let Err(e) = self.generate_qr_code() {
                        warn!("Failed to generate QR code: {}", e);
                    }
                }
            });
        });
        
        ui.add_space(10.0);
        
        // QR code display
        if let Some(texture_id) = self.qr_texture {
            ui.group(|ui| {
                ui.label("QR Code:");
                
                // Display the QR code image
                if let Some(qr_image) = &self.qr_image {
                    let size = egui::vec2(qr_image.size[0] as f32, qr_image.size[1] as f32);
                    ui.image((texture_id, size));
                }
                
                ui.add_space(5.0);
                
                // Save button
                if ui.button("Save QR Code").clicked() {
                    if let Err(e) = self.qr_generator.save_qr_image(&self.clipboard_text) {
                        warn!("Failed to save QR code: {}", e);
                    } else {
                        info!("QR code saved successfully");
                    }
                }
            });
        } else if !self.clipboard_text.is_empty() {
            ui.group(|ui| {
                ui.label("QR Code Status:");
                ui.label(format!("Content: {}", self.clipboard_text));
                ui.label(format!("Type: {}", self.clipboard_data_type));
                ui.label(format!("Status: {}", self.qr_detection_status));
            });
        } else {
            ui.label("No QR code generated yet. Copy some text to clipboard or enter text above.");
        }
    }

    fn show_scanner_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("QR Code Scanner");
        ui.add_space(10.0);

        // File scanning section
        ui.group(|ui| {
            ui.label("Scan QR Code from File:");
            
            ui.horizontal(|ui| {
                ui.label("File path:");
                ui.text_edit_singleline(&mut self.file_path);
            });
            
            ui.horizontal(|ui| {
                if ui.button("Scan File").clicked() {
                    if let Err(e) = self.scan_file() {
                        warn!("Failed to scan file: {}", e);
                        self.scan_result = format!("❌ Error: {}", e);
                    }
                }
                
                if ui.button("Clear").clicked() {
                    self.file_path.clear();
                    self.scan_result.clear();
                }
            });
        });

        ui.add_space(10.0);

        // Scan results
        if !self.scan_result.is_empty() {
            ui.group(|ui| {
                ui.label("Scan Results:");
                ui.label(&self.scan_result);
                
                // If QR code was detected, show option to copy to clipboard
                if self.scan_result.contains("✅ QR code detected!") {
                    ui.add_space(5.0);
                    if ui.button("Copy Content to Clipboard").clicked() {
                        if let Err(e) = self.clipboard_handler.set_text(&self.clipboard_text) {
                            warn!("Failed to copy to clipboard: {}", e);
                        } else {
                            info!("Content copied to clipboard");
                        }
                    }
                }
            });
        }

        ui.add_space(10.0);

        // Clipboard image scanning info
        ui.group(|ui| {
            ui.label("Clipboard Image Scanning:");
            ui.label("Copy an image containing a QR code to your clipboard, then switch to the Generator tab to automatically detect and decode it.");
            ui.label(format!("Current clipboard type: {}", self.clipboard_data_type));
            if !self.qr_detection_status.is_empty() {
                ui.label(format!("Detection status: {}", self.qr_detection_status));
            }
        });
    }
} 