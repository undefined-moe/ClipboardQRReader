use anyhow::Result;
use std::env;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};
use tray_icon::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

use crate::global_state::GlobalClipboardState;

pub struct SystemTray {
    tray_icon: TrayIcon,
    clipboard_state: Arc<Mutex<GlobalClipboardState>>,
    pub quit_id: String,
    pub status_id: String,
    pub about_id: String,
}

impl SystemTray {
    pub fn new(
        clipboard_state: Arc<Mutex<GlobalClipboardState>>
    ) -> Result<Self> {
        let tray = Self::create_tray(clipboard_state)?;
        Ok(tray)
    }

    fn detect_wayland_environment() -> bool {
        env::var_os("WAYLAND_DISPLAY").is_some()
            || env::var("XDG_SESSION_TYPE").as_deref() == Ok("wayland")
    }

    fn setup_wayland_environment() {
        // Set up environment variables for Wayland tray support
        if Self::detect_wayland_environment() {
            info!("Detected Wayland environment, setting up tray environment");

            // Ensure XDG_CURRENT_DESKTOP is set correctly
            if env::var("XDG_CURRENT_DESKTOP").is_err() {
                env::set_var("XDG_CURRENT_DESKTOP", "GNOME");
                info!("Set XDG_CURRENT_DESKTOP to GNOME for better tray support");
            }

            // Set StatusNotifierItem support
            env::set_var("QT_QPA_PLATFORM", "wayland");

            // Enable system tray support
            if env::var("GNOME_SHELL_SESSION_MODE").is_err() {
                env::set_var("GNOME_SHELL_SESSION_MODE", "user");
            }

            info!("Wayland environment variables configured for tray support");
        }
    }

    fn load_icon() -> Result<Icon> {
        let mut icon_data = Vec::new();

        // Blue background (74, 144, 226, 255)
        for _ in 0..256 {
            icon_data.extend_from_slice(&[74, 144, 226, 255]);
        }

        // Make center 8x8 area white
        for y in 4..12 {
            for x in 4..12 {
                let index = (y * 16 + x) * 4;
                if index + 3 < icon_data.len() {
                    icon_data[index] = 255; // R
                    icon_data[index + 1] = 255; // G
                    icon_data[index + 2] = 255; // B
                    icon_data[index + 3] = 255; // A
                }
            }
        }

        let icon = Icon::from_rgba(icon_data, 16, 16)?;
        Ok(icon)
    }

    fn create_tray(clipboard_state: Arc<Mutex<GlobalClipboardState>>) -> Result<Self> {
        #[cfg(unix)]
        if Self::detect_wayland_environment() {
            info!("Running in Wayland environment");
            Self::setup_wayland_environment();

            // Give system tray more time to initialize in Wayland
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        // Create a simple 16x16 icon with blue background and white center
        let icon = Self::load_icon()?;

        // Create menu with proper IDs
        let quit_item = MenuItem::new("Exit", true, None);
        let status_item = MenuItem::new("Show Status", true, None);
        let about_item = MenuItem::new("About ClipboardQR", true, None);

        let tray_menu = Menu::new();
        tray_menu.append(&about_item)?;
        tray_menu.append(&PredefinedMenuItem::separator())?;
        tray_menu.append(&status_item)?;
        tray_menu.append(&PredefinedMenuItem::separator())?;
        tray_menu.append(&quit_item)?;

        // Create tray icon
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("Clipboard QR")
            .with_icon(icon)
            .build()?;

        info!("Tray icon created successfully");
        Ok(Self {
            tray_icon,
            clipboard_state,
            quit_id: quit_item.id().0.clone(),
            status_id: status_item.id().0.clone(),
            about_id: about_item.id().0.clone(),
        })
    }

    pub fn update_icon(&mut self) -> Result<()> {
        // Update tray icon based on clipboard state
        if let Ok(mut state) = self.clipboard_state.lock() {
            if state.has_changed {
                // Update tooltip to show change
                let tooltip = if let Some(data) = &state.last_data {
                    match data {
                        crate::clipboard_handler::ClipboardData::Text(text) => {
                            format!(
                                "Clipboard QR - Text: {}",
                                if text.len() > 30 {
                                    format!("{}...", &text[..30])
                                } else {
                                    text.clone()
                                }
                            )
                        }
                        crate::clipboard_handler::ClipboardData::Image(image) => {
                            format!("Clipboard QR - Image: {}x{}", image.width(), image.height())
                        }
                        crate::clipboard_handler::ClipboardData::Empty => {
                            "Clipboard QR - Empty".to_string()
                        }
                    }
                } else {
                    "Clipboard QR - Monitoring...".to_string()
                };

                if let Err(e) = self.tray_icon.set_tooltip(Some(tooltip)) {
                    warn!("Failed to update tray tooltip: {}", e);
                }

                state.has_changed = false;

                info!("Clipboard state updated, tray icon tooltip updated");
            }
        }
        Ok(())
    }

    pub fn show_notification(&self, title: &str, message: &str) -> Result<()> {
        // Update tooltip to show notification
        if let Err(e) = self
            .tray_icon
            .set_tooltip(Some(format!("{}: {}", title, message)))
        {
            warn!("Failed to set notification tooltip: {}", e);
        }
        info!("Notification: {} - {}", title, message);
        println!("🔔 {}: {}", title, message);
        Ok(())
    }
}

impl Drop for SystemTray {
    fn drop(&mut self) {
        info!("System tray dropped");
    }
}
