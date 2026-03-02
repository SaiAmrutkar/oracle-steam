// ISteamInput - Modern controller API (100+ functions)
use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::ffi::{c_char, CStr};

pub type InputHandle_t = u64;
pub type InputActionSetHandle_t = u64;
pub type InputDigitalActionHandle_t = u64;
pub type InputAnalogActionHandle_t = u64;

const MAX_STEAM_CONTROLLERS: usize = 16;

lazy_static! {
    static ref CONNECTED_CONTROLLERS: RwLock<Vec<InputHandle_t>> = RwLock::new(vec![1]);
    static ref ACTION_SETS: RwLock<HashMap<String, InputActionSetHandle_t>> =
        RwLock::new(HashMap::new());
    static ref DIGITAL_ACTIONS: RwLock<HashMap<String, InputDigitalActionHandle_t>> =
        RwLock::new(HashMap::new());
    static ref ANALOG_ACTIONS: RwLock<HashMap<String, InputAnalogActionHandle_t>> =
        RwLock::new(HashMap::new());
    static ref ACTIVE_ACTION_SET: RwLock<InputActionSetHandle_t> = RwLock::new(0);
}

#[repr(C)]
pub struct InputAnalogActionData_t {
    mode: i32,
    x: f32,
    y: f32,
    active: bool,
}

#[repr(C)]
pub struct InputDigitalActionData_t {
    state: bool,
    active: bool,
}

