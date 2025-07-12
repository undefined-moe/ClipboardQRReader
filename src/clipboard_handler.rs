use arboard::Clipboard;
use anyhow::Result;
use tracing::{debug, warn, info};
use std::time::SystemTime;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use image::{ImageBuffer, Rgba};
use std::sync::mpsc;
use std::thread;

#[cfg(windows)]
use winapi::shared::windef::HWND;
#[cfg(windows)]
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};

#[derive(Debug, Clone)]
pub enum ClipboardData {
    Text(String),
    Image(ImageBuffer<Rgba<u8>, Vec<u8>>),
    Empty,
}

pub struct ClipboardHandler {
    clipboard: Option<Clipboard>,
    last_hash: u64,
    last_check_time: SystemTime,
    #[cfg(any(windows, unix))]
    clipboard_channel: Option<mpsc::Receiver<()>>,
    #[cfg(any(windows, unix))]
    clipboard_thread: Option<thread::JoinHandle<()>>,
}

impl ClipboardHandler {
    pub fn new() -> Self {
        let clipboard = match Clipboard::new() {
            Ok(clipboard) => {
                debug!("Clipboard initialized successfully");
                Some(clipboard)
            },
            Err(e) => {
                warn!("Failed to initialize clipboard: {}", e);
                None
            },
        };

        #[cfg(windows)]
        let (clipboard_channel, clipboard_thread) = Self::start_windows_clipboard_listener();

        #[cfg(unix)]
        let (clipboard_channel, clipboard_thread) = Self::start_linux_clipboard_listener();

        #[cfg(not(any(windows, unix)))]
        let (clipboard_channel, clipboard_thread): (Option<mpsc::Receiver<()>>, Option<thread::JoinHandle<()>>) = (None, None);

        Self { 
            clipboard,
            last_hash: 0,
            last_check_time: SystemTime::now(),
            #[cfg(any(windows, unix))]
            clipboard_channel,
            #[cfg(any(windows, unix))]
            clipboard_thread,
        }
    }

    #[cfg(windows)]
    fn start_windows_clipboard_listener() -> (Option<mpsc::Receiver<()>>, Option<thread::JoinHandle<()>>) {
        use winapi::um::winuser::{AddClipboardFormatListener, RemoveClipboardFormatListener, WM_CLIPBOARDUPDATE};
        use winapi::um::winuser::{GetMessageW, TranslateMessage, DispatchMessageW, MSG};
        use winapi::um::winuser::{CreateWindowExW, RegisterClassExW, WNDCLASSEXW};
        use winapi::um::winuser::{WS_OVERLAPPED, CW_USEDEFAULT};
        use winapi::um::libloaderapi::GetModuleHandleW;
        use winapi::um::errhandlingapi::GetLastError;
        use std::ptr::null_mut;

        let (tx, rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            unsafe {
                let h_instance = GetModuleHandleW(null_mut());
                if h_instance.is_null() {
                    warn!("Failed to get module handle");
                    return;
                }

                // Use a unique class name with timestamp to avoid conflicts
                let unique_id = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis();
                let window_class_name = format!("ClipboardQRListener_{}\0", unique_id).encode_utf16().collect::<Vec<u16>>();
                
                let wc = WNDCLASSEXW {
                    cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                    style: 0,
                    lpfnWndProc: Some(Self::window_proc),
                    cbClsExtra: 0,
                    cbWndExtra: 0,
                    hInstance: h_instance,
                    hIcon: null_mut(),
                    hCursor: null_mut(),
                    hbrBackground: null_mut(),
                    lpszMenuName: null_mut(),
                    lpszClassName: window_class_name.as_ptr(),
                    hIconSm: null_mut(),
                };

                let class_atom = RegisterClassExW(&wc);
                if class_atom == 0 {
                    let error = GetLastError();
                    warn!("Failed to register window class, error code: {}", error);
                    // Continue with polling approach instead
                    return;
                }

                let hwnd = CreateWindowExW(
                    0,
                    window_class_name.as_ptr(),
                    "ClipboardQR Listener\0".encode_utf16().collect::<Vec<u16>>().as_ptr(),
                    WS_OVERLAPPED,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    null_mut(),
                    null_mut(),
                    h_instance,
                    null_mut(),
                );

                if hwnd.is_null() {
                    let error = GetLastError();
                    warn!("Failed to create window, error code: {}", error);
                    return;
                }

                // Add clipboard format listener
                if AddClipboardFormatListener(hwnd) == 0 {
                    let error = GetLastError();
                    warn!("Failed to add clipboard format listener, error code: {}. Falling back to polling.", error);
                    // Fall back to polling approach
                    return;
                } else {
                    info!("Windows clipboard listener started successfully");
                }

                // Message loop
                let mut msg: MSG = std::mem::zeroed();
                loop {
                    let result = GetMessageW(&mut msg, null_mut(), 0, 0);
                    if result <= 0 {
                        break;
                    }

                    if msg.message == WM_CLIPBOARDUPDATE {
                        // Send notification to main thread
                        if let Err(e) = tx.send(()) {
                            warn!("Failed to send clipboard notification: {}", e);
                            break;
                        }
                    }

                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                // Cleanup
                RemoveClipboardFormatListener(hwnd);
            }
        });

        (Some(rx), Some(handle))
    }

