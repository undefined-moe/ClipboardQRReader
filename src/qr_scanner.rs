use bardecoder;
use image::{ImageBuffer, Rgba, Luma, DynamicImage};
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

    /// Scan QR code from a file path
    pub fn scan_qr_from_file(&self, file_path: &str) -> Result<Option<String>> {
        debug!("Scanning QR code from file: {}", file_path);
        
        // Load image from file
        let image = image::open(file_path)?;
        
        // Try to decode QR code
        let results = self.decoder.decode(&image);
        
        if !results.is_empty() {
            // Take the first successful result
            if let Some(result) = results.first() {
                match result {
                    Ok(content) => {
                        debug!("QR code detected in file: {}", content);
                        Ok(Some(content.clone()))
                    },
                    Err(e) => {
                        warn!("QR code detected in file but failed to decode: {}", e);
                        Ok(None)
                    },
                }
            } else {
                Ok(None)
            }
        } else {
            debug!("No QR code found in file: {}", file_path);
            Ok(None)
        }
    }

    /// Convert RGBA image to Luma (grayscale) for better QR detection
    fn convert_rgba_to_luma(&self, rgba_image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>> {
        let mut luma_image = ImageBuffer::new(rgba_image.width(), rgba_image.height());
        
        for (x, y, pixel) in rgba_image.enumerate_pixels() {
            // Convert RGBA to grayscale using standard luminance formula
            let gray_value = (pixel[0] as u32 * 299 + pixel[1] as u32 * 587 + pixel[2] as u32 * 114) / 1000;
            let luma_pixel = Luma([gray_value as u8]);
            luma_image.put_pixel(x, y, luma_pixel);
        }
        
        Ok(luma_image)
    }

    /// Scan multiple QR codes from an image
    pub fn scan_multiple_qr_codes(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<Vec<String>> {
        debug!("Scanning for multiple QR codes in image ({}x{})", image.width(), image.height());
        
        // Convert RGBA to DynamicImage for bardecoder
        let dynamic_image = DynamicImage::ImageRgba8(image.clone());
        let results = self.decoder.decode(&dynamic_image);
        
        let mut qr_contents = Vec::new();
        
        for result in results {
            match result {
                Ok(content) => {
                    debug!("Multiple QR code detected: {}", content);
                    qr_contents.push(content);
                },
                Err(e) => {
                    warn!("Failed to decode one of the QR codes: {}", e);
                },
            }
        }
        
        Ok(qr_contents)
    }

    /// Check if an image contains a QR code without decoding
    pub fn has_qr_code(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<bool> {
        // Convert RGBA to DynamicImage for bardecoder
        let dynamic_image = DynamicImage::ImageRgba8(image.clone());
        let results = self.decoder.decode(&dynamic_image);
        
        Ok(!results.is_empty())
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

    #[test]
    fn test_convert_rgba_to_luma() {
        let scanner = QRScanner::new();
        let mut rgba_image = ImageBuffer::new(10, 10);
        
        // Fill with white pixels
        for pixel in rgba_image.pixels_mut() {
            *pixel = Rgba([255, 255, 255, 255]);
        }
        
        let luma_image = scanner.convert_rgba_to_luma(&rgba_image).unwrap();
        assert_eq!(luma_image.width(), 10);
        assert_eq!(luma_image.height(), 10);
        
        // Check that white pixels converted to appropriate grayscale
        let center_pixel = luma_image.get_pixel(5, 5);
        assert_eq!(center_pixel[0], 255); // Should be white in grayscale
    }

    #[test]
    fn test_has_qr_code_with_empty_image() {
        let scanner = QRScanner::new();
        let rgba_image = ImageBuffer::new(100, 100);
        
        let has_qr = scanner.has_qr_code(&rgba_image).unwrap();
        assert!(!has_qr);
    }
} 