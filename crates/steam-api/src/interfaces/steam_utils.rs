use crate::callbacks::*;
use oracle_core::utils::UtilsManager;
use parking_lot::RwLock;
use std::ffi::{c_void, CStr};
use std::sync::Arc;

pub const STEAMUTILS_INTERFACE_VERSION: &str = "SteamUtils010";

#[repr(C)]
pub struct ISteamUtils {
    vtable: *const ISteamUtilsVTable,
    manager: Arc<RwLock<UtilsManager>>,
}

#[repr(C)]
pub struct ISteamUtilsVTable {
    pub get_seconds_since_app_active: unsafe extern "C" fn(*mut ISteamUtils) -> u32,
    pub get_seconds_since_computer_active: unsafe extern "C" fn(*mut ISteamUtils) -> u32,
    pub get_connected_universe: unsafe extern "C" fn(*mut ISteamUtils) -> i32,
    pub get_server_real_time: unsafe extern "C" fn(*mut ISteamUtils) -> u32,
    pub get_ip_country: unsafe extern "C" fn(*mut ISteamUtils) -> *const i8,
    pub get_image_size: unsafe extern "C" fn(*mut ISteamUtils, i32, *mut u32, *mut u32) -> bool,
    pub get_image_rgba: unsafe extern "C" fn(*mut ISteamUtils, i32, *mut u8, i32) -> bool,
    pub get_current_battery_power: unsafe extern "C" fn(*mut ISteamUtils) -> u8,
    pub get_app_id: unsafe extern "C" fn(*mut ISteamUtils) -> u32,
    pub set_overlay_notification_position: unsafe extern "C" fn(*mut ISteamUtils, i32),
    pub is_api_call_completed: unsafe extern "C" fn(*mut ISteamUtils, u64, *mut bool) -> bool,
    pub get_api_call_failure_reason: unsafe extern "C" fn(*mut ISteamUtils, u64) -> i32,
    pub get_api_call_result:
        unsafe extern "C" fn(*mut ISteamUtils, u64, *mut c_void, i32, i32, *mut bool) -> bool,
    pub run_frame: unsafe extern "C" fn(*mut ISteamUtils),
    pub get_ipc_call_count: unsafe extern "C" fn(*mut ISteamUtils) -> u32,
    pub set_warning_message_hook: unsafe extern "C" fn(*mut ISteamUtils, *mut c_void),
    pub is_overlay_enabled: unsafe extern "C" fn(*mut ISteamUtils) -> bool,
    pub boverlay_needs_present: unsafe extern "C" fn(*mut ISteamUtils) -> bool,
    pub check_file_signature: unsafe extern "C" fn(*mut ISteamUtils, *const i8) -> u64,
    pub show_gamepad_text_input:
        unsafe extern "C" fn(*mut ISteamUtils, i32, i32, *const i8, u32, *const i8) -> bool,
    pub get_entered_gamepad_text_length: unsafe extern "C" fn(*mut ISteamUtils) -> u32,
    pub get_entered_gamepad_text_input:
        unsafe extern "C" fn(*mut ISteamUtils, *mut i8, u32) -> bool,
    pub get_steamos_language: unsafe extern "C" fn(*mut ISteamUtils) -> *const i8,
    pub is_steam_running_in_vr: unsafe extern "C" fn(*mut ISteamUtils) -> bool,
    pub set_overlay_notification_inset: unsafe extern "C" fn(*mut ISteamUtils, i32, i32),
    pub is_steam_in_big_picture_mode: unsafe extern "C" fn(*mut ISteamUtils) -> bool,
    pub start_vr_dashboard: unsafe extern "C" fn(*mut ISteamUtils),
    pub is_vr_headset_streaming_enabled: unsafe extern "C" fn(*mut ISteamUtils) -> bool,
    pub set_vr_headset_streaming_enabled: unsafe extern "C" fn(*mut ISteamUtils, bool),
    pub is_steam_china_launcher: unsafe extern "C" fn(*mut ISteamUtils) -> bool,
    pub init_filter_text: unsafe extern "C" fn(*mut ISteamUtils, u32) -> bool,
    pub filter_text:
        unsafe extern "C" fn(*mut ISteamUtils, i32, u64, *const i8, *mut i8, u32) -> i32,
    pub get_ipv6_connectivity_state: unsafe extern "C" fn(*mut ISteamUtils, i32) -> i32,
    pub is_steam_running_on_steam_deck: unsafe extern "C" fn(*mut ISteamUtils) -> bool,
    pub show_floating_gamepad_text_input:
        unsafe extern "C" fn(*mut ISteamUtils, i32, i32, i32, i32, i32) -> bool,
    pub set_gamepad_text_input_mode: unsafe extern "C" fn(*mut ISteamUtils, i32),
    pub set_floating_gamepad_text_input_mode: unsafe extern "C" fn(*mut ISteamUtils, i32),
    pub dismiss_floating_gamepad_text_input: unsafe extern "C" fn(*mut ISteamUtils) -> bool,
    pub dismiss_gamepad_text_input: unsafe extern "C" fn(*mut ISteamUtils) -> bool,
}

