#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Function to detect environment
detect_environment() {
    echo "🔍 Detecting environment..."
    
    local wayland_display=""
    local x11_display=""
    local desktop_env=""
    
    if [ -n "$WAYLAND_DISPLAY" ]; then
        wayland_display="✓ ($WAYLAND_DISPLAY)"
    else
        wayland_display="✗"
    fi
    
    if [ -n "$DISPLAY" ]; then
        x11_display="✓ ($DISPLAY)"
    else
        x11_display="✗"
    fi
    
    if [ -n "$XDG_CURRENT_DESKTOP" ]; then
        desktop_env="$XDG_CURRENT_DESKTOP"
    else
        desktop_env="Unknown"
    fi
    
    echo "🖥️ Desktop Environment: $desktop_env"
    echo "🌊 Wayland Display: $wayland_display"
    echo "🔲 X11 Display: $x11_display"
    echo
}

# Function to run with native Wayland
run_native_wayland() {
    print_status "Running with native Wayland configuration..."
    
    # Set Wayland-specific environment
    export GDK_BACKEND=wayland
    export QT_QPA_PLATFORM=wayland
    export CLUTTER_BACKEND=wayland
    export SDL_VIDEODRIVER=wayland
    
    # Ensure XDG_CURRENT_DESKTOP is set
    if [ -z "$XDG_CURRENT_DESKTOP" ]; then
        export XDG_CURRENT_DESKTOP="wayland"
        print_warning "Set XDG_CURRENT_DESKTOP=wayland"
    fi
    
    echo "Environment variables set for Wayland:"
    echo "  GDK_BACKEND=$GDK_BACKEND"
    echo "  QT_QPA_PLATFORM=$QT_QPA_PLATFORM"
    echo "  XDG_CURRENT_DESKTOP=$XDG_CURRENT_DESKTOP"
    echo
    
    cargo run --release
}

# Function to run with X11 compatibility
run_x11_compatibility() {
    print_status "Running with X11 compatibility (XWayland)..."
    
    # Set X11-specific environment
    export GDK_BACKEND=x11
    export QT_QPA_PLATFORM=xcb
    export CLUTTER_BACKEND=x11
    export SDL_VIDEODRIVER=x11
    
    # Ensure XDG_CURRENT_DESKTOP is set
    if [ -z "$XDG_CURRENT_DESKTOP" ]; then
        export XDG_CURRENT_DESKTOP="X11"
        print_warning "Set XDG_CURRENT_DESKTOP=X11"
    fi
    
    echo "Environment variables set for X11 compatibility:"
    echo "  GDK_BACKEND=$GDK_BACKEND"
    echo "  QT_QPA_PLATFORM=$QT_QPA_PLATFORM"
    echo "  XDG_CURRENT_DESKTOP=$XDG_CURRENT_DESKTOP"
    echo
    
    cargo run --release
}

# Function to run with default settings
run_default() {
    print_status "Running with default environment settings..."
    cargo run --release
}

# Function to check tray support
check_tray_support() {
    print_status "Checking system tray support..."
    
    local desktop_env="${XDG_CURRENT_DESKTOP:-Unknown}"
    
    case "$desktop_env" in
        *GNOME*)
            echo "🔧 GNOME Desktop detected"
            echo "   System tray support requires extensions:"
            echo "   - Install: gnome-shell-extension-appindicator"
            echo "   - Or: sudo apt install gnome-shell-extension-appindicator"
            echo "   - Enable via GNOME Extensions or gnome-extensions-app"
            ;;
        *KDE*)
            echo "🔧 KDE Plasma detected"
            echo "   System tray should work out of the box"
            echo "   Check: System Settings > Startup and Shutdown > Background Services"
            ;;
        *Sway*)
            echo "🔧 Sway detected"
            echo "   System tray requires waybar or another compatible bar"
            echo "   Configure waybar.json with: \"tray\": {}"
            ;;
        *)
            echo "🔧 Desktop environment: $desktop_env"
            echo "   System tray support depends on your compositor and bar"
            echo "   Consider installing waybar or another tray-compatible bar"
            ;;
    esac
    echo
}

# Function to show help
show_help() {
    echo "🚀 Clipboard QR - Wayland Tray Helper"
    echo
    echo "Usage: $0 [OPTION]"
    echo
    echo "Options:"
    echo "  default       Run with default environment settings"
    echo "  wayland       Run with native Wayland configuration"
    echo "  x11           Run with X11 compatibility (XWayland)"
    echo "  check         Check system tray support for your environment"
    echo "  help          Show this help message"
    echo
    echo "Examples:"
    echo "  $0 wayland    # Try native Wayland first"
    echo "  $0 x11        # Use X11 compatibility if Wayland doesn't work"
    echo "  $0 check      # Check what tray support is available"
    echo
}

# Main function
main() {
    echo "🚀 Clipboard QR - Wayland Tray Helper"
    echo "=================================="
    echo
    
    detect_environment
    
    case "${1:-default}" in
        "wayland")
            run_native_wayland
            ;;
        "x11")
            run_x11_compatibility
            ;;
        "default")
            run_default
            ;;
        "check")
            check_tray_support
            ;;
        "help"|"-h"|"--help")
            show_help
            ;;
        *)
            print_error "Unknown option: $1"
            echo
            show_help
            exit 1
            ;;
    esac
}

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    print_error "Cargo is not installed or not in PATH"
    print_status "Please install Rust and Cargo, or run 'nix develop' in the project directory"
    exit 1
fi

# Check if we're in the project directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found. Please run this script from the project root directory."
    exit 1
fi

# Run main function
main "$@" 