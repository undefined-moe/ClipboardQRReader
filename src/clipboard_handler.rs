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
        // For Linux, we'll use a simple polling approach in a background thread
        // since X11 clipboard events are more complex to implement
        let (tx, rx) = mpsc::channel();
        
        let handle = thread::spawn(move || {
            use std::time::Duration;
            
            info!("Linux clipboard polling thread started");
            
            loop {
                // Sleep for a short time to avoid excessive CPU usage
                thread::sleep(Duration::from_millis(100));
                
                // Send a signal to check clipboard (this will be handled by the main thread)
                if let Err(e) = tx.send(()) {
                    warn!("Failed to send clipboard check signal: {}", e);
                    break;
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