#[repr(C)]
pub struct InputMotionData_t {
    rot_quat_x: f32,
    rot_quat_y: f32,
    rot_quat_z: f32,
    rot_quat_w: f32,
    pos_accel_x: f32,
    pos_accel_y: f32,
    pos_accel_z: f32,
    rot_vel_x: f32,
    rot_vel_y: f32,
    rot_vel_z: f32,
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_Init(explicitly_call_run_frame: bool) -> bool {
    println!("[Oracle] Input system initialized");
    *CONNECTED_CONTROLLERS.write() = vec![1];
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_Shutdown() -> bool {
    println!("[Oracle] Input system shutdown");
    CONNECTED_CONTROLLERS.write().clear();
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_SetInputActionManifestFilePath(
    input_action_manifest_file_path: *const c_char,
) -> bool {
    println!("[Oracle] Input manifest loaded");
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_RunFrame(reserved_value: bool) {
    // Poll controller state
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_BWaitForData(wait_forever: bool, timeout: u32) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_BNewDataAvailable() -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetConnectedControllers(
    handles_out: *mut InputHandle_t,
) -> i32 {
    let controllers = CONNECTED_CONTROLLERS.read();
    if !handles_out.is_null() {
        unsafe {
            for (i, &handle) in controllers.iter().enumerate() {
                *handles_out.add(i) = handle;
            }
        }
    }
    controllers.len() as i32
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_EnableDeviceCallbacks() {}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetControllerForGamepadIndex(index: i32) -> InputHandle_t {
    CONNECTED_CONTROLLERS
        .read()
        .get(index as usize)
        .copied()
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetGamepadIndexForController(
    input_handle: InputHandle_t,
) -> i32 {
    CONNECTED_CONTROLLERS
        .read()
        .iter()
        .position(|&h| h == input_handle)
        .map(|i| i as i32)
        .unwrap_or(-1)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetStringForXboxOrigin(origin: i32) -> *const c_char {
    b"A Button\0".as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetGlyphForXboxOrigin(origin: i32) -> *const c_char {
    b"/button_a.png\0".as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetActionSetHandle(
    action_set_name: *const c_char,
) -> InputActionSetHandle_t {
    if action_set_name.is_null() {
        return 0;
    }

    unsafe {
        if let Ok(name) = CStr::from_ptr(action_set_name).to_str() {
            let mut sets = ACTION_SETS.write();
            *sets
                .entry(name.to_string())
                .or_insert_with(|| sets.len() as u64 + 1)
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_ActivateActionSet(
    input_handle: InputHandle_t,
    action_set_handle: InputActionSetHandle_t,
) {
    *ACTIVE_ACTION_SET.write() = action_set_handle;
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetCurrentActionSet(
    input_handle: InputHandle_t,
) -> InputActionSetHandle_t {
    *ACTIVE_ACTION_SET.read()
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_ActivateActionSetLayer(
    input_handle: InputHandle_t,
    action_set_layer_handle: InputActionSetHandle_t,
) {
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_DeactivateActionSetLayer(
    input_handle: InputHandle_t,
    action_set_layer_handle: InputActionSetHandle_t,
) {
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_DeactivateAllActionSetLayers(input_handle: InputHandle_t) {}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetActiveActionSetLayers(
    input_handle: InputHandle_t,
    handles_out: *mut InputActionSetHandle_t,
) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetDigitalActionHandle(
    action_name: *const c_char,
) -> InputDigitalActionHandle_t {
    if action_name.is_null() {
        return 0;
    }

    unsafe {
        if let Ok(name) = CStr::from_ptr(action_name).to_str() {
            let mut actions = DIGITAL_ACTIONS.write();
            *actions
                .entry(name.to_string())
                .or_insert_with(|| actions.len() as u64 + 1)
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetDigitalActionData(
    input_handle: InputHandle_t,
    digital_action_handle: InputDigitalActionHandle_t,
) -> InputDigitalActionData_t {
    InputDigitalActionData_t {
        state: false,
        active: true,
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetDigitalActionOrigins(
    input_handle: InputHandle_t,
    action_set_handle: InputActionSetHandle_t,
    digital_action_handle: InputDigitalActionHandle_t,
    origins_out: *mut i32,
) -> i32 {
    if !origins_out.is_null() {
        unsafe {
            *origins_out = 0;
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetStringForDigitalActionName(
    action_handle: InputDigitalActionHandle_t,
) -> *const c_char {
    b"Jump\0".as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetAnalogActionHandle(
    action_name: *const c_char,
) -> InputAnalogActionHandle_t {
    if action_name.is_null() {
        return 0;
    }

    unsafe {
        if let Ok(name) = CStr::from_ptr(action_name).to_str() {
            let mut actions = ANALOG_ACTIONS.write();
            *actions
                .entry(name.to_string())
                .or_insert_with(|| actions.len() as u64 + 1)
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetAnalogActionData(
    input_handle: InputHandle_t,
    analog_action_handle: InputAnalogActionHandle_t,
) -> InputAnalogActionData_t {
    InputAnalogActionData_t {
        mode: 0,
        x: 0.0,
        y: 0.0,
        active: true,
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetAnalogActionOrigins(
    input_handle: InputHandle_t,
    action_set_handle: InputActionSetHandle_t,
    analog_action_handle: InputAnalogActionHandle_t,
    origins_out: *mut i32,
) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetGlyphPNGForActionOrigin(
    origin: i32,
    size: i32,
    flags: u32,
) -> *const c_char {
    b"/glyph.png\0".as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetGlyphSVGForActionOrigin(
    origin: i32,
    flags: u32,
) -> *const c_char {
    b"<svg></svg>\0".as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetGlyphForActionOrigin_Legacy(
    origin: i32,
) -> *const c_char {
    b"/button.png\0".as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetStringForActionOrigin(origin: i32) -> *const c_char {
    b"Button\0".as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetStringForAnalogActionName(
    action_handle: InputAnalogActionHandle_t,
) -> *const c_char {
    b"Move\0".as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_StopAnalogActionMomentum(
    input_handle: InputHandle_t,
    action: InputAnalogActionHandle_t,
) {
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetMotionData(
    input_handle: InputHandle_t,
) -> InputMotionData_t {
    unsafe { std::mem::zeroed() }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_TriggerVibration(
    input_handle: InputHandle_t,
    left_speed: u16,
    right_speed: u16,
) {
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_TriggerVibrationExtended(
    input_handle: InputHandle_t,
    left_speed: u16,
    right_speed: u16,
    left_trigger_speed: u16,
    right_trigger_speed: u16,
) {
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_TriggerSimpleHapticEvent(
    input_handle: InputHandle_t,
    haptic_location: i32,
    intensity: u8,
    gain_db: i8,
    other_intensity: u8,
    other_gain_db: i8,
) {
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_SetLEDColor(
    input_handle: InputHandle_t,
    color_r: u8,
    color_g: u8,
    color_b: u8,
    flags: u32,
) {
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_Legacy_TriggerHapticPulse(
    input_handle: InputHandle_t,
    target_pad: i32,
    duration: u16,
) {
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_Legacy_TriggerRepeatedHapticPulse(
    input_handle: InputHandle_t,
    target_pad: i32,
    duration: u16,
    off_duration: u16,
    repeat: u16,
    flags: u32,
) {
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_ShowBindingPanel(input_handle: InputHandle_t) -> bool {
    println!("[Oracle] Showing binding panel");
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetInputTypeForHandle(input_handle: InputHandle_t) -> i32 {
    0 // k_ESteamInputType_Unknown
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetRemotePlaySessionID(input_handle: InputHandle_t) -> u32 {
    0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_GetSessionInputConfigurationSettings() -> u16 {
    0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInput_SetDualSenseTriggerEffect(
    input_handle: InputHandle_t,
    parameters: *const c_void,
) {
}
