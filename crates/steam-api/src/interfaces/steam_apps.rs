use crate::callbacks::*;
use oracle_core::apps::AppsManager;
use parking_lot::RwLock;
use std::ffi::{c_void, CStr};
use std::sync::Arc;

pub const STEAMAPPS_INTERFACE_VERSION: &str = "STEAMAPPS_INTERFACE_VERSION008";

#[repr(C)]
pub struct ISteamApps {
    vtable: *const ISteamAppsVTable,
    manager: Arc<RwLock<AppsManager>>,
}

#[repr(C)]
pub struct ISteamAppsVTable {
    pub is_subscribed: unsafe extern "C" fn(*mut ISteamApps) -> bool,
    pub is_low_violence: unsafe extern "C" fn(*mut ISteamApps) -> bool,
    pub is_cybercafe: unsafe extern "C" fn(*mut ISteamApps) -> bool,
    pub is_vac_banned: unsafe extern "C" fn(*mut ISteamApps) -> bool,
    pub get_current_game_language: unsafe extern "C" fn(*mut ISteamApps) -> *const i8,
    pub get_available_game_languages: unsafe extern "C" fn(*mut ISteamApps) -> *const i8,
    pub is_subscribed_app: unsafe extern "C" fn(*mut ISteamApps, u32) -> bool,
    pub is_dlc_installed: unsafe extern "C" fn(*mut ISteamApps, u32) -> bool,
    pub get_earliest_purchase_unix_time: unsafe extern "C" fn(*mut ISteamApps, u32) -> u32,
    pub is_subscribed_from_free_weekend: unsafe extern "C" fn(*mut ISteamApps) -> bool,
    pub get_dlc_count: unsafe extern "C" fn(*mut ISteamApps) -> i32,
    pub get_dlc_data_by_index:
        unsafe extern "C" fn(*mut ISteamApps, i32, *mut u32, *mut bool, *mut i8, i32) -> bool,
    pub install_dlc: unsafe extern "C" fn(*mut ISteamApps, u32),
    pub uninstall_dlc: unsafe extern "C" fn(*mut ISteamApps, u32),
    pub request_app_proof_of_purchase_key: unsafe extern "C" fn(*mut ISteamApps, u32),
    pub get_current_beta_name: unsafe extern "C" fn(*mut ISteamApps, *mut i8, i32) -> bool,
    pub mark_content_corrupt: unsafe extern "C" fn(*mut ISteamApps, bool) -> bool,
    pub get_installed_depots: unsafe extern "C" fn(*mut ISteamApps, u32, *mut u32, u32) -> u32,
    pub get_app_install_dir: unsafe extern "C" fn(*mut ISteamApps, u32, *mut i8, u32) -> u32,
    pub is_app_installed: unsafe extern "C" fn(*mut ISteamApps, u32) -> bool,
    pub get_app_owner: unsafe extern "C" fn(*mut ISteamApps) -> u64,
    pub get_launch_query_param: unsafe extern "C" fn(*mut ISteamApps, *const i8) -> *const i8,
    pub get_dlc_download_progress:
        unsafe extern "C" fn(*mut ISteamApps, u32, *mut u64, *mut u64) -> bool,
    pub get_app_build_id: unsafe extern "C" fn(*mut ISteamApps) -> i32,
    pub request_all_proof_of_purchase_keys: unsafe extern "C" fn(*mut ISteamApps),
    pub get_file_details: unsafe extern "C" fn(*mut ISteamApps, *const i8) -> u64,
    pub get_launch_command_line: unsafe extern "C" fn(*mut ISteamApps, *mut i8, i32) -> i32,
    pub is_subscribed_from_family_sharing: unsafe extern "C" fn(*mut ISteamApps) -> bool,
    pub is_timed_trial: unsafe extern "C" fn(*mut ISteamApps, *mut u32, *mut u32) -> bool,
    pub set_dlc_context: unsafe extern "C" fn(*mut ISteamApps, u32) -> bool,
    pub get_num_betas: unsafe extern "C" fn(*mut ISteamApps, *mut i32, *mut i32) -> i32,
    pub get_beta_info: unsafe extern "C" fn(
        *mut ISteamApps,
        i32,
        *mut u32,
        *mut u32,
        *mut i8,
        i32,
        *mut i8,
        i32,
        *mut bool,
    ) -> bool,
    pub set_active_beta: unsafe extern "C" fn(*mut ISteamApps, *const i8) -> bool,
}

impl ISteamApps {
    pub fn new(apps_manager: Arc<RwLock<AppsManager>>) -> Box<Self> {
        Box::new(ISteamApps {
            vtable: &STEAM_APPS_VTABLE,
            manager: apps_manager,
        })
    }
}

