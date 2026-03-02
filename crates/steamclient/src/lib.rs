pub mod bootstrap;
pub mod client;
pub mod exports;
pub mod interfaces;

use once_cell::sync::Lazy;
use parking_lot::RwLock;

static CLIENT_HANDLE: Lazy<RwLock<Option<usize>>> = Lazy::new(|| RwLock::new(None));

#[no_mangle]
pub extern "C" fn CreateInterface(name: *const i8, return_code: *mut i32) -> *mut std::ffi::c_void {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn Steam_GetHSteamUserCurrent() -> u32 {
    1
}

#[no_mangle]
pub extern "C" fn Steam_GetHSteamPipe() -> u32 {
    1
}
