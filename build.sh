#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check if we're in a Nix environment
check_nix_environment() {
    if [[ -n "$IN_NIX_SHELL" ]]; then
        print_success "Running in Nix development environment"
        return 0
    else
        print_warning "Not in Nix environment. Consider running 'nix develop' first"
        return 1
    fi
}

# Function to build for current platform
build_current() {
    local target=${1:-"debug"}
    print_status "Building for current platform ($target)..."
    
    if [[ "$target" == "release" ]]; then
        cargo build --release
        print_success "Release build completed: target/release/clipboard-qr"
    else
        cargo build
        print_success "Debug build completed: target/debug/clipboard-qr"
    fi
}

# Function to build for Windows
build_windows() {
    print_status "Building for Windows (cross-compilation)..."
    
    # Check if Windows target is installed
    if ! rustup target list | grep -q "x86_64-pc-windows-gnu (installed)"; then
        print_status "Adding Windows target..."
        rustup target add x86_64-pc-windows-gnu
    fi
    
    cargo build --target x86_64-pc-windows-gnu --release
    print_success "Windows build completed: target/x86_64-pc-windows-gnu/release/clipboard-qr.exe"
}

# Function to run tests
run_tests() {
    print_status "Running tests..."
    cargo test
    print_success "Tests completed"
}

# Function to run linter
run_linter() {
    print_status "Running clippy..."
    cargo clippy
    print_success "Clippy completed"
}

# Function to format code
format_code() {
    print_status "Formatting code..."
    cargo fmt
    print_success "Code formatting completed"
}

# Function to run the application
run_app() {
    local target=${1:-"debug"}
    print_status "Running application ($target)..."
    
    if [[ "$target" == "release" ]]; then
        cargo run --release
    else
        cargo run
    fi
}

# Function to clean build artifacts
clean() {
    print_status "Cleaning build artifacts..."
    cargo clean
    print_success "Clean completed"
}

# Function to show help
show_help() {
    echo "Clipboard QR Build Script"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  build [debug|release]  Build for current platform (default: debug)"
    echo "  windows               Build for Windows (cross-compilation)"
    echo "  test                  Run tests"
    echo "  lint                  Run clippy linter"
    echo "  fmt                   Format code"
    echo "  run [debug|release]   Run application (default: debug)"
    echo "  clean                 Clean build artifacts"
    echo "  all                   Build for all platforms and run tests"
    echo "  help                  Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 build release      # Build release version for current platform"
    echo "  $0 windows            # Cross-compile for Windows"
    echo "  $0 all                # Build all platforms and run tests"
}

# Main script logic
main() {
    local command=${1:-"help"}
    
    case "$command" in
        "build")
            local target=${2:-"debug"}
            check_nix_environment
            build_current "$target"
            ;;
        "windows")
            check_nix_environment
            build_windows
            ;;
        "test")
            check_nix_environment
            run_tests
            ;;
        "lint")
            check_nix_environment
            run_linter
            ;;
        "fmt")
            check_nix_environment
            format_code
            ;;
        "run")
            local target=${2:-"debug"}
            check_nix_environment
            run_app "$target"
            ;;
        "clean")
            clean
            ;;
        "all")
            check_nix_environment
            print_status "Building and testing all targets..."
            build_current "release"
            build_windows
            run_tests
            run_linter
            format_code
            print_success "All tasks completed successfully!"
            ;;
        "help"|*)
            show_help
            ;;
    esac
}

# Run main function with all arguments
main "$@" 