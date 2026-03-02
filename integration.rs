// This should be integrated into your Oracle Steam desktop application

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct OracleAppProtection {
    running: Arc<AtomicBool>,
    heartbeat_thread: Option<thread::JoinHandle<()>>,
}

impl OracleAppProtection {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            heartbeat_thread: None,
        }
    }

    pub fn start(&mut self) {
        self.running.store(true, Ordering::SeqCst);

        // Create IPC endpoint for games to verify app is running
        #[cfg(windows)]
        self.create_named_pipe();

        // Start heartbeat thread
        let running = self.running.clone();
        self.heartbeat_thread = Some(thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                // Call the DLL's heartbeat function
                unsafe {
                    oracle_core::protection::OracleSteam_Heartbeat();
                }

                thread::sleep(Duration::from_millis(1000));
            }
        }));

        println!("[Oracle App] Protection system started");
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);

        if let Some(handle) = self.heartbeat_thread.take() {
            let _ = handle.join();
        }

        println!("[Oracle App] Protection system stopped");
    }

    #[cfg(windows)]
    fn create_named_pipe(&self) {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use winapi::um::fileapi::*;
        use winapi::um::winbase::*;

        thread::spawn(|| {
            let pipe_name = r"\\.\pipe\oracle_steam_heartbeat";
            let wide_name: Vec<u16> = OsStr::new(pipe_name).encode_wide().chain(Some(0)).collect();

            unsafe {
                let pipe = CreateNamedPipeW(
                    wide_name.as_ptr(),
                    PIPE_ACCESS_DUPLEX,
                    PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                    1,
                    512,
                    512,
                    0,
                    std::ptr::null_mut(),
                );

                if pipe != INVALID_HANDLE_VALUE {
                    // Keep pipe open as long as app runs
                    loop {
                        ConnectNamedPipe(pipe, std::ptr::null_mut());
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        });
    }
}

impl Drop for OracleAppProtection {
    fn drop(&mut self) {
        self.stop();
    }
}

// Usage in your Oracle Steam app:
/*
fn main() {
    let mut protection = OracleAppProtection::new();
    protection.start();

    // Your app logic here
    run_oracle_steam_app();

    protection.stop();
}
*/
