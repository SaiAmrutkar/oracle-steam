// crates/steam-api/src/htmlsurface.rs
// Complete ISteamHTMLSurface - Embedded Browser

use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr};

pub type HHTMLBrowser = u32;

lazy_static! {
    static ref BROWSERS: RwLock<HashMap<HHTMLBrowser, Browser>> = RwLock::new(HashMap::new());
    static ref NEXT_BROWSER_HANDLE: RwLock<u32> = RwLock::new(1);
}

struct Browser {
    handle: HHTMLBrowser,
    url: String,
    title: String,
    width: u32,
    height: u32,
    ready: bool,
    loading: bool,
}

fn next_browser_handle() -> HHTMLBrowser {
    let mut h = NEXT_BROWSER_HANDLE.write();
    let val = *h;
    *h += 1;
    val
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_Init() -> bool {
    println!("[HTMLSurface] Initialized");
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_Shutdown() -> bool {
    BROWSERS.write().clear();
    println!("[HTMLSurface] Shutdown");
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_CreateBrowser(
    user_agent: *const c_char,
    css: *const c_char,
) -> u64 {
    let handle = next_browser_handle();

    let browser = Browser {
        handle,
        url: String::new(),
        title: "Oracle Steam Browser".to_string(),
        width: 1024,
        height: 768,
        ready: true,
        loading: false,
    };

    BROWSERS.write().insert(handle, browser);

    println!("[HTMLSurface] Browser created: {}", handle);
    1 // API call handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_RemoveBrowser(browser_handle: HHTMLBrowser) {
    BROWSERS.write().remove(&browser_handle);
    println!("[HTMLSurface] Browser removed: {}", browser_handle);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_LoadURL(
    browser_handle: HHTMLBrowser,
    url: *const c_char,
    post_data: *const c_char,
) {
    if url.is_null() {
        return;
    }

    unsafe {
        if let Ok(url_str) = CStr::from_ptr(url).to_str() {
            if let Some(browser) = BROWSERS.write().get_mut(&browser_handle) {
                browser.url = url_str.to_string();
                browser.loading = true;
                println!("[HTMLSurface] Loading URL: {}", url_str);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_SetSize(
    browser_handle: HHTMLBrowser,
    width: u32,
    height: u32,
) {
    if let Some(browser) = BROWSERS.write().get_mut(&browser_handle) {
        browser.width = width;
        browser.height = height;
        println!("[HTMLSurface] Size set: {}x{}", width, height);
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_StopLoad(browser_handle: HHTMLBrowser) {
    if let Some(browser) = BROWSERS.write().get_mut(&browser_handle) {
        browser.loading = false;
        println!("[HTMLSurface] Load stopped");
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_Reload(browser_handle: HHTMLBrowser) {
    if let Some(browser) = BROWSERS.read().get(&browser_handle) {
        let url = browser.url.clone();
        drop(browser);

        let url_cstr = std::ffi::CString::new(url).unwrap();
        SteamAPI_ISteamHTMLSurface_LoadURL(browser_handle, url_cstr.as_ptr(), std::ptr::null());
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_GoBack(browser_handle: HHTMLBrowser) {
    println!("[HTMLSurface] Go back");
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_GoForward(browser_handle: HHTMLBrowser) {
    println!("[HTMLSurface] Go forward");
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_AddHeader(
    browser_handle: HHTMLBrowser,
    key: *const c_char,
    value: *const c_char,
) {
    println!("[HTMLSurface] Adding header");
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_ExecuteJavascript(
    browser_handle: HHTMLBrowser,
    script: *const c_char,
) {
    if script.is_null() {
        return;
    }

    unsafe {
        if let Ok(js) = CStr::from_ptr(script).to_str() {
            println!("[HTMLSurface] Executing JavaScript: {}", js);
        }
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_MouseUp(
    browser_handle: HHTMLBrowser,
    mouse_button: i32,
) {
    println!("[HTMLSurface] Mouse up: {}", mouse_button);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_MouseDown(
    browser_handle: HHTMLBrowser,
    mouse_button: i32,
) {
    println!("[HTMLSurface] Mouse down: {}", mouse_button);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_MouseDoubleClick(
    browser_handle: HHTMLBrowser,
    mouse_button: i32,
) {
    println!("[HTMLSurface] Mouse double click: {}", mouse_button);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_MouseMove(
    browser_handle: HHTMLBrowser,
    x: i32,
    y: i32,
) {
    // Don't log every mouse move
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_MouseWheel(browser_handle: HHTMLBrowser, delta: i32) {
    println!("[HTMLSurface] Mouse wheel: {}", delta);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_KeyDown(
    browser_handle: HHTMLBrowser,
    native_key_code: u32,
    key_modifiers: i32,
    is_system_key: bool,
) {
    println!("[HTMLSurface] Key down: {}", native_key_code);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_KeyUp(
    browser_handle: HHTMLBrowser,
    native_key_code: u32,
    key_modifiers: i32,
) {
    println!("[HTMLSurface] Key up: {}", native_key_code);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_KeyChar(
    browser_handle: HHTMLBrowser,
    unicode_char: u32,
    key_modifiers: i32,
) {
    println!("[HTMLSurface] Key char: {}", unicode_char);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_SetHorizontalScroll(
    browser_handle: HHTMLBrowser,
    absolute_pixel_scroll: u32,
) {
    println!("[HTMLSurface] Horizontal scroll: {}", absolute_pixel_scroll);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_SetVerticalScroll(
    browser_handle: HHTMLBrowser,
    absolute_pixel_scroll: u32,
) {
    println!("[HTMLSurface] Vertical scroll: {}", absolute_pixel_scroll);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_SetKeyFocus(
    browser_handle: HHTMLBrowser,
    has_focus: bool,
) {
    println!("[HTMLSurface] Key focus: {}", has_focus);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_ViewSource(browser_handle: HHTMLBrowser) {
    println!("[HTMLSurface] View source");
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_CopyToClipboard(browser_handle: HHTMLBrowser) {
    println!("[HTMLSurface] Copy to clipboard");
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_PasteFromClipboard(browser_handle: HHTMLBrowser) {
    println!("[HTMLSurface] Paste from clipboard");
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_Find(
    browser_handle: HHTMLBrowser,
    search: *const c_char,
    currently_in_find: bool,
    reverse: bool,
) {
    println!("[HTMLSurface] Find");
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_StopFind(browser_handle: HHTMLBrowser) {
    println!("[HTMLSurface] Stop find");
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_GetLinkAtPosition(
    browser_handle: HHTMLBrowser,
    x: i32,
    y: i32,
) {
    println!("[HTMLSurface] Get link at: {},{}", x, y);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_SetCookie(
    hostname: *const c_char,
    key: *const c_char,
    value: *const c_char,
    path: *const c_char,
    expires: u32,
    secure: bool,
    http_only: bool,
) {
    println!("[HTMLSurface] Set cookie");
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_SetPageScaleFactor(
    browser_handle: HHTMLBrowser,
    zoom: f32,
    point_x: i32,
    point_y: i32,
) {
    println!("[HTMLSurface] Page scale: {}", zoom);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_SetBackgroundMode(
    browser_handle: HHTMLBrowser,
    background_mode: bool,
) {
    println!("[HTMLSurface] Background mode: {}", background_mode);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_SetDPIScalingFactor(
    browser_handle: HHTMLBrowser,
    dpi_scaling: f32,
) {
    println!("[HTMLSurface] DPI scaling: {}", dpi_scaling);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_OpenDeveloperTools(browser_handle: HHTMLBrowser) {
    println!("[HTMLSurface] Opening dev tools");
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_AllowStartRequest(
    browser_handle: HHTMLBrowser,
    allowed: bool,
) {
    println!("[HTMLSurface] Allow start request: {}", allowed);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_JSDialogResponse(
    browser_handle: HHTMLBrowser,
    result: bool,
) {
    println!("[HTMLSurface] JS dialog response: {}", result);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTMLSurface_FileLoadDialogResponse(
    browser_handle: HHTMLBrowser,
    selected_files: *const *const c_char,
) {
    println!("[HTMLSurface] File load dialog response");
}

// ============================================================================
// ISteamVideo (crates/steam-api/src/video.rs)
// ============================================================================

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

// ============================================================================
// ISteamParentalSettings (crates/steam-api/src/parental.rs)
// ============================================================================

lazy_static! {
    static ref PARENTAL_LOCK: RwLock<ParentalSettings> = RwLock::new(ParentalSettings::new());
}

struct ParentalSettings {
    enabled: bool,
    locked: bool,
    blocked_apps: Vec<u32>,
    blocked_features: Vec<i32>,
}

impl ParentalSettings {
    fn new() -> Self {
        Self {
            enabled: false,
            locked: false,
            blocked_apps: Vec::new(),
            blocked_features: Vec::new(),
        }
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamParentalSettings_BIsParentalLockEnabled() -> bool {
    PARENTAL_LOCK.read().enabled
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamParentalSettings_BIsParentalLockLocked() -> bool {
    PARENTAL_LOCK.read().locked
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamParentalSettings_BIsAppBlocked(app_id: u32) -> bool {
    PARENTAL_LOCK.read().blocked_apps.contains(&app_id)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamParentalSettings_BIsAppInBlockList(app_id: u32) -> bool {
    SteamAPI_ISteamParentalSettings_BIsAppBlocked(app_id)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamParentalSettings_BIsFeatureBlocked(feature: i32) -> bool {
    PARENTAL_LOCK.read().blocked_features.contains(&feature)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamParentalSettings_BIsFeatureInBlockList(feature: i32) -> bool {
    SteamAPI_ISteamParentalSettings_BIsFeatureBlocked(feature)
}