    #[cfg(windows)]
    extern "system" fn window_proc(
        hwnd: HWND,
        msg: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        use winapi::um::winuser::DefWindowProcW;
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }

    #[cfg(unix)]
    fn start_linux_clipboard_listener() -> (Option<mpsc::Receiver<()>>, Option<thread::JoinHandle<()>>) {
        use std::time::Duration;
        use std::env;
        let (tx, rx) = mpsc::channel();

        // Wayland: 直接轮询
        if env::var_os("WAYLAND_DISPLAY").is_some() {
            let handle = thread::spawn(move || {
                loop {
                    thread::sleep(Duration::from_millis(100));
                    if let Err(e) = tx.send(()) {
                        warn!("Failed to send clipboard check signal: {}", e);
                        break;
                    }
                }
            });
            return (Some(rx), Some(handle));
        }

        use x11rb::connection::Connection;
        use x11rb::protocol::xproto::{ConnectionExt, WindowClass};
        use x11rb::protocol::Event;
        
        let (conn, screen_num) = match x11rb::connect(None) {
            Ok((conn, screen_num)) => (conn, screen_num),
            Err(e) => {
                warn!("Failed to connect to X11 server: {}. Falling back to polling.", e);
                // Fall back to polling approach
                loop {
                    thread::sleep(Duration::from_millis(100));
                    if let Err(e) = tx.send(()) {
                        warn!("Failed to send clipboard check signal: {}", e);
                        break;
                    }
                }
                return (Some(rx), None);
            },
        };

        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;
        
        // Get atoms for clipboard
        let clipboard_atom = match conn.intern_atom(false, b"CLIPBOARD") {
            Ok(reply) => reply.reply().unwrap().atom,
            Err(e) => {
                warn!("Failed to get CLIPBOARD atom: {}. Falling back to polling.", e);
                loop {
                    thread::sleep(Duration::from_millis(100));
                    if let Err(e) = tx.send(()) {
                        warn!("Failed to send clipboard check signal: {}", e);
                        break;
                    }
                }
                return (Some(rx), None);
            },
        };
        
        let targets_atom = match conn.intern_atom(false, b"TARGETS") {
            Ok(reply) => reply.reply().unwrap().atom,
            Err(e) => {
                warn!("Failed to get TARGETS atom: {}. Falling back to polling.", e);
                loop {
                    thread::sleep(Duration::from_millis(100));
                    if let Err(e) = tx.send(()) {
                        warn!("Failed to send clipboard check signal: {}", e);
                        break;
                    }
                }
                return (Some(rx), None);
            },
        };

        // Create a window to receive selection events
        let window = match conn.generate_id() {
            Ok(id) => id,
            Err(e) => {
                warn!("Failed to generate window ID: {}. Falling back to polling.", e);
                loop {
                    thread::sleep(Duration::from_millis(100));
                    if let Err(e) = tx.send(()) {
                        warn!("Failed to send clipboard check signal: {}", e);
                        break;
                    }
                }
                return (Some(rx), None);
            },
        };

        // Create the window
        if let Err(e) = conn.create_window(
            0, // depth
            window,
            root,
            0, 0, // x, y
            1, 1, // width, height
            0, // border_width
            WindowClass::COPY_FROM_PARENT, // class - use COPY_FROM_PARENT instead of InputOutput
            0, // visual
            &x11rb::protocol::xproto::CreateWindowAux::new(), // value_list
        ) {
            warn!("Failed to create window: {}. Falling back to polling.", e);
            loop {
                thread::sleep(Duration::from_millis(100));
                if let Err(e) = tx.send(()) {
                    warn!("Failed to send clipboard check signal: {}", e);
                    break;
                }
            }
            return (Some(rx), None);
        }

        // Select for selection change events
        if let Err(e) = conn.change_window_attributes(
            window,
            &x11rb::protocol::xproto::ChangeWindowAttributesAux::new()
                .event_mask(x11rb::protocol::xproto::EventMask::NO_EVENT),
        ) {
            warn!("Failed to set window attributes: {}. Falling back to polling.", e);
            loop {
                thread::sleep(Duration::from_millis(100));
                if let Err(e) = tx.send(()) {
                    warn!("Failed to send clipboard check signal: {}", e);
                    break;
                }
            }
            return (Some(rx), None);
        }

        info!("Linux X11 clipboard listener started successfully");
        
        // Event loop
        let handle = thread::spawn(move || {
            loop {
                match conn.wait_for_event() {
                    Ok(event) => {
                        match event {
                            Event::SelectionNotify(_) => {
                                // Selection changed, notify main thread
                                if let Err(e) = tx.send(()) {
                                    warn!("Failed to send clipboard notification: {}", e);
                                    break;
                                }
                            },
                            _ => {
                                // Ignore other events
                            },
                        }
                    },
                    Err(e) => {
                        warn!("Error waiting for X11 event: {}. Falling back to polling.", e);
                        // Fall back to polling
                        loop {
                            thread::sleep(Duration::from_millis(100));
                            if let Err(e) = tx.send(()) {
                                warn!("Failed to send clipboard check signal: {}", e);
                                break;
                            }
                        }
                        break;
                    },
                }
            }
        });

        (Some(rx), Some(handle))
    }

