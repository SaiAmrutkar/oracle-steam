// crates/steam-api/src/video.rs
// ISteamVideo - Video playback and broadcasting

use std::ffi::c_char;

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamVideo_GetVideoURL(app_id: u32) {
    println!("[Video] Getting video URL for app: {}", app_id);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamVideo_IsBroadcasting(num_viewers: *mut i32) -> bool {
    if !num_viewers.is_null() {
        unsafe {
            *num_viewers = 0;
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamVideo_GetOPFSettings(app_id: u32) {
    println!("[Video] Getting OPF settings for app: {}", app_id);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamVideo_GetOPFStringForApp(
    app_id: u32,
    buffer: *mut c_char,
    buffer_size: *mut i32,
) -> bool {
    if buffer.is_null() || buffer_size.is_null() {
        return false;
    }

    let opf_string = "Oracle Steam Video";
    let bytes = opf_string.as_bytes();

    unsafe {
        let available = *buffer_size as usize;
        if available > bytes.len() {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, bytes.len());
            *(buffer as *mut u8).add(bytes.len()) = 0;
            *buffer_size = (bytes.len() + 1) as i32;
            return true;
        }
    }

    false
}
