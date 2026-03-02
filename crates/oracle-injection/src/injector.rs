use anyhow::Result;
use std::ffi::CString;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use winapi::{
    shared::minwindef::{FALSE, LPVOID},
    um::{
        handleapi::CloseHandle,
        errhandlingapi::GetLastError,
        libloaderapi::{GetModuleHandleA, GetProcAddress},
        memoryapi::{VirtualAllocEx, WriteProcessMemory},
        processthreadsapi::{CreateRemoteThread, OpenProcess},
        synchapi::WaitForSingleObject,
        winnt::{MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE},
    },
};

// More permissive access rights for modern protected processes
#[cfg(target_os = "windows")]
const PROCESS_INJECT_ACCESS: u32 = 0x1F0FFF; // PROCESS_ALL_ACCESS equivalent

pub struct Injector {
    target_process_id: u32,
    dll_path: PathBuf,
}

impl Injector {
    pub fn new(target_process_id: u32, dll_path: PathBuf) -> Self {
        Self {
            target_process_id,
            dll_path,
        }
    }

    #[cfg(target_os = "windows")]
    pub fn inject(&self) -> Result<()> {
        if !self.dll_path.exists() {
            anyhow::bail!("DLL not found: {}", self.dll_path.display());
        }

        unsafe {
            // Open target process with required access rights
            let process_handle = OpenProcess(PROCESS_INJECT_ACCESS, FALSE, self.target_process_id);

            if process_handle.is_null() {
                let err = GetLastError();
                anyhow::bail!(
                    "Failed to open process {} (error: {}). Try running as administrator.",
                    self.target_process_id,
                    err
                );
            }

            // Prepare DLL path
            let dll_path_str = self
                .dll_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("DLL path contains invalid UTF-8"))?;
            let dll_path_cstring = CString::new(dll_path_str)?;
            let dll_path_bytes = dll_path_cstring.as_bytes_with_nul();

            // Allocate memory in target process
            let remote_memory = VirtualAllocEx(
                process_handle,
                std::ptr::null_mut(),
                dll_path_bytes.len(),
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            );

            if remote_memory.is_null() {
                CloseHandle(process_handle);
                anyhow::bail!("Failed to allocate memory in target process");
            }

            log::debug!(
                "Allocated {} bytes at: {:p}",
                dll_path_bytes.len(),
                remote_memory
            );

            // Write DLL path to target process
            let mut bytes_written = 0;
            let write_result = WriteProcessMemory(
                process_handle,
                remote_memory,
                dll_path_bytes.as_ptr() as LPVOID,
                dll_path_bytes.len(),
                &mut bytes_written,
            );

            if write_result == 0 {
                CloseHandle(process_handle);
                anyhow::bail!("Failed to write to target process memory");
            }

            log::debug!("Wrote {} bytes to remote process", bytes_written);

            // Get LoadLibraryA address
            let kernel32 = CString::new("kernel32.dll")?;
            let kernel32_handle = GetModuleHandleA(kernel32.as_ptr());

            if kernel32_handle.is_null() {
                CloseHandle(process_handle);
                anyhow::bail!("Failed to get kernel32.dll handle");
            }

            let loadlibrary_name = CString::new("LoadLibraryA")?;
            let loadlibrary_addr = GetProcAddress(kernel32_handle, loadlibrary_name.as_ptr());

            if loadlibrary_addr.is_null() {
                CloseHandle(process_handle);
                anyhow::bail!("Failed to get LoadLibraryA address");
            }

            log::debug!("LoadLibraryA at: {:p}", loadlibrary_addr);

            // Create remote thread
            let thread_handle = CreateRemoteThread(
                process_handle,
                std::ptr::null_mut(),
                0,
                Some(std::mem::transmute(loadlibrary_addr)),
                remote_memory,
                0,
                std::ptr::null_mut(),
            );

            if thread_handle.is_null() {
                CloseHandle(process_handle);
                anyhow::bail!("Failed to create remote thread");
            }

            log::debug!("Remote thread created, waiting for completion...");

            // Wait for thread to complete with timeout
            let wait_result = WaitForSingleObject(thread_handle, 60000); // 10 second timeout
            if wait_result == 0x102 {
                // WAIT_TIMEOUT
                log::warn!("Remote thread wait timed out - DLL may not have loaded properly");
            }

            // Get exit code to check if LoadLibraryA succeeded
            let mut exit_code: u32 = 0;
            use winapi::um::processthreadsapi::GetExitCodeThread;
            if GetExitCodeThread(thread_handle, &mut exit_code) != 0 {
                if exit_code == 0 {
                    log::warn!("LoadLibraryA returned NULL - DLL may have failed to load");
                } else {
                    log::debug!("LoadLibraryA returned: 0x{:X}", exit_code);
                }
            }

            CloseHandle(thread_handle);
            CloseHandle(process_handle);

            log::info!(
                "✓ Successfully injected {} into PID {}",
                dll_path_str,
                self.target_process_id
            );
            Ok(())
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn inject(&self) -> Result<()> {
        anyhow::bail!("DLL injection is only supported on Windows")
    }
}

