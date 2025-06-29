use anyhow::Result;
use tracing::{info, error};
use std::io::{self, Write};

use crate::qr_generator::QRGenerator;
use crate::clipboard_handler::ClipboardHandler;

pub struct ClipboardQRCLI {
    qr_generator: QRGenerator,
    clipboard_handler: ClipboardHandler,
}

impl ClipboardQRCLI {
    pub fn new() -> Self {
        Self {
            qr_generator: QRGenerator::new(),
            clipboard_handler: ClipboardHandler::new(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        println!("🔗 Clipboard QR Code Generator (CLI Mode)");
        println!("==========================================");
        
        loop {
            println!("\nOptions:");
            println!("1. Read from clipboard and display QR code");
            println!("2. Enter text manually and display QR code");
            println!("3. Generate SVG QR code (save to file)");
            println!("4. Save QR code as PNG file");
            println!("5. Exit");
            print!("Choose an option (1-5): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let choice = input.trim();

            match choice {
                "1" => self.handle_clipboard_input()?,
                "2" => self.handle_manual_input()?,
                "3" => self.handle_svg_generation()?,
                "4" => self.handle_png_save()?,
                "5" => {
                    println!("Goodbye!");
                    break;
                },
                _ => println!("Invalid option. Please choose 1-5."),
            }
        }

        Ok(())
    }

    fn handle_clipboard_input(&mut self) -> Result<()> {
        println!("Reading from clipboard...");
        
        match self.clipboard_handler.get_text() {
            Ok(text) => {
                if text.is_empty() {
                    println!("Clipboard is empty.");
                    return Ok(());
                }
                
                println!("Clipboard content: {}", text);
                println!("\nQR Code:");
            },
            Err(e) => {
                error!("Failed to read clipboard: {}", e);
                println!("Failed to read clipboard: {}", e);
            },
        }
        
        Ok(())
    }

    fn handle_manual_input(&mut self) -> Result<()> {
        print!("Enter text to generate QR code: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let text = input.trim();

        if text.is_empty() {
            println!("No text entered.");
            return Ok(());
        }

        println!("\nQR Code:");
        Ok(())
    }

    fn handle_svg_generation(&mut self) -> Result<()> {
        print!("Enter text to generate SVG QR code: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let text = input.trim();

        if text.is_empty() {
            println!("No text entered.");
            return Ok(());
        }

        match self.qr_generator.generate_svg(text) {
            Ok(svg) => {
                // Create output directory if it doesn't exist
                let output_dir = std::path::Path::new("output");
                if !output_dir.exists() {
                    std::fs::create_dir_all(output_dir)?;
                }

                // Generate filename
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                text.hash(&mut hasher);
                let hash = hasher.finish();
                let filename = format!("qr_code_{:x}.svg", hash);
                let filepath = output_dir.join(filename);

                // Save SVG
                std::fs::write(&filepath, svg)?;
                println!("✅ SVG QR code saved to: {:?}", filepath);
            },
            Err(e) => {
                error!("Failed to generate SVG: {}", e);
                println!("❌ Failed to generate SVG: {}", e);
            },
        }

        Ok(())
    }

    fn handle_png_save(&mut self) -> Result<()> {
        print!("Enter text to save as PNG QR code: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let text = input.trim();

        if text.is_empty() {
            println!("No text entered.");
            return Ok(());
        }

        println!("Saving QR code as PNG...");
        match self.qr_generator.save_qr_image(text) {
            Ok(()) => {
                println!("✅ QR code saved successfully!");
            },
            Err(e) => {
                error!("Failed to save QR code: {}", e);
                println!("❌ Failed to save QR code: {}", e);
            },
        }
        
        Ok(())
    }
} 