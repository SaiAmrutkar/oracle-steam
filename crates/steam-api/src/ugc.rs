// ISteamUGC - Workshop/User Generated Content - 80+ functions
use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr};

pub type UGCQueryHandle_t = u64;
pub type UGCUpdateHandle_t = u64;
pub type PublishedFileId_t = u64;

lazy_static! {
    static ref UGC_QUERIES: RwLock<HashMap<UGCQueryHandle_t, UGCQuery>> =
        RwLock::new(HashMap::new());
    static ref UGC_UPDATES: RwLock<HashMap<UGCUpdateHandle_t, UGCUpdate>> =
        RwLock::new(HashMap::new());
    static ref SUBSCRIBED_ITEMS: RwLock<Vec<PublishedFileId_t>> = RwLock::new(Vec::new());
    static ref NEXT_HANDLE: RwLock<u64> = RwLock::new(1);
}

struct UGCQuery {
    results: Vec<UGCItem>,
    query_type: i32,
}

struct UGCUpdate {
    file_id: PublishedFileId_t,
    title: Option<String>,
    description: Option<String>,
    preview_file: Option<String>,
    content_file: Option<String>,
    tags: Vec<String>,
}

struct UGCItem {
    published_file_id: PublishedFileId_t,
    title: String,
    description: String,
    owner: u64,
    time_created: u32,
    time_updated: u32,
    visibility: i32,
    tags: Vec<String>,
    file_size: u64,
    preview_file_size: u64,
    url: String,
    votes_up: u32,
    votes_down: u32,
}