impl ISteamUtils {
    pub fn new(utils_manager: Arc<RwLock<UtilsManager>>) -> Box<Self> {
        Box::new(ISteamUtils {
            vtable: &STEAM_UTILS_VTABLE,
            manager: utils_manager,
        })
    }
}

unsafe extern "C" fn get_seconds_since_app_active(this: *mut ISteamUtils) -> u32 {
    let utils = &*this;
    utils.manager.read().get_seconds_since_app_active()
}

unsafe extern "C" fn get_seconds_since_computer_active(this: *mut ISteamUtils) -> u32 {
    let utils = &*this;
    utils.manager.read().get_seconds_since_computer_active()
}

unsafe extern "C" fn get_connected_universe(_this: *mut ISteamUtils) -> i32 {
    1 // k_EUniversePublic
}

unsafe extern "C" fn get_server_real_time(_this: *mut ISteamUtils) -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32
}

unsafe extern "C" fn get_ip_country(this: *mut ISteamUtils) -> *const i8 {
    let utils = &*this;
    utils.manager.read().get_ip_country_ptr()
}

unsafe extern "C" fn get_image_size(
    this: *mut ISteamUtils,
    image: i32,
    width: *mut u32,
    height: *mut u32,
) -> bool {
    let utils = &*this;
    let manager = utils.manager.read();

    match manager.get_image_size(image) {
        Some((w, h)) => {
            if !width.is_null() {
                *width = w;
            }
            if !height.is_null() {
                *height = h;
            }
            true
        }
        None => false,
    }
}

unsafe extern "C" fn get_image_rgba(
    this: *mut ISteamUtils,
    image: i32,
    dest: *mut u8,
    dest_buffer_size: i32,
) -> bool {
    let utils = &*this;
    let manager = utils.manager.read();

    if dest.is_null() || dest_buffer_size <= 0 {
        return false;
    }

    match manager.get_image_data(image) {
        Some(data) => {
            if data.len() > dest_buffer_size as usize {
                return false;
            }

            std::ptr::copy_nonoverlapping(data.as_ptr(), dest, data.len());
            true
        }
        None => false,
    }
}

unsafe extern "C" fn get_current_battery_power(this: *mut ISteamUtils) -> u8 {
    let utils = &*this;
    utils.manager.read().get_battery_power()
}

unsafe extern "C" fn get_app_id(_this: *mut ISteamUtils) -> u32 {
    oracle_core::config::get_app_id()
}

unsafe extern "C" fn set_overlay_notification_position(this: *mut ISteamUtils, position: i32) {
    let utils = &*this;
    utils
        .manager
        .write()
        .set_overlay_notification_position(position);
}

unsafe extern "C" fn is_api_call_completed(
    _this: *mut ISteamUtils,
    handle: u64,
    failed: *mut bool,
) -> bool {
    oracle_core::callbacks::is_call_completed(handle, failed)
}

unsafe extern "C" fn get_api_call_failure_reason(_this: *mut ISteamUtils, handle: u64) -> i32 {
    oracle_core::callbacks::get_call_failure_reason(handle)
}

unsafe extern "C" fn get_api_call_result(
    _this: *mut ISteamUtils,
    handle: u64,
    callback: *mut c_void,
    callback_size: i32,
    expected_callback: i32,
    failed: *mut bool,
) -> bool {
    oracle_core::callbacks::get_call_result(
        handle,
        callback,
        callback_size,
        expected_callback,
        failed,
    )
}

unsafe extern "C" fn run_frame(_this: *mut ISteamUtils) {
    oracle_core::callbacks::run_callbacks();
}

