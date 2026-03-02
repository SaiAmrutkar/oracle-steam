// Early Injection - Start Steam suspended, inject hook, then resume
// This is how GreenLuma works - the hook is present from the very start

use anyhow::{Context, Result};
use std::ffi::CString;
use std::path::PathBuf;
use std::ptr::null_mut;

#[cfg(target_os = "windows")]
use winapi::{
    shared::minwindef::{FALSE, LPVOID},
    um::{
        handleapi::CloseHandle,
        libloaderapi::{GetModuleHandleA, GetProcAddress},
        memoryapi::{VirtualAllocEx, WriteProcessMemory},
        processthreadsapi::{CreateProcessA, CreateRemoteThread, ResumeThread},
        synchapi::WaitForSingleObject,
        winbase::CREATE_SUSPENDED,
        winnt::{MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE},
    },
};

pub struct EarlyInjector {
    steam_path: PathBuf,
    dll_path: PathBuf,
}

impl EarlyInjector {
    pub fn new(steam_path: PathBuf, dll_path: PathBuf) -> Self {
        Self {
            steam_path,
            dll_path,
        }
    }

    /// Start Steam as suspended process, inject hook, then resume
    #[cfg(target_os = "windows")]
    pub fn inject_early(&self) -> Result<u32> {
        unsafe {
            // Convert paths to C strings
            let steam_exe = CString::new(
                self.steam_path.to_str().context("Invalid steam path")?
            )?;
            
            let dll_path_str = self.dll_path.to_str().context("Invalid DLL path")?;
            let dll_path_cstring = CString::new(dll_path_str)?;

            log::info!("Starting Steam in suspended mode...");
            log::info!("Steam: {}", self.steam_path.display());
            log::info!("DLL: {}", self.dll_path.display());

            // Create Steam process in suspended state
            let mut startup_info: winapi::um::processthreadsapi::STARTUPINFOA = 
                std::mem::zeroed();
            startup_info.cb = std::mem::size_of::<winapi::um::processthreadsapi::STARTUPINFOA>() as u32;
            
            let mut process_info: winapi::um::processthreadsapi::PROCESS_INFORMATION = 
                std::mem::zeroed();

            let success = CreateProcessA(
                steam_exe.as_ptr(),
                null_mut(), // Command line
                null_mut(), // Process security attributes
                null_mut(), // Thread security attributes
                FALSE,      // Inherit handles
                CREATE_SUSPENDED, // Start suspended!
                null_mut(), // Environment
                null_mut(), // Current directory
                &mut startup_info,
                &mut process_info,
            );

            if success == 0 {
                anyhow::bail!("Failed to create Steam process (error: {})", 
                    winapi::um::errhandlingapi::GetLastError());
            }

            let process_handle = process_info.hProcess;
            let thread_handle = process_info.hThread;
            let process_id = process_info.dwProcessId;

            log::info!("Steam started suspended (PID: {})", process_id);

            // Now inject the DLL while Steam is suspended
            self.inject_dll(process_handle, &dll_path_cstring)?;

            log::info!("DLL injected, resuming Steam...");

            // Resume the main thread
            if ResumeThread(thread_handle) == 0xFFFFFFFF {
                log::warn!("Failed to resume main thread");
            }

            // Clean up handles
            CloseHandle(thread_handle);
            CloseHandle(process_handle);

            log::info!("Steam resumed with hook installed!");
            
            Ok(process_id)
        }
    }

    /// Inject DLL into suspended process
    #[cfg(target_os = "windows")]
    unsafe fn inject_dll(&self, process_handle: *mut winapi::ctypes::c_void, dll_path: &CString) -> Result<()> {
        let dll_path_bytes = dll_path.as_bytes_with_nul();

        // Allocate memory in target process
        let remote_memory = VirtualAllocEx(
            process_handle,
            null_mut(),
            dll_path_bytes.len(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );

        if remote_memory.is_null() {
            anyhow::bail!("Failed to allocate memory in Steam process");
        }

        // Write DLL path
        let mut bytes_written = 0;
        let write_result = WriteProcessMemory(
            process_handle,
            remote_memory,
            dll_path_bytes.as_ptr() as LPVOID,
            dll_path_bytes.len(),
            &mut bytes_written,
        );

        if write_result == 0 {
            anyhow::bail!("Failed to write to Steam process memory");
        }

        log::debug!("Wrote {} bytes to Steam process", bytes_written);

        // Get LoadLibraryA address
        let kernel32 = CString::new("kernel32.dll")?;
        let kernel32_handle = GetModuleHandleA(kernel32.as_ptr());
        
        if kernel32_handle.is_null() {
            anyhow::bail!("Failed to get kernel32.dll handle");
        }

        let loadlibrary_name = CString::new("LoadLibraryA")?;
        let loadlibrary_addr = GetProcAddress(kernel32_handle, loadlibrary_name.as_ptr());

        if loadlibrary_addr.is_null() {
            anyhow::bail!("Failed to get LoadLibraryA address");
        }

        // Create remote thread to load the DLL
        let thread_handle = CreateRemoteThread(
            process_handle,
            null_mut(),
            0,
            Some(std::mem::transmute(loadlibrary_addr)),
            remote_memory,
            0,
            null_mut(),
        );

        if thread_handle.is_null() {
            anyhow::bail!("Failed to create remote thread in Steam");
        }

        log::info!("Remote thread created");

        // DON'T wait for DLL to fully load - just give it a moment to start
        // The DLL will continue loading asynchronously
        // This is key for Rust DLLs which have slow initialization
        WaitForSingleObject(thread_handle, 100); // 100ms is enough to start loading
        
        // Close handle - the thread will continue running
        CloseHandle(thread_handle);
        
        log::info!("DLL load initiated (will complete asynchronously)");
        
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn inject_early(&self) -> Result<u32> {
        anyhow::bail!("Early injection only supported on Windows")
    }
}