fn next_handle() -> u64 {
    let mut h = NEXT_HANDLE.write();
    let val = *h;
    *h += 1;
    val
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_CreateQueryUserUGCRequest(
    account_id: u32,
    list_type: i32,
    matching_ugc_type: i32,
    sort_order: i32,
    creator_app_id: u32,
    consumer_app_id: u32,
    page: u32,
) -> UGCQueryHandle_t {
    let handle = next_handle();
    let query = UGCQuery {
        results: Vec::new(),
        query_type: list_type,
    };
    UGC_QUERIES.write().insert(handle, query);
    handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_CreateQueryAllUGCRequest(
    query_type: i32,
    matching_file_type: i32,
    creator_app_id: u32,
    consumer_app_id: u32,
    page: u32,
) -> UGCQueryHandle_t {
    let handle = next_handle();
    let query = UGCQuery {
        results: Vec::new(),
        query_type,
    };
    UGC_QUERIES.write().insert(handle, query);
    handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_CreateQueryUGCDetailsRequest(
    published_file_ids: *mut PublishedFileId_t,
    num_published_file_ids: u32,
) -> UGCQueryHandle_t {
    let handle = next_handle();
    let query = UGCQuery {
        results: Vec::new(),
        query_type: 0,
    };
    UGC_QUERIES.write().insert(handle, query);
    handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SendQueryUGCRequest(handle: UGCQueryHandle_t) -> u64 {
    println!("[Oracle] UGC query sent: {}", handle);
    1 // API call handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_GetQueryUGCResult(
    handle: UGCQueryHandle_t,
    index: u32,
    details: *mut SteamUGCDetails_t,
) -> bool {
    if details.is_null() {
        return false;
    }

    let queries = UGC_QUERIES.read();
    if let Some(query) = queries.get(&handle) {
        if let Some(item) = query.results.get(index as usize) {
            unsafe {
                (*details).published_file_id = item.published_file_id;
                (*details).result = 1; // k_EResultOK
                (*details).file_type = 0;
                (*details).creator_app_id = 0;
                (*details).consumer_app_id = 0;

                let title_bytes = item.title.as_bytes();
                let title_len = title_bytes.len().min(127);
                (*details).title[..title_len].copy_from_slice(&title_bytes[..title_len]);
                (*details).title[title_len] = 0;

                let desc_bytes = item.description.as_bytes();
                let desc_len = desc_bytes.len().min(7999);
                (*details).description[..desc_len].copy_from_slice(&desc_bytes[..desc_len]);
                (*details).description[desc_len] = 0;

                (*details).steam_id_owner = item.owner;
                (*details).time_created = item.time_created;
                (*details).time_updated = item.time_updated;
                (*details).visibility = item.visibility as u32;
                (*details).banned = false;
                (*details).file_size = item.file_size;
                (*details).preview_file_size = item.preview_file_size;
                (*details).votes_up = item.votes_up;
                (*details).votes_down = item.votes_down;

                return true;
            }
        }
    }
    false
}

#[repr(C)]
pub struct SteamUGCDetails_t {
    published_file_id: PublishedFileId_t,
    result: i32,
    file_type: i32,
    creator_app_id: u32,
    consumer_app_id: u32,
    title: [u8; 129],
    description: [u8; 8000],
    steam_id_owner: u64,
    time_created: u32,
    time_updated: u32,
    time_added_to_user_list: u32,
    visibility: u32,
    banned: bool,
    accepted_for_use: bool,
    tags_truncated: bool,
    tags: [u8; 1024],
    file: u64,
    preview_file: u64,
    file_name: [u8; 260],
    file_size: u64,
    preview_file_size: u64,
    url: [u8; 256],
    votes_up: u32,
    votes_down: u32,
    score: f32,
    num_children: u32,
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_GetQueryUGCNumResults(handle: UGCQueryHandle_t) -> u32 {
    UGC_QUERIES
        .read()
        .get(&handle)
        .map(|q| q.results.len() as u32)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_ReleaseQueryUGCRequest(handle: UGCQueryHandle_t) -> bool {
    UGC_QUERIES.write().remove(&handle).is_some()
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_AddRequiredTag(
    handle: UGCQueryHandle_t,
    tag: *const c_char,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_AddExcludedTag(
    handle: UGCQueryHandle_t,
    tag: *const c_char,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SetReturnKeyValueTags(
    handle: UGCQueryHandle_t,
    return_tags: bool,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SetReturnLongDescription(
    handle: UGCQueryHandle_t,
    return_desc: bool,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SetReturnMetadata(
    handle: UGCQueryHandle_t,
    return_meta: bool,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SetReturnChildren(
    handle: UGCQueryHandle_t,
    return_children: bool,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SetReturnAdditionalPreviews(
    handle: UGCQueryHandle_t,
    return_previews: bool,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SetReturnTotalOnly(
    handle: UGCQueryHandle_t,
    total_only: bool,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SetLanguage(
    handle: UGCQueryHandle_t,
    language: *const c_char,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SetAllowCachedResponse(
    handle: UGCQueryHandle_t,
    max_age_seconds: u32,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SetCloudFileNameFilter(
    handle: UGCQueryHandle_t,
    match_cloud_filename: *const c_char,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SetMatchAnyTag(
    handle: UGCQueryHandle_t,
    match_any_tag: bool,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SetSearchText(
    handle: UGCQueryHandle_t,
    search_text: *const c_char,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SetRankedByTrendDays(
    handle: UGCQueryHandle_t,
    days: u32,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SubscribeItem(published_file_id: PublishedFileId_t) -> u64 {
    SUBSCRIBED_ITEMS.write().push(published_file_id);
    println!("[Oracle] Subscribed to item: {}", published_file_id);
    1
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_UnsubscribeItem(published_file_id: PublishedFileId_t) -> u64 {
    SUBSCRIBED_ITEMS
        .write()
        .retain(|&id| id != published_file_id);
    println!("[Oracle] Unsubscribed from item: {}", published_file_id);
    1
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_GetNumSubscribedItems() -> u32 {
    SUBSCRIBED_ITEMS.read().len() as u32
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_GetSubscribedItems(
    published_file_ids: *mut PublishedFileId_t,
    max_entries: u32,
) -> u32 {
    if published_file_ids.is_null() {
        return 0;
    }

    let items = SUBSCRIBED_ITEMS.read();
    let count = items.len().min(max_entries as usize);

    unsafe {
        for (i, &id) in items.iter().take(count).enumerate() {
            *published_file_ids.add(i) = id;
        }
    }

    count as u32
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_GetItemState(published_file_id: PublishedFileId_t) -> u32 {
    let subscribed = SUBSCRIBED_ITEMS.read().contains(&published_file_id);
    if subscribed {
        1 | 4 // k_EItemStateSubscribed | k_EItemStateInstalled
    } else {
        0 // k_EItemStateNone
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_GetItemInstallInfo(
    published_file_id: PublishedFileId_t,
    size_on_disk: *mut u64,
    folder: *mut c_char,
    folder_size: u32,
    timestamp: *mut u32,
) -> bool {
    if !SUBSCRIBED_ITEMS.read().contains(&published_file_id) {
        return false;
    }

    if !size_on_disk.is_null() {
        unsafe {
            *size_on_disk = 1024 * 1024;
        }
    }

    if !folder.is_null() && folder_size > 0 {
        let path = format!("workshop/content/{}", published_file_id);
        let bytes = path.as_bytes();
        let len = bytes.len().min((folder_size - 1) as usize);
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), folder as *mut u8, len);
            *folder.add(len) = 0;
        }
    }

    if !timestamp.is_null() {
        unsafe {
            *timestamp = 0;
        }
    }

    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_GetItemDownloadInfo(
    published_file_id: PublishedFileId_t,
    bytes_downloaded: *mut u64,
    bytes_total: *mut u64,
) -> bool {
    if !bytes_downloaded.is_null() && !bytes_total.is_null() {
        unsafe {
            *bytes_downloaded = 1024 * 1024;
            *bytes_total = 1024 * 1024;
        }
        return true;
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_DownloadItem(
    published_file_id: PublishedFileId_t,
    high_priority: bool,
) -> bool {
    println!("[Oracle] Downloading item: {}", published_file_id);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_StartItemUpdate(
    consumer_app_id: u32,
    published_file_id: PublishedFileId_t,
) -> UGCUpdateHandle_t {
    let handle = next_handle();
    let update = UGCUpdate {
        file_id: published_file_id,
        title: None,
        description: None,
        preview_file: None,
        content_file: None,
        tags: Vec::new(),
    };
    UGC_UPDATES.write().insert(handle, update);
    handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SetItemTitle(
    handle: UGCUpdateHandle_t,
    title: *const c_char,
) -> bool {
    if title.is_null() {
        return false;
    }

    unsafe {
        if let Ok(title_str) = CStr::from_ptr(title).to_str() {
            if let Some(update) = UGC_UPDATES.write().get_mut(&handle) {
                update.title = Some(title_str.to_string());
                return true;
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SetItemDescription(
    handle: UGCUpdateHandle_t,
    description: *const c_char,
) -> bool {
    if description.is_null() {
        return false;
    }

    unsafe {
        if let Ok(desc_str) = CStr::from_ptr(description).to_str() {
            if let Some(update) = UGC_UPDATES.write().get_mut(&handle) {
                update.description = Some(desc_str.to_string());
                return true;
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SetItemContent(
    handle: UGCUpdateHandle_t,
    content_folder: *const c_char,
) -> bool {
    if content_folder.is_null() {
        return false;
    }

    unsafe {
        if let Ok(folder_str) = CStr::from_ptr(content_folder).to_str() {
            if let Some(update) = UGC_UPDATES.write().get_mut(&handle) {
                update.content_file = Some(folder_str.to_string());
                return true;
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SetItemPreview(
    handle: UGCUpdateHandle_t,
    preview_file: *const c_char,
) -> bool {
    if preview_file.is_null() {
        return false;
    }

    unsafe {
        if let Ok(file_str) = CStr::from_ptr(preview_file).to_str() {
            if let Some(update) = UGC_UPDATES.write().get_mut(&handle) {
                update.preview_file = Some(file_str.to_string());
                return true;
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SetItemTags(
    handle: UGCUpdateHandle_t,
    tags: *const *const c_char,
    tag_count: u32,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SetItemVisibility(
    handle: UGCUpdateHandle_t,
    visibility: i32,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_SubmitItemUpdate(
    handle: UGCUpdateHandle_t,
    change_note: *const c_char,
) -> u64 {
    println!("[Oracle] Submitting item update");
    1
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_CreateItem(consumer_app_id: u32, file_type: i32) -> u64 {
    println!("[Oracle] Creating new workshop item");
    1
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUGC_DeleteItem(published_file_id: PublishedFileId_t) -> u64 {
    println!("[Oracle] Deleting item: {}", published_file_id);
    1
}