unsafe extern "C" fn get_ipc_call_count(_this: *mut ISteamUtils) -> u32 {
    0
}

unsafe extern "C" fn set_warning_message_hook(this: *mut ISteamUtils, function: *mut c_void) {
    let utils = &*this;
    utils
        .manager
        .write()
        .set_warning_message_hook(function as usize);
}

unsafe extern "C" fn is_overlay_enabled(_this: *mut ISteamUtils) -> bool {
    oracle_overlay::is_enabled()
}

unsafe extern "C" fn boverlay_needs_present(_this: *mut ISteamUtils) -> bool {
    oracle_overlay::needs_present()
}

unsafe extern "C" fn check_file_signature(this: *mut ISteamUtils, filename: *const i8) -> u64 {
    let file_path = if filename.is_null() {
        return 0;
    } else {
        CStr::from_ptr(filename).to_string_lossy().into_owned()
    };

    let call_handle = oracle_core::callbacks::generate_api_call_handle();

    std::thread::spawn(move || {
        let signature_valid = oracle_core::protection::check_file_signature(&file_path);

        oracle_core::callbacks::complete_api_call(
            call_handle,
            CheckFileSignature_t {
                check_file_signature: if signature_valid { 0 } else { 2 },
            },
        );
    });

    call_handle
}

unsafe extern "C" fn show_gamepad_text_input(
    this: *mut ISteamUtils,
    input_mode: i32,
    line_input_mode: i32,
    description: *const i8,
    max_chars: u32,
    existing_text: *const i8,
) -> bool {
    let utils = &*this;
    let mut manager = utils.manager.write();

    let desc = if description.is_null() {
        String::new()
    } else {
        CStr::from_ptr(description).to_string_lossy().into_owned()
    };

    let existing = if existing_text.is_null() {
        String::new()
    } else {
        CStr::from_ptr(existing_text).to_string_lossy().into_owned()
    };

    manager.show_gamepad_text_input(input_mode, line_input_mode, desc, max_chars, existing)
}

unsafe extern "C" fn get_entered_gamepad_text_length(this: *mut ISteamUtils) -> u32 {
    let utils = &*this;
    utils.manager.read().get_gamepad_text_length()
}

unsafe extern "C" fn get_entered_gamepad_text_input(
    this: *mut ISteamUtils,
    text: *mut i8,
    text_size: u32,
) -> bool {
    let utils = &*this;
    let manager = utils.manager.read();

    if text.is_null() || text_size == 0 {
        return false;
    }

    match manager.get_gamepad_text() {
        Some(input_text) => {
            let bytes = input_text.as_bytes();
            let copy_len = std::cmp::min(bytes.len(), (text_size - 1) as usize);

            std::ptr::copy_nonoverlapping(bytes.as_ptr(), text as *mut u8, copy_len);
            *(text as *mut u8).add(copy_len) = 0;

            true
        }
        None => false,
    }
}

unsafe extern "C" fn get_steamos_language(_this: *mut ISteamUtils) -> *const i8 {
    b"english\0".as_ptr() as *const i8
}

unsafe extern "C" fn is_steam_running_in_vr(this: *mut ISteamUtils) -> bool {
    let utils = &*this;
    utils.manager.read().is_vr_mode()
}

unsafe extern "C" fn set_overlay_notification_inset(
    this: *mut ISteamUtils,
    horizontal_inset: i32,
    vertical_inset: i32,
) {
    let utils = &*this;
    utils
        .manager
        .write()
        .set_overlay_notification_inset(horizontal_inset, vertical_inset);
}

unsafe extern "C" fn is_steam_in_big_picture_mode(this: *mut ISteamUtils) -> bool {
    let utils = &*this;
    utils.manager.read().is_big_picture_mode()
}

unsafe extern "C" fn start_vr_dashboard(_this: *mut ISteamUtils) {
    oracle_overlay::start_vr_dashboard();
}

unsafe extern "C" fn is_vr_headset_streaming_enabled(this: *mut ISteamUtils) -> bool {
    let utils = &*this;
    utils.manager.read().is_vr_streaming_enabled()
}

unsafe extern "C" fn set_vr_headset_streaming_enabled(this: *mut ISteamUtils, enabled: bool) {
    let utils = &*this;
    utils.manager.write().set_vr_streaming_enabled(enabled);
}

