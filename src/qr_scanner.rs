use bardecoder;
use image::{ImageBuffer, Rgba, DynamicImage};
use anyhow::Result;
use tracing::{warn, debug};

pub struct QRScanner {
    decoder: bardecoder::Decoder<DynamicImage, image::GrayImage, String>,
}

impl QRScanner {
    pub fn new() -> Self {
        Self {
            decoder: bardecoder::default_decoder(),
        }
    }

    /// Scan QR code from an RGBA image
    pub fn scan_qr_from_rgba(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<Option<String>> {
        debug!("Scanning QR code from RGBA image ({}x{})", image.width(), image.height());
        
        // Convert RGBA to DynamicImage for bardecoder
        let dynamic_image = DynamicImage::ImageRgba8(image.clone());
        
        // Try to decode QR code
        let results = self.decoder.decode(&dynamic_image);
        
        if !results.is_empty() {
            // Take the first successful result
            if let Some(result) = results.first() {
                match result {
                    Ok(content) => {
                        debug!("QR code detected: {}", content);
                        Ok(Some(content.clone()))
                    },
                    Err(e) => {
                        warn!("QR code detected but failed to decode: {}", e);
                        Ok(None)
                    },
                }
            } else {
                Ok(None)
            }
        } else {
            debug!("No QR code found in image");
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    #[test]
    fn test_scanner_creation() {
        let scanner = QRScanner::new();
        // Test that scanner can be created
    }
} 