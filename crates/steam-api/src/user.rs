use crate::STEAM_CLIENT;

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUser_GetSteamID() -> u64 {
    STEAM_CLIENT
        .read()
        .as_ref()
        .map(|c| c.get_steam_id())
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUser_BLoggedOn() -> bool {
    STEAM_CLIENT.read().is_some()
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamUser_GetPlayerSteamLevel() -> i32 {
    STEAM_CLIENT
        .read()
        .as_ref()
        .and_then(|c| {
            let profile = c.get_user_profile();
            Some(profile.level as i32)
        })
        .unwrap_or(1)
}
