use qrcode::{QrCode, render::svg};
use image::{ImageBuffer, Luma};
use anyhow::Result;
use std::fs;
use std::path::Path;
use tracing::info;
use eframe::egui::ColorImage;

pub struct QRGenerator {
    size: u32,
}

impl QRGenerator {
    pub fn new() -> Self {
        Self {
            size: 300,
        }
    }

    pub fn generate_qr_image(&self, text: &str) -> Result<Option<ColorImage>> {
        if text.is_empty() {
            return Ok(None);
        }

        // Generate QR code
        let code = QrCode::new(text)?;
        
        // Convert to image buffer
        let image_buffer = self.qr_code_to_image(&code)?;
        
        // Convert to RGBA
        let rgba_image = self.convert_to_rgba(&image_buffer)?;
        
        // Convert to egui ColorImage
        let size = [rgba_image.width() as usize, rgba_image.height() as usize];
        let pixels: Vec<egui::Color32> = rgba_image
            .pixels()
            .map(|pixel| {
                egui::Color32::from_rgba_premultiplied(
                    pixel[0], pixel[1], pixel[2], pixel[3]
                )
            })
            .collect();
        
        Ok(Some(ColorImage { size, pixels }))
    }

    fn qr_code_to_image(&self, code: &QrCode) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>> {
        let image = code.render()
            .dark_color(Luma([0]))
            .light_color(Luma([255]))
            .build();
        
        Ok(image)
    }

    fn convert_to_rgba(&self, luma_image: &ImageBuffer<Luma<u8>, Vec<u8>>) -> Result<ImageBuffer<image::Rgba<u8>, Vec<u8>>> {
        let mut rgba_image = ImageBuffer::new(luma_image.width(), luma_image.height());
        
        for (x, y, pixel) in luma_image.enumerate_pixels() {
            let gray_value = pixel[0];
            let rgba_pixel = image::Rgba([gray_value, gray_value, gray_value, 255]);
            rgba_image.put_pixel(x, y, rgba_pixel);
        }
        
        Ok(rgba_image)
    }

    pub fn save_qr_image(&self, text: &str) -> Result<()> {
        if text.is_empty() {
            return Err(anyhow::anyhow!("No text to generate QR code"));
        }

        // Generate QR code
        let code = QrCode::new(text)?;
        
        // Create output directory if it doesn't exist
        let output_dir = Path::new("output");
        if !output_dir.exists() {
            fs::create_dir_all(output_dir)?;
        }
        
        // Generate filename based on content hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();
        let filename = format!("qr_code_{:x}.png", hash);
        let filepath = output_dir.join(filename);
        
        // Convert to image and save
        let image_buffer = self.qr_code_to_image(&code)?;
        image_buffer.save(&filepath)?;
        
        info!("QR code saved to: {:?}", filepath);
        Ok(())
    }

    pub fn generate_svg(&self, text: &str) -> Result<String> {
        if text.is_empty() {
            return Err(anyhow::anyhow!("No text to generate QR code"));
        }

        let code = QrCode::new(text)?;
        let svg_string = code.render()
            .min_dimensions(self.size, self.size)
            .dark_color(svg::Color("#000000"))
            .light_color(svg::Color("#ffffff"))
            .build();
        
        Ok(svg_string)
    }

    pub fn print_qr_terminal(&self, text: &str) -> Result<()> {
        if text.is_empty() {
            return Err(anyhow::anyhow!("No text to generate QR code"));
        }

        let code = QrCode::new(text)?;
        let string = code.render()
            .dark_color('█')
            .light_color(' ')
            .build();
        
        println!("{}", string);
        Ok(())
    }
} 