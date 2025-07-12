#[cfg(target_os = "windows")]
fn is_launched_from_terminal() -> bool {
    use std::ptr;
    use winapi::um::processthreadsapi::{GetCurrentProcessId, OpenProcess};
    use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS};
    use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
    use winapi::um::psapi::GetModuleFileNameExW;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    unsafe {
        let current_pid = GetCurrentProcessId();
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        
        if snapshot == winapi::um::handleapi::INVALID_HANDLE_VALUE {
            return false;
        }

        let mut pe32: PROCESSENTRY32W = std::mem::zeroed();
        pe32.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut pe32) == 0 {
            winapi::um::handleapi::CloseHandle(snapshot);
            return false;
        }

        loop {
            if pe32.th32ProcessID == current_pid {
                // Found current process, check its parent
                let parent_pid = pe32.th32ParentProcessID;
                winapi::um::handleapi::CloseHandle(snapshot);
                
                // Now find the parent process
                let parent_snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
                if parent_snapshot == winapi::um::handleapi::INVALID_HANDLE_VALUE {
                    return false;
                }

                let mut parent_pe32: PROCESSENTRY32W = std::mem::zeroed();
                parent_pe32.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

                if Process32FirstW(parent_snapshot, &mut parent_pe32) == 0 {
                    winapi::um::handleapi::CloseHandle(parent_snapshot);
                    return false;
                }

                loop {
                    if parent_pe32.th32ProcessID == parent_pid {
                        // Get parent process handle
                        let parent_handle = OpenProcess(
                            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                            0,
                            parent_pid,
                        );

                        if parent_handle != ptr::null_mut() {
                            let mut filename = [0u16; 260];
                            let len = GetModuleFileNameExW(parent_handle, ptr::null_mut(), filename.as_mut_ptr(), 260);
                            winapi::um::handleapi::CloseHandle(parent_handle);
                            winapi::um::handleapi::CloseHandle(parent_snapshot);

                            if len > 0 {
                                let filename_str = OsString::from_wide(&filename[..len as usize]);
                                let filename_lower = filename_str.to_string_lossy().to_lowercase();
                                
                                // Check if parent is a terminal emulator
                                return filename_lower.contains("cmd.exe") ||
                                       filename_lower.contains("powershell.exe") ||
                                       filename_lower.contains("conhost.exe") ||
                                       filename_lower.contains("terminal.exe") ||
                                       filename_lower.contains("wt.exe") ||
                                       filename_lower.contains("bash.exe") ||
                                       filename_lower.contains("zsh.exe") ||
                                       filename_lower.contains("sh.exe");
                            }
                        }
                        break;
                    }

                    if Process32NextW(parent_snapshot, &mut parent_pe32) == 0 {
                        break;
                    }
                }
                winapi::um::handleapi::CloseHandle(parent_snapshot);
                break;
            }

            if Process32NextW(snapshot, &mut pe32) == 0 {
                break;
            }
        }

        winapi::um::handleapi::CloseHandle(snapshot);
        false
    }
}

#[cfg(not(target_os = "windows"))]
fn is_launched_from_terminal() -> bool {
    // On non-Windows systems, we can check if stdin is a terminal
    use std::io::IsTerminal;
    std::io::stdin().is_terminal()
}

#[cfg(target_os = "windows")]
fn hide_console_window() {
    unsafe { winapi::um::wincon::FreeConsole() };
}

#[cfg(not(target_os = "windows"))]
fn hide_console_window() {
    // do nothing
}

pub fn hide_console_if_needed() {
    if is_launched_from_terminal() {
        return;
    }
    hide_console_window();
}
