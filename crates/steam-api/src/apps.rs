use std::ffi::c_char;

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamApps_BIsSubscribedApp(_app_id: u32) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamApps_BIsDlcInstalled(_app_id: u32) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamApps_GetCurrentGameLanguage() -> *const c_char {
    b"english\0".as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamApps_GetAppInstallDir(
    _app_id: u32,
    _folder: *mut c_char,
    _buffer_size: u32,
) -> u32 {
    0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamApps_GetCurrentBetaName(
    _name: *mut c_char,
    _name_buffer_size: i32,
) -> bool {
    false
}
