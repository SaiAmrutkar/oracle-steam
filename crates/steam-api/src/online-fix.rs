// crates/steam-api/src/onlinefix.rs

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn SteamAPI_InitSafe() -> bool {
    crate::exports::SteamAPI_Init()
}

#[no_mangle]
pub extern "C" fn SteamAPI_IsSteamRunning() -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_GetSteamInstallPath() -> *const c_char {
    static mut PATH: Vec<u8> = Vec::new();

    unsafe {
        if PATH.is_empty() {
            let path = std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            PATH = path.into_bytes();
            PATH.push(0);
        }

        PATH.as_ptr() as *const c_char
    }
}

#[no_mangle]
pub extern "C" fn SteamInternal_FindOrCreateUserInterface(
    hSteamUser: i32,
    pszVersion: *const c_char,
) -> *mut std::ffi::c_void {
    println!("[OnlineFix] FindOrCreateUserInterface");
    0x1 as *mut std::ffi::c_void
}

#[no_mangle]
pub extern "C" fn SteamInternal_CreateInterface(version: *const c_char) -> *mut std::ffi::c_void {
    if version.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let ver = CStr::from_ptr(version).to_str().unwrap_or("");
        println!("[OnlineFix] CreateInterface: {}", ver);
    }

    0x1 as *mut std::ffi::c_void
}
