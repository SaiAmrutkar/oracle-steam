use crate::{CURRENT_APP_ID, STEAM_CLIENT};
use std::ffi::{c_char, CStr};

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_CreateLobby(
    _lobby_type: i32,
    max_members: i32,
) -> u64 {
    if let Some(client) = STEAM_CLIENT.read().as_ref() {
        let app_id = *CURRENT_APP_ID.read();
        let lobby_id = client.create_lobby(app_id, max_members as u32);
        println!("[OracleSteam] Lobby created: {}", lobby_id);
        return lobby_id;
    }
    0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_JoinLobby(lobby_id: u64) -> bool {
    if let Some(client) = STEAM_CLIENT.read().as_ref() {
        if client.join_lobby(lobby_id).is_ok() {
            println!("[OracleSteam] Joined lobby: {}", lobby_id);
            return true;
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_LeaveLobby(lobby_id: u64) {
    if let Some(client) = STEAM_CLIENT.read().as_ref() {
        client.leave_lobby(lobby_id);
        println!("[OracleSteam] Left lobby: {}", lobby_id);
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_SetLobbyData(
    lobby_id: u64,
    key: *const c_char,
    value: *const c_char,
) -> bool {
    if key.is_null() || value.is_null() {
        return false;
    }

    unsafe {
        if let (Ok(k), Ok(v)) = (CStr::from_ptr(key).to_str(), CStr::from_ptr(value).to_str()) {
            if let Some(client) = STEAM_CLIENT.read().as_ref() {
                return client
                    .set_lobby_data(lobby_id, k.to_string(), v.to_string())
                    .is_ok();
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_GetLobbyData(
    _lobby_id: u64,
    _key: *const c_char,
) -> *const c_char {
    b"\0".as_ptr() as *const c_char
}

// Additional ISteamMatchmaking functions

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_GetNumLobbyMembers(lobby_id: u64) -> i32 {
    if let Some(client) = STEAM_CLIENT.read().as_ref() {
        return client.get_lobby_member_count(lobby_id) as i32;
    }
    0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_GetLobbyMemberByIndex(lobby_id: u64, member: i32) -> u64 {
    if let Some(client) = STEAM_CLIENT.read().as_ref() {
        return client.get_lobby_member_by_index(lobby_id, member as usize);
    }
    0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_GetLobbyOwner(lobby_id: u64) -> u64 {
    if let Some(client) = STEAM_CLIENT.read().as_ref() {
        return client.get_lobby_owner(lobby_id);
    }
    0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_SetLobbyType(lobby_id: u64, lobby_type: i32) -> bool {
    if let Some(client) = STEAM_CLIENT.read().as_ref() {
        return client.set_lobby_type(lobby_id, lobby_type as u32).is_ok();
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_SetLobbyJoinable(lobby_id: u64, joinable: bool) -> bool {
    if let Some(client) = STEAM_CLIENT.read().as_ref() {
        return client.set_lobby_joinable(lobby_id, joinable).is_ok();
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_GetLobbyMemberLimit(lobby_id: u64) -> i32 {
    if let Some(client) = STEAM_CLIENT.read().as_ref() {
        return client.get_lobby_member_limit(lobby_id) as i32;
    }
    0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_SetLobbyMemberLimit(lobby_id: u64, max_members: i32) -> bool {
    if let Some(client) = STEAM_CLIENT.read().as_ref() {
        return client.set_lobby_member_limit(lobby_id, max_members as u32).is_ok();
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_InviteUserToLobby(lobby_id: u64, steam_id_invitee: u64) -> bool {
    if let Some(client) = STEAM_CLIENT.read().as_ref() {
        return client.invite_user_to_lobby(lobby_id, steam_id_invitee).is_ok();
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_RequestLobbyList() -> u64 {
    // Return async call handle
    1
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_AddRequestLobbyListStringFilter(
    key: *const c_char,
    value: *const c_char,
    comparison_type: i32,
) {
    // Store filter for next RequestLobbyList call
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_AddRequestLobbyListNumericalFilter(
    key: *const c_char,
    value_to_match: i32,
    comparison_type: i32,
) {
    // Store filter
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_AddRequestLobbyListNearValueFilter(
    key: *const c_char,
    value_to_be_close_to: i32,
) {
    // Store filter
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_AddRequestLobbyListFilterSlotsAvailable(min_slots: i32) {
    // Store filter
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_AddRequestLobbyListDistanceFilter(distance_filter: i32) {
    // Store filter
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_AddRequestLobbyListResultCountFilter(max_results: i32) {
    // Store filter
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_GetLobbyByIndex(lobby: i32) -> u64 {
    // Return lobby from last RequestLobbyList
    0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_SendLobbyChatMsg(
    lobby_id: u64,
    msg_body: *const std::ffi::c_void,
    msg_size: i32,
) -> bool {
    if msg_body.is_null() || msg_size < 1 {
        return false;
    }

    if let Some(client) = STEAM_CLIENT.read().as_ref() {
        unsafe {
            let slice = std::slice::from_raw_parts(msg_body as *const u8, msg_size as usize);
            return client.send_lobby_chat(lobby_id, slice).is_ok();
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_GetLobbyChatEntry(
    lobby_id: u64,
    chat_id: i32,
    steam_id_user: *mut u64,
    msg_data: *mut std::ffi::c_void,
    msg_size: i32,
    chat_entry_type: *mut i32,
) -> i32 {
    // Return number of bytes written
    0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_RequestLobbyData(lobby_id: u64) -> bool {
    if let Some(client) = STEAM_CLIENT.read().as_ref() {
        return client.request_lobby_data(lobby_id).is_ok();
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_SetLobbyGameServer(
    lobby_id: u64,
    game_server_ip: u32,
    game_server_port: u16,
    game_server_steam_id: u64,
) {
    if let Some(client) = STEAM_CLIENT.read().as_ref() {
        client.set_lobby_game_server(lobby_id, game_server_ip, game_server_port, game_server_steam_id).ok();
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_GetLobbyGameServer(
    lobby_id: u64,
    game_server_ip: *mut u32,
    game_server_port: *mut u16,
    game_server_steam_id: *mut u64,
) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_SetLobbyMemberData(lobby_id: u64, key: *const c_char, value: *const c_char) -> bool {
    if key.is_null() || value.is_null() {
        return false;
    }

    unsafe {
        if let (Ok(k), Ok(v)) = (CStr::from_ptr(key).to_str(), CStr::from_ptr(value).to_str()) {
            if let Some(client) = STEAM_CLIENT.read().as_ref() {
                return client.set_lobby_member_data(lobby_id, k.to_string(), v.to_string()).is_ok();
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_GetLobbyMemberData(
    lobby_id: u64,
    steam_id_user: u64,
    key: *const c_char,
) -> *const c_char {
    b"\0".as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_GetLobbyDataCount(lobby_id: u64) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_GetLobbyDataByIndex(
    lobby_id: u64,
    lobby_data: i32,
    key: *mut c_char,
    key_size: i32,
    value: *mut c_char,
    value_size: i32,
) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_DeleteLobbyData(lobby_id: u64, key: *const c_char) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMatchmaking_SetLinkedLobby(lobby_id: u64, lobby_dependent: u64) -> bool {
    false
}
