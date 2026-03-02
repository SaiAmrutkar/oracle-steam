// ISteamRemoteStorage - Cloud saves, Workshop, Screenshots
use crate::STEAM_CLIENT;
use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr};
use std::path::PathBuf;

const MAX_CLOUD_FILE_SIZE: usize = 200 * 1024 * 1024; // 200 MB

lazy_static! {
    static ref CLOUD_FILES: RwLock<HashMap<String, Vec<u8>>> = RwLock::new(HashMap::new());
    static ref CLOUD_QUOTA: RwLock<(u64, u64)> = RwLock::new((200 * 1024 * 1024, 0)); // (total, used)
    static ref STORAGE_PATH: RwLock<PathBuf> = RwLock::new(
        std::env::current_dir()
            .unwrap_or_default()
            .join("oracle_data")
            .join("remote_storage")
    );
}

// ============================================================================
// FILE OPERATIONS
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_FileWrite(
    file: *const c_char,
    data: *const c_void,
    data_size: i32,
) -> bool {
    if file.is_null() || data.is_null() || data_size < 1 {
        return false;
    }

    unsafe {
        let filename = match CStr::from_ptr(file).to_str() {
            Ok(s) => s,
            Err(_) => return false,
        };

        // Check quota
        let (total, used) = *CLOUD_QUOTA.read();
        if used + data_size as u64 > total {
            println!("[Oracle] Cloud storage quota exceeded");
            return false;
        }

        // Copy data
        let data_vec = std::slice::from_raw_parts(data as *const u8, data_size as usize).to_vec();

        // Store in memory
        CLOUD_FILES
            .write()
            .insert(filename.to_string(), data_vec.clone());

        // Also save to disk
        let path = STORAGE_PATH.read().join(filename);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&path, &data_vec).ok();

        // Update quota
        CLOUD_QUOTA.write().1 += data_size as u64;

        println!(
            "[Oracle] Cloud file written: {} ({} bytes)",
            filename, data_size
        );
        true
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_FileRead(
    file: *const c_char,
    data: *mut c_void,
    data_to_read: i32,
) -> i32 {
    if file.is_null() || data.is_null() || data_to_read < 1 {
        return 0;
    }

    unsafe {
        let filename = match CStr::from_ptr(file).to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let files = CLOUD_FILES.read();
        if let Some(file_data) = files.get(filename) {
            let to_read = file_data.len().min(data_to_read as usize);
            std::ptr::copy_nonoverlapping(file_data.as_ptr(), data as *mut u8, to_read);
            println!("[Oracle] Cloud file read: {} ({} bytes)", filename, to_read);
            return to_read as i32;
        }

        // Try loading from disk
        let path = STORAGE_PATH.read().join(filename);
        if let Ok(file_data) = std::fs::read(&path) {
            let to_read = file_data.len().min(data_to_read as usize);
            std::ptr::copy_nonoverlapping(file_data.as_ptr(), data as *mut u8, to_read);

            // Cache it
            drop(files);
            CLOUD_FILES.write().insert(filename.to_string(), file_data);

            println!(
                "[Oracle] Cloud file read from disk: {} ({} bytes)",
                filename, to_read
            );
            return to_read as i32;
        }

        0
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_FileForget(file: *const c_char) -> bool {
    if file.is_null() {
        return false;
    }

    unsafe {
        let filename = match CStr::from_ptr(file).to_str() {
            Ok(s) => s,
            Err(_) => return false,
        };

        CLOUD_FILES.write().remove(filename);
        println!("[Oracle] Cloud file forgotten: {}", filename);
        true
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_FileDelete(file: *const c_char) -> bool {
    if file.is_null() {
        return false;
    }

    unsafe {
        let filename = match CStr::from_ptr(file).to_str() {
            Ok(s) => s,
            Err(_) => return false,
        };

        // Remove from memory
        if let Some(data) = CLOUD_FILES.write().remove(filename) {
            // Update quota
            CLOUD_QUOTA.write().1 = CLOUD_QUOTA.read().1.saturating_sub(data.len() as u64);
        }

        // Remove from disk
        let path = STORAGE_PATH.read().join(filename);
        std::fs::remove_file(&path).ok();

        println!("[Oracle] Cloud file deleted: {}", filename);
        true
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_FileExists(file: *const c_char) -> bool {
    if file.is_null() {
        return false;
    }

    unsafe {
        let filename = match CStr::from_ptr(file).to_str() {
            Ok(s) => s,
            Err(_) => return false,
        };

        if CLOUD_FILES.read().contains_key(filename) {
            return true;
        }

        let path = STORAGE_PATH.read().join(filename);
        path.exists()
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_FilePersisted(file: *const c_char) -> bool {
    SteamAPI_ISteamRemoteStorage_FileExists(file)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_GetFileSize(file: *const c_char) -> i32 {
    if file.is_null() {
        return 0;
    }

    unsafe {
        let filename = match CStr::from_ptr(file).to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        };

        if let Some(data) = CLOUD_FILES.read().get(filename) {
            return data.len() as i32;
        }

        let path = STORAGE_PATH.read().join(filename);
        if let Ok(metadata) = std::fs::metadata(&path) {
            return metadata.len() as i32;
        }

        0
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_GetFileTimestamp(file: *const c_char) -> i64 {
    if file.is_null() {
        return 0;
    }

    unsafe {
        let filename = match CStr::from_ptr(file).to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let path = STORAGE_PATH.read().join(filename);
        if let Ok(metadata) = std::fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                    return duration.as_secs() as i64;
                }
            }
        }

        0
    }
}

// ============================================================================
// FILE ENUMERATION
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_GetFileCount() -> i32 {
    CLOUD_FILES.read().len() as i32
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_GetFileNameAndSize(
    file_idx: i32,
    file_size: *mut i32,
) -> *const c_char {
    static mut FILENAME_BUFFER: [u8; 260] = [0; 260];

    let files = CLOUD_FILES.read();
    if let Some((filename, data)) = files.iter().nth(file_idx as usize) {
        if !file_size.is_null() {
            unsafe {
                *file_size = data.len() as i32;
            }
        }

        let bytes = filename.as_bytes();
        let len = bytes.len().min(259);
        unsafe {
            FILENAME_BUFFER[..len].copy_from_slice(&bytes[..len]);
            FILENAME_BUFFER[len] = 0;
            return FILENAME_BUFFER.as_ptr() as *const c_char;
        }
    }

    std::ptr::null()
}

// ============================================================================
// QUOTA MANAGEMENT
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_GetQuota(
    total_bytes: *mut u64,
    available_bytes: *mut u64,
) -> bool {
    if total_bytes.is_null() || available_bytes.is_null() {
        return false;
    }

    let (total, used) = *CLOUD_QUOTA.read();
    unsafe {
        *total_bytes = total;
        *available_bytes = total.saturating_sub(used);
    }

    true
}

// ============================================================================
// CLOUD SETTINGS
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_IsCloudEnabledForAccount() -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_IsCloudEnabledForApp() -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_SetCloudEnabledForApp(enabled: bool) {
    println!("[Oracle] Cloud sync for app: {}", enabled);
}

// ============================================================================
// FILE SHARING (UGC)
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_FileShare(file: *const c_char) -> u64 {
    if file.is_null() {
        return 0;
    }

    println!("[Oracle] File share requested");
    1 // API call handle
}

// ============================================================================
// UGC DOWNLOAD
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_UGCDownload(content: u64, priority: u32) -> u64 {
    println!("[Oracle] UGC download: {}", content);
    1 // API call handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_GetUGCDownloadProgress(
    content: u64,
    bytes_downloaded: *mut i32,
    bytes_expected: *mut i32,
) -> bool {
    if !bytes_downloaded.is_null() && !bytes_expected.is_null() {
        unsafe {
            *bytes_downloaded = 1024;
            *bytes_expected = 1024;
        }
        return true;
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_UGCRead(
    content: u64,
    data: *mut c_void,
    data_to_read: i32,
    offset: u32,
    action: i32,
) -> i32 {
    0 // No data
}

// ============================================================================
// WORKSHOP PUBLISHING
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_PublishWorkshopFile(
    file: *const c_char,
    preview_file: *const c_char,
    consumer_app_id: u32,
    title: *const c_char,
    description: *const c_char,
    visibility: i32,
    tags: *mut c_void,
    workshop_file_type: i32,
) -> u64 {
    println!("[Oracle] Publishing workshop file");
    1 // API call handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_CreatePublishedFileUpdateRequest(
    published_file_id: u64,
) -> u64 {
    println!("[Oracle] Creating workshop update request");
    1 // Update handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_UpdatePublishedFileFile(
    update_handle: u64,
    file: *const c_char,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_UpdatePublishedFilePreviewFile(
    update_handle: u64,
    preview_file: *const c_char,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_UpdatePublishedFileTitle(
    update_handle: u64,
    title: *const c_char,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_UpdatePublishedFileDescription(
    update_handle: u64,
    description: *const c_char,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_UpdatePublishedFileVisibility(
    update_handle: u64,
    visibility: i32,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_UpdatePublishedFileTags(
    update_handle: u64,
    tags: *mut c_void,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_CommitPublishedFileUpdate(
    update_handle: u64,
) -> u64 {
    println!("[Oracle] Committing workshop update");
    1 // API call handle
}

// ============================================================================
// SUBSCRIPTIONS
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_SubscribePublishedFile(
    published_file_id: u64,
) -> u64 {
    println!(
        "[Oracle] Subscribed to workshop item: {}",
        published_file_id
    );
    1
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_UnsubscribePublishedFile(
    published_file_id: u64,
) -> u64 {
    println!(
        "[Oracle] Unsubscribed from workshop item: {}",
        published_file_id
    );
    1
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_GetPublishedItemVoteDetails(
    published_file_id: u64,
) -> u64 {
    1
}

// ============================================================================
// SYNC PLATFORMS
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_SetSyncPlatforms(
    file: *const c_char,
    remote_storage_platform: i32,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamRemoteStorage_GetSyncPlatforms(file: *const c_char) -> i32 {
    3 // All platforms (Windows, Mac, Linux)
}
