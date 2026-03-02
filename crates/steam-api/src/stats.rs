use crate::{CURRENT_APP_ID, STEAM_CLIENT};
use std::ffi::{c_char, CStr};

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUserStats_RequestCurrentStats() -> bool {
    println!("[OracleSteam] Stats requested");
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUserStats_GetAchievement(
    name: *const c_char,
    achieved: *mut bool,
) -> bool {
    if name.is_null() || achieved.is_null() {
        return false;
    }

    unsafe {
        if let Ok(ach_name) = CStr::from_ptr(name).to_str() {
            if let Some(client) = STEAM_CLIENT.read().as_ref() {
                let app_id = *CURRENT_APP_ID.read();
                let unlocked = client.is_achievement_unlocked(app_id, ach_name);
                *achieved = unlocked;
                return true;
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUserStats_SetAchievement(name: *const c_char) -> bool {
    if name.is_null() {
        return false;
    }

    unsafe {
        if let Ok(ach_name) = CStr::from_ptr(name).to_str() {
            if let Some(client) = STEAM_CLIENT.read().as_ref() {
                let app_id = *CURRENT_APP_ID.read();
                if let Ok(unlocked) = client.unlock_achievement(app_id, ach_name) {
                    if unlocked {
                        println!("[OracleSteam] Achievement unlocked: {}", ach_name);
                    }
                    return true;
                }
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUserStats_ClearAchievement(_name: *const c_char) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUserStats_StoreStats() -> bool {
    println!("[OracleSteam] Stats stored");
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUserStats_GetStatInt(name: *const c_char, data: *mut i32) -> bool {
    if name.is_null() || data.is_null() {
        return false;
    }

    unsafe {
        if let Ok(stat_name) = CStr::from_ptr(name).to_str() {
            if let Some(client) = STEAM_CLIENT.read().as_ref() {
                let app_id = *CURRENT_APP_ID.read();
                if let Some(value) = client.get_stat_int(app_id, stat_name) {
                    *data = value;
                    return true;
                }
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUserStats_GetStatFloat(
    name: *const c_char,
    data: *mut f32,
) -> bool {
    if name.is_null() || data.is_null() {
        return false;
    }

    unsafe {
        if let Ok(stat_name) = CStr::from_ptr(name).to_str() {
            if let Some(client) = STEAM_CLIENT.read().as_ref() {
                let app_id = *CURRENT_APP_ID.read();
                if let Some(value) = client.get_stat_float(app_id, stat_name) {
                    *data = value;
                    return true;
                }
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUserStats_SetStatInt(name: *const c_char, data: i32) -> bool {
    if name.is_null() {
        return false;
    }

    unsafe {
        if let Ok(stat_name) = CStr::from_ptr(name).to_str() {
            if let Some(client) = STEAM_CLIENT.read().as_ref() {
                let app_id = *CURRENT_APP_ID.read();
                client.set_stat_int(app_id, stat_name, data);
                return true;
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUserStats_SetStatFloat(name: *const c_char, data: f32) -> bool {
    if name.is_null() {
        return false;
    }

    unsafe {
        if let Ok(stat_name) = CStr::from_ptr(name).to_str() {
            if let Some(client) = STEAM_CLIENT.read().as_ref() {
                let app_id = *CURRENT_APP_ID.read();
                client.set_stat_float(app_id, stat_name, data);
                return true;
            }
        }
    }
    false
}
