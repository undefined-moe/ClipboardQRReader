use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::{error, info};
use winit::{
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
};

mod clipboard_handler;
mod global_state;
mod qr_generator;
mod qr_scanner;
mod tray;
mod hide_console;

use clipboard_handler::ClipboardHandler;
use global_state::GlobalClipboardState;
use qr_generator::QRGenerator;
use qr_scanner::QRScanner;
use tray::SystemTray;
use tray_icon::{menu::MenuEvent, TrayIconEvent};

use winit::application::ApplicationHandler;
use winit::window::{Window, WindowId};

use hide_console::hide_console_if_needed;

#[derive(Debug)]
enum UserEvent {
    TrayIconEvent(TrayIconEvent),
    MenuEvent(MenuEvent),
}

#[derive(Default)]
struct App {
    window: Option<Window>,
    system_tray: Option<SystemTray>,
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes().with_visible(false))
                .unwrap(),
        );
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::MenuEvent(menu_event) => {
                info!("Menu event: {:?}", menu_event);
                if menu_event.id == self.system_tray.as_ref().unwrap().quit_id {
                    info!("Quit menu item selected");
                    event_loop.exit();
                }
            }
            UserEvent::TrayIconEvent(tray_event) => {
                info!("Tray event: {:?}", tray_event);
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Wait);
        info!("About to wait");
        if let Some(ref mut tray) = self.system_tray {
            if let Err(e) = tray.update_icon() {
                error!("Failed to update tray icon: {}", e);
            }
        }
    }
}


fn main() -> Result<()> {
    hide_console_if_needed();

    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("Starting Clipboard QR Application");

    // Create event loop with user events
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    info!("Event loop created successfully");

    // Create global clipboard state
    let clipboard_state = Arc::new(Mutex::new(GlobalClipboardState::new()));
    let clipboard_state_clone = clipboard_state.clone();

    // Start background clipboard monitoring thread
    let _background_thread = thread::spawn(move || {
        let qr_generator = QRGenerator::new();
        let qr_scanner = QRScanner::new();
        let mut clipboard_handler = ClipboardHandler::new();
        info!("Background clipboard monitoring thread started");

        loop {
            // Check for clipboard changes
            match clipboard_handler.get_data_if_changed() {
                Ok(Some(new_data)) => {
                    // Update global state
                    if let Ok(mut state) = clipboard_state_clone.lock() {
                        state.last_data = Some(new_data.clone());
                        state.has_changed = true;
                    }
                    info!("Clipboard data updated in background thread");

                    match &new_data {
                        crate::clipboard_handler::ClipboardData::Text(text) => {
                            println!("\n🔄 Clipboard text updated: {}", text);
                            println!("QR Code:");
                            if let Err(e) = qr_generator.print_qr_terminal(&text) {
                                println!("❌ Failed to generate QR code: {}", e);
                            }
                        }
                        crate::clipboard_handler::ClipboardData::Image(image) => {
                            println!(
                                "\n🔄 Clipboard image updated ({}x{})",
                                image.width(),
                                image.height()
                            );
                            println!("Scanning for QR codes...");

                            match qr_scanner.scan_qr_from_rgba(&image) {
                                Ok(Some(content)) => {
                                    println!("✅ QR code detected in clipboard image!");
                                    println!("Content: {}", content);

                                    // Also display QR code for the detected content
                                    println!("QR Code for detected content:");
                                    if let Err(e) = qr_generator.print_qr_terminal(&content) {
                                        println!("❌ Failed to generate QR code: {}", e);
                                    }
                                }
                                Ok(None) => {
                                    println!("❌ No QR code found in clipboard image");
                                }
                                Err(e) => {
                                    println!("❌ Error scanning QR code: {}", e);
                                }
                            }
                        }
                        crate::clipboard_handler::ClipboardData::Empty => {
                            println!("\n🔄 Clipboard cleared");
                        }
                    }
                }
                Ok(None) => {
                    // No change, continue monitoring
                }
                Err(e) => {
                    error!("Error checking clipboard: {}", e);
                }
            }

            // Sleep to avoid excessive CPU usage
            thread::sleep(Duration::from_millis(100));
        }
    });

    let system_tray = Some(SystemTray::new(clipboard_state.clone()).unwrap());
    // Set up tray event handlers
    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::TrayIconEvent(event));
    }));
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    let mut app = App {
        window: None,
        system_tray,
    };

    event_loop.run_app(&mut app)?;
    info!("Hidden window created for event loop");

    Ok(())
}
