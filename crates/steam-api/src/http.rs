// crates/steam-api/src/http.rs
// Complete ISteamHTTP implementation

use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr};

pub type HTTPRequestHandle = u32;
pub type HTTPCookieContainerHandle = u32;

lazy_static! {
    static ref HTTP_REQUESTS: RwLock<HashMap<HTTPRequestHandle, HttpRequest>> =
        RwLock::new(HashMap::new());
    static ref COOKIE_CONTAINERS: RwLock<HashMap<HTTPCookieContainerHandle, CookieContainer>> =
        RwLock::new(HashMap::new());
    static ref NEXT_HANDLE: RwLock<u32> = RwLock::new(1);
}

struct HttpRequest {
    handle: HTTPRequestHandle,
    url: String,
    method: String,
    headers: HashMap<String, String>,
    params: HashMap<String, String>,
    body: Vec<u8>,
    response_body: Vec<u8>,
    response_headers: HashMap<String, String>,
    status_code: u32,
    completed: bool,
    cookie_container: HTTPCookieContainerHandle,
}

struct CookieContainer {
    handle: HTTPCookieContainerHandle,
    cookies: HashMap<String, String>,
}

fn next_handle() -> u32 {
    let mut h = NEXT_HANDLE.write();
    let val = *h;
    *h += 1;
    val
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_CreateHTTPRequest(
    method: i32,
    url: *const c_char,
) -> HTTPRequestHandle {
    if url.is_null() {
        return 0;
    }

    let url_str = unsafe { CStr::from_ptr(url).to_str().unwrap_or("").to_string() };

    let method_str = match method {
        0 => "GET",
        1 => "HEAD",
        2 => "POST",
        3 => "PUT",
        4 => "DELETE",
        5 => "OPTIONS",
        6 => "PATCH",
        _ => "GET",
    }
    .to_string();

    let handle = next_handle();

    let request = HttpRequest {
        handle,
        url: url_str.clone(),
        method: method_str,
        headers: HashMap::new(),
        params: HashMap::new(),
        body: Vec::new(),
        response_body: Vec::new(),
        response_headers: HashMap::new(),
        status_code: 0,
        completed: false,
        cookie_container: 0,
    };

    HTTP_REQUESTS.write().insert(handle, request);

    println!("[HTTP] Created request {}: {}", handle, url_str);
    handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_SetHTTPRequestContextValue(
    request: HTTPRequestHandle,
    context_value: u64,
) -> bool {
    HTTP_REQUESTS.read().contains_key(&request)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_SetHTTPRequestNetworkActivityTimeout(
    request: HTTPRequestHandle,
    timeout_seconds: u32,
) -> bool {
    HTTP_REQUESTS.read().contains_key(&request)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_SetHTTPRequestHeaderValue(
    request: HTTPRequestHandle,
    header_name: *const c_char,
    header_value: *const c_char,
) -> bool {
    if header_name.is_null() || header_value.is_null() {
        return false;
    }

    unsafe {
        if let (Ok(name), Ok(value)) = (
            CStr::from_ptr(header_name).to_str(),
            CStr::from_ptr(header_value).to_str(),
        ) {
            if let Some(req) = HTTP_REQUESTS.write().get_mut(&request) {
                req.headers.insert(name.to_string(), value.to_string());
                return true;
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_SetHTTPRequestGetOrPostParameter(
    request: HTTPRequestHandle,
    param_name: *const c_char,
    param_value: *const c_char,
) -> bool {
    if param_name.is_null() || param_value.is_null() {
        return false;
    }

    unsafe {
        if let (Ok(name), Ok(value)) = (
            CStr::from_ptr(param_name).to_str(),
            CStr::from_ptr(param_value).to_str(),
        ) {
            if let Some(req) = HTTP_REQUESTS.write().get_mut(&request) {
                req.params.insert(name.to_string(), value.to_string());
                return true;
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_SendHTTPRequest(
    request: HTTPRequestHandle,
    call_handle: *mut u64,
) -> bool {
    let mut requests = HTTP_REQUESTS.write();

    if let Some(req) = requests.get_mut(&request) {
        println!("[HTTP] Sending {} request to: {}", req.method, req.url);

        // Simulate async request completion
        req.completed = true;
        req.status_code = 200;
        req.response_body = b"{\"success\": true}".to_vec();
        req.response_headers
            .insert("Content-Type".to_string(), "application/json".to_string());

        if !call_handle.is_null() {
            unsafe {
                *call_handle = request as u64;
            }
        }

        return true;
    }

    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_SendHTTPRequestAndStreamResponse(
    request: HTTPRequestHandle,
    call_handle: *mut u64,
) -> bool {
    SteamAPI_ISteamHTTP_SendHTTPRequest(request, call_handle)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_DeferHTTPRequest(request: HTTPRequestHandle) -> bool {
    HTTP_REQUESTS.read().contains_key(&request)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_PrioritizeHTTPRequest(request: HTTPRequestHandle) -> bool {
    HTTP_REQUESTS.read().contains_key(&request)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_GetHTTPResponseHeaderSize(
    request: HTTPRequestHandle,
    header_name: *const c_char,
    response_header_size: *mut u32,
) -> bool {
    if header_name.is_null() || response_header_size.is_null() {
        return false;
    }

    unsafe {
        if let Ok(name) = CStr::from_ptr(header_name).to_str() {
            let requests = HTTP_REQUESTS.read();
            if let Some(req) = requests.get(&request) {
                if let Some(value) = req.response_headers.get(name) {
                    *response_header_size = value.len() as u32;
                    return true;
                }
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_GetHTTPResponseHeaderValue(
    request: HTTPRequestHandle,
    header_name: *const c_char,
    header_value_buffer: *mut u8,
    buffer_size: u32,
) -> bool {
    if header_name.is_null() || header_value_buffer.is_null() {
        return false;
    }

    unsafe {
        if let Ok(name) = CStr::from_ptr(header_name).to_str() {
            let requests = HTTP_REQUESTS.read();
            if let Some(req) = requests.get(&request) {
                if let Some(value) = req.response_headers.get(name) {
                    let bytes = value.as_bytes();
                    let len = bytes.len().min((buffer_size - 1) as usize);
                    std::ptr::copy_nonoverlapping(bytes.as_ptr(), header_value_buffer, len);
                    *header_value_buffer.add(len) = 0;
                    return true;
                }
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_GetHTTPResponseBodySize(
    request: HTTPRequestHandle,
    body_size: *mut u32,
) -> bool {
    if body_size.is_null() {
        return false;
    }

    let requests = HTTP_REQUESTS.read();
    if let Some(req) = requests.get(&request) {
        unsafe {
            *body_size = req.response_body.len() as u32;
        }
        return true;
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_GetHTTPResponseBodyData(
    request: HTTPRequestHandle,
    body_data_buffer: *mut u8,
    buffer_size: u32,
) -> bool {
    if body_data_buffer.is_null() {
        return false;
    }

    let requests = HTTP_REQUESTS.read();
    if let Some(req) = requests.get(&request) {
        let len = req.response_body.len().min(buffer_size as usize);
        unsafe {
            std::ptr::copy_nonoverlapping(req.response_body.as_ptr(), body_data_buffer, len);
        }
        return true;
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_GetHTTPStreamingResponseBodyData(
    request: HTTPRequestHandle,
    offset: u32,
    body_data_buffer: *mut u8,
    buffer_size: u32,
) -> bool {
    if body_data_buffer.is_null() {
        return false;
    }

    let requests = HTTP_REQUESTS.read();
    if let Some(req) = requests.get(&request) {
        let start = offset as usize;
        if start < req.response_body.len() {
            let remaining = &req.response_body[start..];
            let len = remaining.len().min(buffer_size as usize);
            unsafe {
                std::ptr::copy_nonoverlapping(remaining.as_ptr(), body_data_buffer, len);
            }
            return true;
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_ReleaseHTTPRequest(request: HTTPRequestHandle) -> bool {
    HTTP_REQUESTS.write().remove(&request).is_some()
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_GetHTTPDownloadProgressPct(
    request: HTTPRequestHandle,
    percent: *mut f32,
) -> bool {
    if percent.is_null() {
        return false;
    }

    unsafe {
        *percent = 100.0;
    }
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_SetHTTPRequestRawPostBody(
    request: HTTPRequestHandle,
    content_type: *const c_char,
    body: *const u8,
    body_len: u32,
) -> bool {
    if body.is_null() {
        return false;
    }

    let mut requests = HTTP_REQUESTS.write();
    if let Some(req) = requests.get_mut(&request) {
        let body_vec = unsafe { std::slice::from_raw_parts(body, body_len as usize).to_vec() };
        req.body = body_vec;

        if !content_type.is_null() {
            unsafe {
                if let Ok(ct) = CStr::from_ptr(content_type).to_str() {
                    req.headers
                        .insert("Content-Type".to_string(), ct.to_string());
                }
            }
        }
        return true;
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_CreateCookieContainer(
    allow_responses_to_modify: bool,
) -> HTTPCookieContainerHandle {
    let handle = next_handle();

    let container = CookieContainer {
        handle,
        cookies: HashMap::new(),
    };

    COOKIE_CONTAINERS.write().insert(handle, container);
    handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_ReleaseCookieContainer(
    cookie_handle: HTTPCookieContainerHandle,
) -> bool {
    COOKIE_CONTAINERS.write().remove(&cookie_handle).is_some()
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_SetCookie(
    cookie_handle: HTTPCookieContainerHandle,
    host: *const c_char,
    url: *const c_char,
    cookie: *const c_char,
) -> bool {
    if host.is_null() || cookie.is_null() {
        return false;
    }

    unsafe {
        if let (Ok(host_str), Ok(cookie_str)) = (
            CStr::from_ptr(host).to_str(),
            CStr::from_ptr(cookie).to_str(),
        ) {
            if let Some(container) = COOKIE_CONTAINERS.write().get_mut(&cookie_handle) {
                container
                    .cookies
                    .insert(host_str.to_string(), cookie_str.to_string());
                return true;
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_SetHTTPRequestCookieContainer(
    request: HTTPRequestHandle,
    cookie_handle: HTTPCookieContainerHandle,
) -> bool {
    if let Some(req) = HTTP_REQUESTS.write().get_mut(&request) {
        req.cookie_container = cookie_handle;
        return true;
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_SetHTTPRequestUserAgentInfo(
    request: HTTPRequestHandle,
    user_agent: *const c_char,
) -> bool {
    if user_agent.is_null() {
        return false;
    }

    unsafe {
        if let Ok(ua) = CStr::from_ptr(user_agent).to_str() {
            if let Some(req) = HTTP_REQUESTS.write().get_mut(&request) {
                req.headers.insert("User-Agent".to_string(), ua.to_string());
                return true;
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_SetHTTPRequestRequiresVerifiedCertificate(
    request: HTTPRequestHandle,
    requires_verified_certificate: bool,
) -> bool {
    HTTP_REQUESTS.read().contains_key(&request)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_SetHTTPRequestAbsoluteTimeoutMS(
    request: HTTPRequestHandle,
    milliseconds: u32,
) -> bool {
    HTTP_REQUESTS.read().contains_key(&request)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamHTTP_GetHTTPRequestWasTimedOut(
    request: HTTPRequestHandle,
    was_timed_out: *mut bool,
) -> bool {
    if was_timed_out.is_null() {
        return false;
    }

    unsafe {
        *was_timed_out = false;
    }
    true
}

// ============================================================================
// SCREENSHOTS (crates/steam-api/src/screenshots.rs)
// ============================================================================

// Screenshots implementation
use std::path::PathBuf;

pub type ScreenshotHandle = u32;

lazy_static! {
    static ref SCREENSHOTS: RwLock<HashMap<ScreenshotHandle, Screenshot>> =
        RwLock::new(HashMap::new());
    static ref SCREENSHOT_HANDLE: RwLock<u32> = RwLock::new(1);
    static ref SCREENSHOTS_DIR: RwLock<PathBuf> = RwLock::new(
        std::env::current_dir()
            .unwrap_or_default()
            .join("oracle_data")
            .join("screenshots")
    );
}

struct Screenshot {
    handle: ScreenshotHandle,
    filename: String,
    width: u32,
    height: u32,
    tagged: bool,
    location: String,
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamScreenshots_WriteScreenshot(
    rgb: *const c_void,
    rgb_size: u32,
    width: i32,
    height: i32,
) -> ScreenshotHandle {
    if rgb.is_null() {
        return 0;
    }

    let handle = {
        let mut h = SCREENSHOT_HANDLE.write();
        let val = *h;
        *h += 1;
        val
    };

    let filename = format!("screenshot_{}.png", handle);

    let screenshot = Screenshot {
        handle,
        filename: filename.clone(),
        width: width as u32,
        height: height as u32,
        tagged: false,
        location: String::new(),
    };

    SCREENSHOTS.write().insert(handle, screenshot);

    println!(
        "[Screenshots] Captured: {}x{} ({})",
        width, height, filename
    );
    handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamScreenshots_AddScreenshotToLibrary(
    filename: *const c_char,
    thumbnail_filename: *const c_char,
    width: i32,
    height: i32,
) -> ScreenshotHandle {
    if filename.is_null() {
        return 0;
    }

    let handle = {
        let mut h = SCREENSHOT_HANDLE.write();
        let val = *h;
        *h += 1;
        val
    };

    let filename_str = unsafe {
        CStr::from_ptr(filename)
            .to_str()
            .unwrap_or("screenshot.png")
            .to_string()
    };

    let screenshot = Screenshot {
        handle,
        filename: filename_str.clone(),
        width: width as u32,
        height: height as u32,
        tagged: false,
        location: String::new(),
    };

    SCREENSHOTS.write().insert(handle, screenshot);

    println!("[Screenshots] Added to library: {}", filename_str);
    handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamScreenshots_TriggerScreenshot() {
    println!("[Screenshots] Screenshot triggered");
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamScreenshots_HookScreenshots(hook: bool) {
    println!("[Screenshots] Hook: {}", hook);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamScreenshots_SetLocation(
    screenshot: ScreenshotHandle,
    location: *const c_char,
) -> bool {
    if location.is_null() {
        return false;
    }

    unsafe {
        if let Ok(loc) = CStr::from_ptr(location).to_str() {
            if let Some(ss) = SCREENSHOTS.write().get_mut(&screenshot) {
                ss.location = loc.to_string();
                return true;
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamScreenshots_TagUser(
    screenshot: ScreenshotHandle,
    steam_id: u64,
) -> bool {
    if let Some(ss) = SCREENSHOTS.write().get_mut(&screenshot) {
        ss.tagged = true;
        return true;
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamScreenshots_TagPublishedFile(
    screenshot: ScreenshotHandle,
    published_file_id: u64,
) -> bool {
    SCREENSHOTS.read().contains_key(&screenshot)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamScreenshots_IsScreenshotsHooked() -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamScreenshots_AddVRScreenshotToLibrary(
    vr_type: i32,
    filename: *const c_char,
    vr_filename: *const c_char,
) -> ScreenshotHandle {
    SteamAPI_ISteamScreenshots_AddScreenshotToLibrary(filename, vr_filename, 1920, 1080)
}