unsafe extern "C" fn is_steam_china_launcher(_this: *mut ISteamUtils) -> bool {
    false
}

unsafe extern "C" fn init_filter_text(this: *mut ISteamUtils, options: u32) -> bool {
    let utils = &*this;
    utils.manager.write().init_text_filter(options)
}

unsafe extern "C" fn filter_text(
    this: *mut ISteamUtils,
    context: i32,
    source_steamid: u64,
    input_message: *const i8,
    output_message: *mut i8,
    output_size: u32,
) -> i32 {
    let utils = &*this;
    let manager = utils.manager.read();

    if input_message.is_null() || output_message.is_null() || output_size == 0 {
        return 0;
    }

    let input = CStr::from_ptr(input_message).to_string_lossy().into_owned();

    match manager.filter_text(context, source_steamid, &input) {
        Some(filtered) => {
            let bytes = filtered.as_bytes();
            let copy_len = std::cmp::min(bytes.len(), (output_size - 1) as usize);

            std::ptr::copy_nonoverlapping(bytes.as_ptr(), output_message as *mut u8, copy_len);
            *(output_message as *mut u8).add(copy_len) = 0;

            copy_len as i32
        }
        None => 0,
    }
}

unsafe extern "C" fn get_ipv6_connectivity_state(this: *mut ISteamUtils, protocol: i32) -> i32 {
    let utils = &*this;
    utils.manager.read().get_ipv6_connectivity_state(protocol)
}

unsafe extern "C" fn is_steam_running_on_steam_deck(_this: *mut ISteamUtils) -> bool {
    oracle_core::platform::is_steam_deck()
}

unsafe extern "C" fn show_floating_gamepad_text_input(
    this: *mut ISteamUtils,
    keyboard_mode: i32,
    text_field_x_position: i32,
    text_field_y_position: i32,
    text_field_width: i32,
    text_field_height: i32,
) -> bool {
    let utils = &*this;
    utils.manager.write().show_floating_gamepad_input(
        keyboard_mode,
        text_field_x_position,
        text_field_y_position,
        text_field_width,
        text_field_height,
    )
}

unsafe extern "C" fn set_gamepad_text_input_mode(this: *mut ISteamUtils, mode: i32) {
    let utils = &*this;
    utils.manager.write().set_gamepad_text_input_mode(mode);
}

unsafe extern "C" fn set_floating_gamepad_text_input_mode(this: *mut ISteamUtils, mode: i32) {
    let utils = &*this;
    utils
        .manager
        .write()
        .set_floating_gamepad_text_input_mode(mode);
}

unsafe extern "C" fn dismiss_floating_gamepad_text_input(this: *mut ISteamUtils) -> bool {
    let utils = &*this;
    utils.manager.write().dismiss_floating_gamepad_text_input()
}

unsafe extern "C" fn dismiss_gamepad_text_input(this: *mut ISteamUtils) -> bool {
    let utils = &*this;
    utils.manager.write().dismiss_gamepad_text_input()
}

static STEAM_UTILS_VTABLE: ISteamUtilsVTable = ISteamUtilsVTable {
    get_seconds_since_app_active,
    get_seconds_since_computer_active,
    get_connected_universe,
    get_server_real_time,
    get_ip_country,
    get_image_size,
    get_image_rgba,
    get_current_battery_power,
    get_app_id,
    set_overlay_notification_position,
    is_api_call_completed,
    get_api_call_failure_reason,
    get_api_call_result,
    run_frame,
    get_ipc_call_count,
    set_warning_message_hook,
    is_overlay_enabled,
    boverlay_needs_present,
    check_file_signature,
    show_gamepad_text_input,
    get_entered_gamepad_text_length,
    get_entered_gamepad_text_input,
    get_steamos_language,
    is_steam_running_in_vr,
    set_overlay_notification_inset,
    is_steam_in_big_picture_mode,
    start_vr_dashboard,
    is_vr_headset_streaming_enabled,
    set_vr_headset_streaming_enabled,
    is_steam_china_launcher,
    init_filter_text,
    filter_text,
    get_ipv6_connectivity_state,
    is_steam_running_on_steam_deck,
    show_floating_gamepad_text_input,
    set_gamepad_text_input_mode,
    set_floating_gamepad_text_input_mode,
    dismiss_floating_gamepad_text_input,
    dismiss_gamepad_text_input,
};