unsafe extern "C" fn is_subscribed(this: *mut ISteamApps) -> bool {
    let apps = &*this;
    apps.manager.read().is_subscribed()
}

unsafe extern "C" fn is_low_violence(this: *mut ISteamApps) -> bool {
    let apps = &*this;
    apps.manager.read().is_low_violence()
}

unsafe extern "C" fn is_cybercafe(_this: *mut ISteamApps) -> bool {
    false
}

unsafe extern "C" fn is_vac_banned(_this: *mut ISteamApps) -> bool {
    false
}

unsafe extern "C" fn get_current_game_language(this: *mut ISteamApps) -> *const i8 {
    let apps = &*this;
    apps.manager.read().get_current_language_ptr()
}

unsafe extern "C" fn get_available_game_languages(this: *mut ISteamApps) -> *const i8 {
    let apps = &*this;
    apps.manager.read().get_available_languages_ptr()
}

unsafe extern "C" fn is_subscribed_app(this: *mut ISteamApps, app_id: u32) -> bool {
    let apps = &*this;
    apps.manager.read().is_subscribed_app(app_id)
}

unsafe extern "C" fn is_dlc_installed(this: *mut ISteamApps, app_id: u32) -> bool {
    let apps = &*this;
    apps.manager.read().is_dlc_installed(app_id)
}

unsafe extern "C" fn get_earliest_purchase_unix_time(this: *mut ISteamApps, app_id: u32) -> u32 {
    let apps = &*this;
    apps.manager.read().get_earliest_purchase_time(app_id)
}

unsafe extern "C" fn is_subscribed_from_free_weekend(_this: *mut ISteamApps) -> bool {
    false
}

unsafe extern "C" fn get_dlc_count(this: *mut ISteamApps) -> i32 {
    let apps = &*this;
    apps.manager.read().get_dlc_count()
}

unsafe extern "C" fn get_dlc_data_by_index(
    this: *mut ISteamApps,
    dlc_index: i32,
    app_id: *mut u32,
    available: *mut bool,
    name: *mut i8,
    name_buffer_size: i32,
) -> bool {
    let apps = &*this;
    let manager = apps.manager.read();

    match manager.get_dlc_by_index(dlc_index) {
        Some(dlc) => {
            if !app_id.is_null() {
                *app_id = dlc.app_id;
            }
            if !available.is_null() {
                *available = dlc.available;
            }
            if !name.is_null() && name_buffer_size > 0 {
                let name_bytes = dlc.name.as_bytes();
                let copy_len = std::cmp::min(name_bytes.len(), (name_buffer_size - 1) as usize);
                std::ptr::copy_nonoverlapping(name_bytes.as_ptr(), name as *mut u8, copy_len);
                *(name as *mut u8).add(copy_len) = 0;
            }
            true
        }
        None => false,
    }
}

unsafe extern "C" fn install_dlc(this: *mut ISteamApps, app_id: u32) {
    let apps = &*this;
    let mut manager = apps.manager.write();

    if let Err(e) = manager.install_dlc(app_id) {
        log::error!("Failed to install DLC {}: {}", app_id, e);
    }
}

unsafe extern "C" fn uninstall_dlc(this: *mut ISteamApps, app_id: u32) {
    let apps = &*this;
    let mut manager = apps.manager.write();

    if let Err(e) = manager.uninstall_dlc(app_id) {
        log::error!("Failed to uninstall DLC {}: {}", app_id, e);
    }
}

unsafe extern "C" fn request_app_proof_of_purchase_key(this: *mut ISteamApps, app_id: u32) {
    let apps = &*this;
    let manager = apps.manager.read();

    std::thread::spawn(move || {
        // Generate proof of purchase key
        let key = oracle_core::licensing::generate_pop_key(app_id);

        oracle_core::callbacks::queue_callback(AppProofOfPurchaseKeyResponse_t {
            result: 1, // k_EResultOK
            app_id,
            key,
        });
    });
}

unsafe extern "C" fn get_current_beta_name(
    this: *mut ISteamApps,
    name: *mut i8,
    name_buffer_size: i32,
) -> bool {
    let apps = &*this;
    let manager = apps.manager.read();

    if name.is_null() || name_buffer_size <= 0 {
        return false;
    }

    match manager.get_current_beta_name() {
        Some(beta_name) => {
            let name_bytes = beta_name.as_bytes();
            let copy_len = std::cmp::min(name_bytes.len(), (name_buffer_size - 1) as usize);
            std::ptr::copy_nonoverlapping(name_bytes.as_ptr(), name as *mut u8, copy_len);
            *(name as *mut u8).add(copy_len) = 0;
            true
        }
        None => false,
    }
}

