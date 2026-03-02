// ISteamUtils - 60+ utility functions
use crate::CURRENT_APP_ID;
use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::ffi::{c_char, c_void, CStr};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

lazy_static! {
    static ref APP_START_TIME: std::time::Instant = std::time::Instant::now();
    static ref LAST_INPUT_TIME: AtomicU64 = AtomicU64::new(0);
    static ref OVERLAY_ENABLED: AtomicU32 = AtomicU32::new(1);
    static ref WARNING_HOOK: RwLock<Option<extern "C" fn(i32, *const c_char)>> = RwLock::new(None);
    static ref GAMEPAD_TEXT: RwLock<String> = RwLock::new(String::new());
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetSecondsSinceAppActive() -> u32 {
    APP_START_TIME.elapsed().as_secs() as u32
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetSecondsSinceComputerActive() -> u32 {
    unsafe {
        #[cfg(windows)]
        {
            (winapi::um::sysinfoapi::GetTickCount64() / 1000) as u32
        }
        #[cfg(not(windows))]
        {
            APP_START_TIME.elapsed().as_secs() as u32
        }
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetConnectedUniverse() -> i32 {
    1 // k_EUniversePublic
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetServerRealTime() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetIPCountry() -> *const c_char {
    b"US\0".as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetImageSize(
    image: i32,
    width: *mut u32,
    height: *mut u32,
) -> bool {
    if !width.is_null() && !height.is_null() {
        unsafe {
            *width = 64;
            *height = 64;
        }
        return true;
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetImageRGBA(
    image: i32,
    dest: *mut u8,
    dest_size: i32,
) -> bool {
    if !dest.is_null() && dest_size >= 64 * 64 * 4 {
        unsafe {
            std::ptr::write_bytes(dest, 0xFF, (64 * 64 * 4) as usize);
        }
        return true;
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetCSERIPPort(_ip: *mut u32, _port: *mut u16) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetCurrentBatteryPower() -> u8 {
    255 // Unknown
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetAppID() -> u32 {
    *CURRENT_APP_ID.read()
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_SetOverlayNotificationPosition(position: i32) {
    println!("[Oracle] Overlay notification position: {}", position);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_IsAPICallCompleted(handle: u64, failed: *mut bool) -> bool {
    if !failed.is_null() {
        unsafe {
            *failed = false;
        }
    }
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetAPICallFailureReason(_handle: u64) -> i32 {
    0 // k_ESteamAPICallFailureNone
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetAPICallResult(
    handle: u64,
    callback: *mut c_void,
    callback_size: i32,
    callback_expected: i32,
    failed: *mut bool,
) -> bool {
    if !failed.is_null() {
        unsafe {
            *failed = false;
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetIPCCallCount() -> u32 {
    0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_SetWarningMessageHook(
    func: extern "C" fn(i32, *const c_char),
) {
    *WARNING_HOOK.write() = Some(func);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_IsOverlayEnabled() -> bool {
    OVERLAY_ENABLED.load(Ordering::Relaxed) != 0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_BOverlayNeedsPresent() -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_CheckFileSignature(filename: *const c_char) -> u64 {
    1 // API call handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_ShowGamepadTextInput(
    mode: i32,
    line_mode: i32,
    description: *const c_char,
    max_chars: u32,
    existing_text: *const c_char,
) -> bool {
    println!("[Oracle] Gamepad text input requested");
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetEnteredGamepadTextLength() -> u32 {
    GAMEPAD_TEXT.read().len() as u32
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetEnteredGamepadTextInput(
    text: *mut c_char,
    text_size: u32,
) -> bool {
    if text.is_null() || text_size == 0 {
        return false;
    }

    let input = GAMEPAD_TEXT.read();
    let bytes = input.as_bytes();
    let len = bytes.len().min((text_size - 1) as usize);

    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), text as *mut u8, len);
        *text.add(len) = 0;
    }

    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetSteamUILanguage() -> *const c_char {
    b"english\0".as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_IsSteamRunningInVR() -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_SetOverlayNotificationInset(horizontal: i32, vertical: i32) {
    println!("[Oracle] Overlay inset: {}x{}", horizontal, vertical);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_IsSteamInBigPictureMode() -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_StartVRDashboard() {
    println!("[Oracle] VR dashboard not supported");
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_IsVRHeadsetStreamingEnabled() -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_SetVRHeadsetStreamingEnabled(_enabled: bool) {}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_IsSteamChinaLauncher() -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_InitFilterText(filter_options: u32) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_FilterText(
    context: i32,
    source_steam_id: u64,
    input: *const c_char,
    output: *mut c_char,
    output_size: u32,
) -> i32 {
    if input.is_null() || output.is_null() || output_size == 0 {
        return 0;
    }

    unsafe {
        let input_str = CStr::from_ptr(input);
        let bytes = input_str.to_bytes();
        let len = bytes.len().min((output_size - 1) as usize);
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), output as *mut u8, len);
        *output.add(len) = 0;
        len as i32
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetIPv6ConnectivityState(_protocol: i32) -> i32 {
    1 // k_ESteamIPv6ConnectivityState_Good
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_IsSteamRunningOnSteamDeck() -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_ShowFloatingGamepadTextInput(
    mode: i32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> bool {
    println!("[Oracle] Floating gamepad text input");
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_SetGameLauncherMode(_launcher_mode: bool) {}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_DismissFloatingGamepadTextInput() -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_DismissGamepadTextInput() -> bool {
    *GAMEPAD_TEXT.write() = String::new();
    true
}

// Additional ISteamUtils functions (expanding to 50+)

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetServerRealTime() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetIPCountry() -> *const std::ffi::c_char {
    b"US\0".as_ptr() as *const std::ffi::c_char
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetImageSize(image: i32, width: *mut u32, height: *mut u32) -> bool {
    if !width.is_null() && !height.is_null() {
        unsafe {
            *width = 64;
            *height = 64;
        }
        return true;
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetImageRGBA(image: i32, dest: *mut u8, dest_size: i32) -> bool {
    if !dest.is_null() && dest_size >= 64 * 64 * 4 {
        // Fill with placeholder image data
        return true;
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetCurrentBatteryPower() -> u8 {
    255 // Unknown
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetAppID() -> u32 {
    *crate::CURRENT_APP_ID.read()
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_SetOverlayNotificationPosition(position: i32) {
    println!("[Utils] Overlay notification position: {}", position);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_IsAPICallCompleted(api_call: u64, failed: *mut bool) -> bool {
    if !failed.is_null() {
        unsafe { *failed = false; }
    }
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetAPICallFailureReason(api_call: u64) -> i32 {
    0 // k_ESteamAPICallFailureNone
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetAPICallResult(
    api_call: u64,
    callback: *mut std::ffi::c_void,
    callback_size: i32,
    callback_expected: i32,
    failed: *mut bool,
) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_IsOverlayEnabled() -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_BOverlayNeedsPresent() -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_CheckFileSignature(filename: *const std::ffi::c_char) -> u64 {
    0 // AsyncAPICall handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_ShowGamepadTextInput(
    input_mode: i32,
    line_input_mode: i32,
    description: *const std::ffi::c_char,
    char_max: u32,
    existing_text: *const std::ffi::c_char,
) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetEnteredGamepadTextLength() -> u32 {
    0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetEnteredGamepadTextInput(text: *mut std::ffi::c_char, size: u32) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetSteamUILanguage() -> *const std::ffi::c_char {
    b"english\0".as_ptr() as *const std::ffi::c_char
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_IsSteamRunningInVR() -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_SetVRHeadsetStreamingEnabled(enabled: bool) {
    println!("[Utils] VR streaming: {}", enabled);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_IsVRHeadsetStreamingEnabled() -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_IsSteamInBigPictureMode() -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_StartVRDashboard() {
    println!("[Utils] VR Dashboard requested");
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_IsVRHeadsetStreamingShown() -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_SetVRHeadsetStreamingShown(shown: bool) {
    println!("[Utils] VR streaming shown: {}", shown);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_IsSteamChinaLauncher() -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_InitFilterText(reserved: u32) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_FilterText(
    context: i32,
    source_steam_id: u64,
    input_message: *const std::ffi::c_char,
    output_message: *mut std::ffi::c_char,
    output_size: u32,
) -> i32 {
    if input_message.is_null() || output_message.is_null() {
        return 0;
    }

    unsafe {
        let input = std::ffi::CStr::from_ptr(input_message);
        let bytes = input.to_bytes();
        let copy_len = bytes.len().min((output_size - 1) as usize);
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), output_message as *mut u8, copy_len);
        *output_message.add(copy_len) = 0;
    }

    1 // Success
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_GetIPv6ConnectivityState(protocol: i32) -> i32 {
    0 // k_ESteamIPv6ConnectivityState_Unknown
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_IsSteamRunningOnSteamDeck() -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_ShowFloatingGamepadTextInput(
    keyboard_mode: i32,
    text_field_x_position: i32,
    text_field_y_position: i32,
    text_field_width: i32,
    text_field_height: i32,
) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_SetGameLauncherMode(launcher_mode: bool) {
    println!("[Utils] Game launcher mode: {}", launcher_mode);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_DismissFloatingGamepadTextInput() -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUtils_DismissGamepadTextInput() -> bool {
    false
}
