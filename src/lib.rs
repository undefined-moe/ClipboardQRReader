pub mod qr_generator;
pub mod qr_scanner;
pub mod clipboard_handler;
pub mod global_state;

pub use qr_generator::QRGenerator;
pub use qr_scanner::QRScanner;
pub use clipboard_handler::ClipboardHandler;
pub use global_state::GlobalClipboardState;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_handler_creation() {
        let handler = ClipboardHandler::new();
        // Note: clipboard availability depends on the system
        // We just test that it can be created
    }

    #[test]
    fn test_qr_generator_with_empty_text() {
        let generator = QRGenerator::new();
        let result = generator.generate_qr_image("").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_qr_generator_with_valid_text() {
        let generator = QRGenerator::new();
        let result = generator.generate_qr_image("Hello, World!").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_svg_generation() {
        let generator = QRGenerator::new();
        let result = generator.generate_svg("Test QR Code");
        assert!(result.is_ok());
        let svg = result.unwrap();
        assert!(svg.contains("svg"));
    }

    #[test]
    fn test_qr_scanner_creation() {
        let scanner = QRScanner::new();
        // Test that scanner can be created
    }

    #[test]
    fn test_terminal_qr_generation() {
        let generator = QRGenerator::new();
        let result = generator.print_qr_terminal("Test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_terminal_qr_empty_text() {
        let generator = QRGenerator::new();
        let result = generator.print_qr_terminal("");
        assert!(result.is_err());
    }
} 