unsafe extern "C" fn mark_content_corrupt(this: *mut ISteamApps, missing_files_only: bool) -> bool {
    let apps = &*this;
    let mut manager = apps.manager.write();

    manager.mark_content_corrupt(missing_files_only)
}

unsafe extern "C" fn get_installed_depots(
    this: *mut ISteamApps,
    app_id: u32,
    depot_ids: *mut u32,
    max_depots: u32,
) -> u32 {
    let apps = &*this;
    let manager = apps.manager.read();

    if depot_ids.is_null() || max_depots == 0 {
        return 0;
    }

    let depots = manager.get_installed_depots(app_id);
    let copy_count = std::cmp::min(depots.len(), max_depots as usize);

    std::ptr::copy_nonoverlapping(depots.as_ptr(), depot_ids, copy_count);

    copy_count as u32
}

unsafe extern "C" fn get_app_install_dir(
    this: *mut ISteamApps,
    app_id: u32,
    folder: *mut i8,
    folder_buffer_size: u32,
) -> u32 {
    let apps = &*this;
    let manager = apps.manager.read();

    if folder.is_null() || folder_buffer_size == 0 {
        return 0;
    }

    match manager.get_app_install_dir(app_id) {
        Some(install_dir) => {
            let dir_bytes = install_dir.as_bytes();
            let copy_len = std::cmp::min(dir_bytes.len(), (folder_buffer_size - 1) as usize);
            std::ptr::copy_nonoverlapping(dir_bytes.as_ptr(), folder as *mut u8, copy_len);
            *(folder as *mut u8).add(copy_len) = 0;
            copy_len as u32
        }
        None => 0,
    }
}

unsafe extern "C" fn is_app_installed(this: *mut ISteamApps, app_id: u32) -> bool {
    let apps = &*this;
    apps.manager.read().is_app_installed(app_id)
}

unsafe extern "C" fn get_app_owner(this: *mut ISteamApps) -> u64 {
    let apps = &*this;
    apps.manager.read().get_app_owner()
}

unsafe extern "C" fn get_launch_query_param(this: *mut ISteamApps, key: *const i8) -> *const i8 {
    let apps = &*this;
    let manager = apps.manager.read();

    let key_str = if key.is_null() {
        return std::ptr::null();
    } else {
        CStr::from_ptr(key).to_string_lossy().into_owned()
    };

    manager.get_launch_query_param_ptr(&key_str)
}

unsafe extern "C" fn get_dlc_download_progress(
    this: *mut ISteamApps,
    app_id: u32,
    bytes_downloaded: *mut u64,
    bytes_total: *mut u64,
) -> bool {
    let apps = &*this;
    let manager = apps.manager.read();

    match manager.get_dlc_download_progress(app_id) {
        Some((downloaded, total)) => {
            if !bytes_downloaded.is_null() {
                *bytes_downloaded = downloaded;
            }
            if !bytes_total.is_null() {
                *bytes_total = total;
            }
            true
        }
        None => false,
    }
}

unsafe extern "C" fn get_app_build_id(this: *mut ISteamApps) -> i32 {
    let apps = &*this;
    apps.manager.read().get_app_build_id()
}

unsafe extern "C" fn request_all_proof_of_purchase_keys(_this: *mut ISteamApps) {
    std::thread::spawn(|| {
        oracle_core::callbacks::queue_callback(RequestAllProofOfPurchaseKeysResponse_t {
            result: 1, // k_EResultOK
        });
    });
}

unsafe extern "C" fn get_file_details(this: *mut ISteamApps, filename: *const i8) -> u64 {
    let file_path = if filename.is_null() {
        return 0;
    } else {
        CStr::from_ptr(filename).to_string_lossy().into_owned()
    };

    let call_handle = oracle_core::callbacks::generate_api_call_handle();

    std::thread::spawn(move || {
        match oracle_core::protection::get_file_details(&file_path) {
            Ok(details) => {
                oracle_core::callbacks::complete_api_call(
                    call_handle,
                    FileDetailsResult_t {
                        result: 1, // k_EResultOK
                        file_size: details.size,
                        file_hash: details.sha,
                        flags: details.flags,
                    },
                );
            }
            Err(_) => {
                oracle_core::callbacks::complete_api_call(
                    call_handle,
                    FileDetailsResult_t {
                        result: 2, // k_EResultFail
                        file_size: 0,
                        file_hash: [0; 20],
                        flags: 0,
                    },
                );
            }
        }
    });

    call_handle
}

