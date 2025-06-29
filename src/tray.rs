use tray_icon::{
    TrayIcon, TrayIconBuilder, 
    menu::{Menu, MenuItem, PredefinedMenuItem, MenuEvent},
    Icon, TrayIconEvent
};
use std::sync::{Arc, Mutex};
use anyhow::Result;
use tracing::info;
use std::thread;

use crate::global_state::GlobalClipboardState;

pub struct SystemTray {
    tray_icon: TrayIcon,
    clipboard_state: Arc<Mutex<GlobalClipboardState>>,
}

impl SystemTray {
    pub fn new(clipboard_state: Arc<Mutex<GlobalClipboardState>>) -> Result<Self> {
        let tray = Self::create_tray(clipboard_state)?;
        Ok(tray)
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
                    icon_data[index] = 255;     // R
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
        // Create a simple 16x16 icon with blue background and white center
        // 16x16 = 256 pixels, each pixel is 4 bytes (RGBA)
        let icon = Self::load_icon()?;

        // Create menu with proper IDs
        let quit_item = MenuItem::new("Exit", true, None);
        let status_item = MenuItem::new("Show Status", true, None);
        
        let tray_menu = Menu::new();
        tray_menu.append(&status_item)?;
        tray_menu.append(&PredefinedMenuItem::separator())?;
        tray_menu.append(&quit_item)?;

        // Create tray icon first
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("Clipboard QR")
            .with_icon(icon)
            .build()?;

        info!("Tray icon created successfully");

        // Wait a moment to ensure tray icon is fully initialized
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Now start event handling threads after tray icon is created
        let menu_channel = MenuEvent::receiver();
        let tray_channel = TrayIconEvent::receiver();
        let clipboard_state_clone = clipboard_state.clone();
        let clipboard_state_clone2 = clipboard_state.clone();
        
        info!("Event channels created, starting event handlers...");
        // Start menu event handler
        thread::spawn(move || {
            info!("Tray menu event handler started");
            for event in menu_channel {
                info!("Menu event received: {:?}", event.id);
                match event.id.as_ref() {
                    "Exit" => {
                        info!("Exit menu item clicked");
                        std::process::exit(0);
                    },
                    "Show Status" => {
                        info!("Show Status menu item clicked");
                        if let Ok(state) = clipboard_state_clone.lock() {
                            if let Some(data) = &state.last_data {
                                match data {
                                    crate::clipboard_handler::ClipboardData::Text(text) => {
                                        info!("Current clipboard text: {}", text);
                                        println!("Current clipboard text: {}", text);
                                    },
                                    crate::clipboard_handler::ClipboardData::Image(image) => {
                                        info!("Current clipboard image: {}x{}", image.width(), image.height());
                                        println!("Current clipboard image: {}x{}", image.width(), image.height());
                                    },
                                    crate::clipboard_handler::ClipboardData::Empty => {
                                        info!("Clipboard is empty");
                                        println!("Clipboard is empty");
                                    },
                                }
                            } else {
                                info!("No clipboard data available");
                                println!("No clipboard data available");
                            }
                        }
                    },
                    _ => {
                        info!("Unknown menu item clicked: {:?}", event.id);
                    }
                }
            }
        });

        // Start tray icon event handler
        thread::spawn(move || {
            info!("Tray icon event handler started");
            println!("🔧 Tray event handler thread started");
            println!("📡 Waiting for tray events...");
            
            for event in tray_channel {
                info!("Tray event received: {:?}", event);
                
                match event {
                    TrayIconEvent::Click { .. } => {
                        info!("Tray icon left clicked");
                        println!("🎯 Tray icon LEFT clicked!");
                        if let Ok(state) = clipboard_state_clone2.lock() {
                            if let Some(data) = &state.last_data {
                                match data {
                                    crate::clipboard_handler::ClipboardData::Text(text) => {
                                        println!("📋 Current clipboard text: {}", text);
                                    },
                                    crate::clipboard_handler::ClipboardData::Image(image) => {
                                        println!("🖼️ Current clipboard image: {}x{}", image.width(), image.height());
                                    },
                                    crate::clipboard_handler::ClipboardData::Empty => {
                                        println!("📭 Clipboard is empty");
                                    },
                                }
                            } else {
                                println!("❌ No clipboard data available");
                            }
                        } else {
                            println!("❌ Failed to lock clipboard state");
                        }
                    },
                    _ => {
                        info!("Other tray event: {:?}", event);
                        println!("🔍 Other tray event: {:?}", event);
                    }
                }
            }
            info!("Tray icon event handler loop ended");
        });

        info!("System tray created successfully with event handlers");
        Ok(Self {
            tray_icon,
            clipboard_state,
        })
    }

    pub fn update_icon(&mut self) -> Result<()> {
        // Update tray icon based on clipboard state
        if let Ok(state) = self.clipboard_state.lock() {
            if state.has_changed {
                // Could change icon to indicate new clipboard data
                info!("Clipboard state updated, tray icon could be updated");
            }
        }
        Ok(())
    }

    pub fn show_notification(&self, title: &str, message: &str) -> Result<()> {
        // Update tooltip to show notification
        self.tray_icon.set_tooltip(Some(format!("{}: {}", title, message)))?;
        info!("Notification: {} - {}", title, message);
        Ok(())
    }
}

impl Drop for SystemTray {
    fn drop(&mut self) {
        info!("System tray dropped");
    }
} 