    pub fn get_data(&mut self) -> Result<ClipboardData> {
        match &mut self.clipboard {
            Some(clipboard) => {
                // Try to get image first
                if let Ok(image) = clipboard.get_image() {
                    debug!("Successfully read image from clipboard");
                    let img_buffer = ImageBuffer::from_raw(
                        image.width as u32,
                        image.height as u32,
                        image.bytes.into_owned(),
                    ).ok_or_else(|| anyhow::anyhow!("Failed to create image buffer"))?;
                    
                    return Ok(ClipboardData::Image(img_buffer));
                }
                
                // Try to get text
                match clipboard.get_text() {
                    Ok(text) => {
                        debug!("Successfully read text from clipboard");
                        if text.is_empty() {
                            Ok(ClipboardData::Empty)
                        } else {
                            Ok(ClipboardData::Text(text))
                        }
                    },
                    Err(e) => {
                        warn!("Failed to read text from clipboard: {}", e);
                        Ok(ClipboardData::Empty)
                    },
                }
            },
            None => {
                Err(anyhow::anyhow!("Clipboard not available"))
            },
        }
    }

    pub fn has_changed(&mut self) -> Result<bool> {
        let current_data = self.get_data()?;
        let mut hasher = DefaultHasher::new();
        
        match &current_data {
            ClipboardData::Text(text) => text.hash(&mut hasher),
            ClipboardData::Image(image) => {
                // Hash the image dimensions and first few pixels for change detection
                (image.width(), image.height()).hash(&mut hasher);
                if let Some(pixel) = image.get_pixel_checked(0, 0) {
                    pixel.hash(&mut hasher);
                }
            },
            ClipboardData::Empty => "empty".hash(&mut hasher),
        }
        
        let current_hash = hasher.finish();
        let changed = current_hash != self.last_hash;
        
        if changed {
            self.last_hash = current_hash;
            self.last_check_time = SystemTime::now();
        }
        
        Ok(changed)
    }

    pub fn get_data_if_changed(&mut self) -> Result<Option<ClipboardData>> {
        // Check for clipboard events first
        #[cfg(any(windows, unix))]
        {
            if let Some(ref rx) = self.clipboard_channel {
                if let Ok(_) = rx.try_recv() {
                    // Clipboard changed, get the new data
                    return Ok(Some(self.get_data()?));
                }
            }
        }

        // Fallback to polling
        if self.has_changed()? {
            Ok(Some(self.get_data()?))
        } else {
            Ok(None)
        }
    }

    pub fn set_text(&mut self, text: &str) -> Result<()> {
        match &mut self.clipboard {
            Some(clipboard) => {
                match clipboard.set_text(text) {
                    Ok(()) => {
                        debug!("Successfully set text to clipboard");
                        // Update hash to prevent immediate change detection
                        let mut hasher = DefaultHasher::new();
                        text.hash(&mut hasher);
                        self.last_hash = hasher.finish();
                        Ok(())
                    },
                    Err(e) => {
                        warn!("Failed to set text to clipboard: {}", e);
                        Err(anyhow::anyhow!("Failed to set clipboard text: {}", e))
                    },
                }
            },
            None => {
                Err(anyhow::anyhow!("Clipboard not available"))
            },
        }
    }

    pub fn is_available(&self) -> bool {
        self.clipboard.is_some()
    }

    pub fn get_last_check_time(&self) -> SystemTime {
        self.last_check_time
    }
}

impl Drop for ClipboardHandler {
    fn drop(&mut self) {
        #[cfg(any(windows, unix))]
        {
            // Cleanup clipboard listener thread
            if let Some(handle) = self.clipboard_thread.take() {
                if let Err(e) = handle.join() {
                    warn!("Failed to join clipboard listener thread: {:?}", e);
                }
            }
        }
    }
} 