unsafe extern "C" fn get_launch_command_line(
    this: *mut ISteamApps,
    command_line: *mut i8,
    buffer_size: i32,
) -> i32 {
    let apps = &*this;
    let manager = apps.manager.read();

    if command_line.is_null() || buffer_size <= 0 {
        return 0;
    }

    match manager.get_launch_command_line() {
        Some(cmd) => {
            let cmd_bytes = cmd.as_bytes();
            let copy_len = std::cmp::min(cmd_bytes.len(), (buffer_size - 1) as usize);
            std::ptr::copy_nonoverlapping(cmd_bytes.as_ptr(), command_line as *mut u8, copy_len);
            *(command_line as *mut u8).add(copy_len) = 0;
            copy_len as i32
        }
        None => 0,
    }
}

unsafe extern "C" fn is_subscribed_from_family_sharing(_this: *mut ISteamApps) -> bool {
    false
}

unsafe extern "C" fn is_timed_trial(
    this: *mut ISteamApps,
    seconds_allowed: *mut u32,
    seconds_played: *mut u32,
) -> bool {
    let apps = &*this;
    let manager = apps.manager.read();

    match manager.get_timed_trial_info() {
        Some((allowed, played)) => {
            if !seconds_allowed.is_null() {
                *seconds_allowed = allowed;
            }
            if !seconds_played.is_null() {
                *seconds_played = played;
            }
            true
        }
        None => false,
    }
}

unsafe extern "C" fn set_dlc_context(this: *mut ISteamApps, app_id: u32) -> bool {
    let apps = &*this;
    let mut manager = apps.manager.write();

    manager.set_dlc_context(app_id)
}

unsafe extern "C" fn get_num_betas(
    this: *mut ISteamApps,
    available: *mut i32,
    private: *mut i32,
) -> i32 {
    let apps = &*this;
    let manager = apps.manager.read();

    let (available_count, private_count) = manager.get_beta_counts();

    if !available.is_null() {
        *available = available_count;
    }
    if !private.is_null() {
        *private = private_count;
    }

    available_count + private_count
}

unsafe extern "C" fn get_beta_info(
    this: *mut ISteamApps,
    beta_index: i32,
    app_id: *mut u32,
    flags: *mut u32,
    build_id: *mut i8,
    build_id_size: i32,
    description: *mut i8,
    description_size: i32,
    password_required: *mut bool,
) -> bool {
    let apps = &*this;
    let manager = apps.manager.read();

    match manager.get_beta_info(beta_index) {
        Some(beta) => {
            if !app_id.is_null() {
                *app_id = beta.app_id;
            }
            if !flags.is_null() {
                *flags = beta.flags;
            }
            if !build_id.is_null() && build_id_size > 0 {
                let build_bytes = beta.build_id.as_bytes();
                let copy_len = std::cmp::min(build_bytes.len(), (build_id_size - 1) as usize);
                std::ptr::copy_nonoverlapping(build_bytes.as_ptr(), build_id as *mut u8, copy_len);
                *(build_id as *mut u8).add(copy_len) = 0;
            }
            if !description.is_null() && description_size > 0 {
                let desc_bytes = beta.description.as_bytes();
                let copy_len = std::cmp::min(desc_bytes.len(), (description_size - 1) as usize);
                std::ptr::copy_nonoverlapping(
                    desc_bytes.as_ptr(),
                    description as *mut u8,
                    copy_len,
                );
                *(description as *mut u8).add(copy_len) = 0;
            }
            if !password_required.is_null() {
                *password_required = beta.password_required;
            }
            true
        }
        None => false,
    }
}

unsafe extern "C" fn set_active_beta(this: *mut ISteamApps, beta_name: *const i8) -> bool {
    let apps = &*this;
    let mut manager = apps.manager.write();

    let beta_str = if beta_name.is_null() {
        String::new()
    } else {
        CStr::from_ptr(beta_name).to_string_lossy().into_owned()
    };

    manager.set_active_beta(beta_str)
}

static STEAM_APPS_VTABLE: ISteamAppsVTable = ISteamAppsVTable {
    is_subscribed,
    is_low_violence,
    is_cybercafe,
    is_vac_banned,
    get_current_game_language,
    get_available_game_languages,
    is_subscribed_app,
    is_dlc_installed,
    get_earliest_purchase_unix_time,
    is_subscribed_from_free_weekend,
    get_dlc_count,
    get_dlc_data_by_index,
    install_dlc,
    uninstall_dlc,
    request_app_proof_of_purchase_key,
    get_current_beta_name,
    mark_content_corrupt,
    get_installed_depots,
    get_app_install_dir,
    is_app_installed,
    get_app_owner,
    get_launch_query_param,
    get_dlc_download_progress,
    get_app_build_id,
    request_all_proof_of_purchase_keys,
    get_file_details,
    get_launch_command_line,
    is_subscribed_from_family_sharing,
    is_timed_trial,
    set_dlc_context,
    get_num_betas,
    get_beta_info,
    set_active_beta,
};
