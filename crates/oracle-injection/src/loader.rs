use anyhow::{Context, Result};
use std::path::Path;

#[cfg(target_os = "windows")]
use winapi::um::libloaderapi::{FreeLibrary, LoadLibraryA};

pub struct LoadedDll {
    #[cfg(target_os = "windows")]
    handle: *mut std::ffi::c_void,
}

impl LoadedDll {
    #[cfg(target_os = "windows")]
    pub fn handle(&self) -> *mut std::ffi::c_void {
        self.handle
    }
}

impl Drop for LoadedDll {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        unsafe {
            if !self.handle.is_null() {
                FreeLibrary(self.handle as *mut _);
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub fn load_dll(path: &Path) -> Result<LoadedDll> {
    use std::ffi::CString;

    let path_str = path.to_str().context("Invalid DLL path")?;
    let path_cstring = CString::new(path_str)?;

    unsafe {
        let handle = LoadLibraryA(path_cstring.as_ptr());
        if handle.is_null() {
            anyhow::bail!("LoadLibraryA failed for: {}", path_str);
        }

        log::info!("✓ Loaded DLL: {}", path_str);
        Ok(LoadedDll {
            handle: handle as *mut _,
        })
    }
}

#[cfg(not(target_os = "windows"))]
pub fn load_dll(_path: &Path) -> Result<LoadedDll> {
    anyhow::bail!("DLL loading only supported on Windows")
}
