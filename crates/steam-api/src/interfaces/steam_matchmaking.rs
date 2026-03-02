use crate::callbacks::*;
use oracle_networking::matchmaking::MatchmakingManager;
use std::ffi::{c_void, CStr};
use std::sync::Arc;
use parking_lot::RwLock;

pub const STEAMMATCHMAKING_INTERFACE_VERSION: &str = "SteamMatchMaking009";

#[repr(C)]
pub struct ISteamMatchmaking {
    vtable: *const ISteamMatchmakingVTable,
    manager: Arc<RwLock<MatchmakingManager>>,
}

#[repr(C)]
pub struct ISteamMatchmakingVTable {
    pub get_favorite_game_count: unsafe extern "C" fn(*mut ISteamMatchmaking) -> i32,
    pub get_favorite_game: unsafe extern "C" fn(*mut ISteamMatchmaking, i32, *mut u32, *mut u32, *mut u16, *mut u16, *mut u32, *mut u32) -> bool,
    pub add_favorite_game: unsafe extern "C" fn(*mut ISteamMatchmaking, u32, u32, u16, u16, u32, u32) -> i32,
    pub remove_favorite_game: unsafe extern "C" fn(*mut ISteamMatchmaking, u32, u32, u16, u16, u32) -> bool,
    pub request_lobby_list: unsafe extern "C" fn(*mut ISteamMatchmaking) -> u64,
    pub add_request_lobby_list_string_filter: unsafe extern "C" fn(*mut ISteamMatchmaking, *const i8, *const i8, i32),
    pub add_request_lobby_list_numerical_filter: unsafe extern "C" fn(*mut ISteamMatchmaking, *const i8, i32, i32),
    pub add_request_lobby_list_near_value_filter: unsafe extern "C" fn(*mut ISteamMatchmaking, *const i8, i32),
    pub add_request_lobby_list_filter_slots_available: unsafe extern "C" fn(*mut ISteamMatchmaking, i32),
    pub add_request_lobby_list_distance_filter: unsafe extern "C" fn(*mut ISteamMatchmaking, i32),
    pub add_request_lobby_list_result_count_filter: unsafe extern "C" fn(*mut ISteamMatchmaking, i32),
    pub add_request_lobby_list_compatible_members_filter: unsafe extern "C" fn(*mut ISteamMatchmaking, u64),
    pub get_lobby_by_index: unsafe extern "C" fn(*mut ISteamMatchmaking, i32) -> u64,
    pub create_lobby: unsafe extern "C" fn(*mut ISteamMatchmaking, i32, i32) -> u64,
    pub join_lobby: unsafe extern "C" fn(*mut ISteamMatchmaking, u64) -> u64,
    pub leave_lobby: unsafe extern "C" fn(*mut ISteamMatchmaking, u64),
    pub invite_user_to_lobby: unsafe extern "C" fn(*mut ISteamMatchmaking, u64, u64) -> bool,
    pub get_num_lobby_members: unsafe extern "C" fn(*mut ISteamMatchmaking, u64) -> i32,
    pub get_lobby_member_by_index: unsafe extern "C" fn(*mut ISteamMatchmaking, u64, i32) -> u64,
    pub get_lobby_data: unsafe extern "C" fn(*mut ISteamMatchmaking, u64, *const i8) -> *const i8,
    pub set_lobby_data: unsafe extern "C" fn(*mut ISteamMatchmaking, u64, *const i8, *const i8) -> bool,
    pub get_lobby_data_count: unsafe extern "C" fn(*mut ISteamMatchmaking, u64) -> i32,
    pub get_lobby_data_by_index: unsafe extern "C" fn(*mut ISteamMatchmaking, u64, i32, *mut i8, i32, *mut i8, i32) -> bool,
    pub delete_lobby_data: unsafe extern "C" fn(*mut ISteamMatchmaking, u64, *const i8) -> bool,
    pub get_lobby_member_data: unsafe extern "C" fn(*mut ISteamMatchmaking, u64, u64, *const i8) -> *const i8,
    pub set_lobby_member_data: unsafe extern "C" fn(*mut ISteamMatchmaking, u64, *const i8, *const i8),
    pub send_lobby_chat_msg: unsafe extern "C" fn(*mut ISteamMatchmaking, u64, *const c_void, i32) -> bool,
    pub get_lobby_chat_entry: unsafe extern "C" fn(*mut ISteamMatchmaking, u64, i32, *mut u64, *mut c_void, i32, *mut i32) -> i32,
    pub request_lobby_data: unsafe extern "C" fn(*mut ISteamMatchmaking, u64) -> bool,
    pub set_lobby_game_server: unsafe extern "C" fn(*mut ISteamMatchmaking, u64, u32, u16, u64),
    pub get_lobby_game_server: unsafe extern "C" fn(*mut ISteamMatchmaking, u64, *mut u32, *mut u16, *mut u64) -> bool,
    pub set_lobby_member_limit: unsafe extern "C" fn(*mut ISteamMatchmaking, u64, i32) -> bool,
    pub get_lobby_member_limit: unsafe extern "C" fn(*mut ISteamMatchmaking, u64) -> i32,
    pub set_lobby_type: unsafe extern "C" fn(*mut ISteamMatchmaking, u64, i32) -> bool,
    pub set_lobby_joinable: unsafe extern "C" fn(*mut ISteamMatchmaking, u64, bool) -> bool,
    pub get_lobby_owner: unsafe extern "C" fn(*mut ISteamMatchmaking, u64) -> u64,
    pub set_lobby_owner: unsafe extern "C" fn(*mut ISteamMatchmaking, u64, u64) -> bool,
    pub set_linked_lobby: unsafe extern "C" fn(*mut ISteamMatchmaking, u64, u64) -> bool,
}

impl ISteamMatchmaking {
    pub fn new(matchmaking_manager: Arc<RwLock<MatchmakingManager>>) -> Box<Self> {
        Box::new(ISteamMatchmaking {
            vtable: &STEAM_MATCHMAKING_VTABLE,
            manager: matchmaking_manager,
        })
    }
}

unsafe extern "C" fn get_favorite_game_count(this: *mut ISteamMatchmaking) -> i32 {
    let mm = &*this;
    mm.manager.read().get_favorite_game_count()
}

unsafe extern "C" fn get_favorite_game(
    this: *mut ISteamMatchmaking,
    game_index: i32,
    app_id: *mut u32,
    ip: *mut u32,
    conn_port: *mut u16,
    query_port: *mut u16,
    flags: *mut u32,
    time32_last_played_on_server: *mut u32,
) -> bool {
    let mm = &*this;
    let manager = mm.manager.read();
    
    match manager.get_favorite_game(game_index) {
        Some(fav) => {
            if !app_id.is_null() { *app_id = fav.app_id; }
            if !ip.is_null() { *ip = fav.ip; }
            if !conn_port.is_null() { *conn_port = fav.conn_port; }
            if !query_port.is_null() { *query_port = fav.query_port; }
            if !flags.is_null() { *flags = fav.flags; }
            if !time32_last_played_on_server.is_null() { 
                *time32_last_played_on_server = fav.last_played; 
            }
            true
        }
        None => false,
    }
}

unsafe extern "C" fn add_favorite_game(
    this: *mut ISteamMatchmaking,
    app_id: u32,
    ip: u32,
    conn_port: u16,
    query_port: u16,
    flags: u32,
    time32_last_played_on_server: u32,
) -> i32 {
    let mm = &*this;
    let mut manager = mm.manager.write();
    
    manager.add_favorite_game(app_id, ip, conn_port, query_port, flags, time32_last_played_on_server)
}

unsafe extern "C" fn remove_favorite_game(
    this: *mut ISteamMatchmaking,
    app_id: u32,
    ip: u32,
    conn_port: u16,
    query_port: u16,
    flags: u32,
) -> bool {
    let mm = &*this;
    let mut manager = mm.manager.write();
    
    manager.remove_favorite_game(app_id, ip, conn_port, query_port, flags)
}

unsafe extern "C" fn request_lobby_list(this: *mut ISteamMatchmaking) -> u64 {
    let mm = &*this;
    let manager = mm.manager.read();
    
    let call_handle = oracle_core::callbacks::generate_api_call_handle();
    let manager_clone = mm.manager.clone();
    
    std::thread::spawn(move || {
        let manager = manager_clone.read();
        let lobbies = manager.request_lobby_list();
        
        oracle_core::callbacks::complete_api_call(
            call_handle,
            LobbyMatchList_t {
                lobbies_matching: lobbies.len() as u32,
            },
        );
    });
    
    call_handle
}

unsafe extern "C" fn add_request_lobby_list_string_filter(
    this: *mut ISteamMatchmaking,
    key_to_match: *const i8,
    value_to_match: *const i8,
    comparison_type: i32,
) {
    let mm = &*this;
    let mut manager = mm.manager.write();
    
    let key = if key_to_match.is_null() {
        return;
    } else {
        CStr::from_ptr(key_to_match).to_string_lossy().into_owned()
    };
    
    let value = if value_to_match.is_null() {
        String::new()
    } else {
        CStr::from_ptr(value_to_match).to_string_lossy().into_owned()
    };
    
    manager.add_string_filter(key, value, comparison_type);
}

unsafe extern "C" fn add_request_lobby_list_numerical_filter(
    this: *mut ISteamMatchmaking,
    key_to_match: *const i8,
    value_to_match: i32,
    comparison_type: i32,
) {
    let mm = &*this;
    let mut manager = mm.manager.write();
    
    let key = if key_to_match.is_null() {
        return;
    } else {
        CStr::from_ptr(key_to_match).to_string_lossy().into_owned()
    };
    
    manager.add_numerical_filter(key, value_to_match, comparison_type);
}

unsafe extern "C" fn add_request_lobby_list_near_value_filter(
    this: *mut ISteamMatchmaking,
    key_to_match: *const i8,
    value_to_be_close_to: i32,
) {
    let mm = &*this;
    let mut manager = mm.manager.write();
    
    let key = if key_to_match.is_null() {
        return;
    } else {
        CStr::from_ptr(key_to_match).to_string_lossy().into_owned()
    };
    
    manager.add_near_value_filter(key, value_to_be_close_to);
}

unsafe extern "C" fn add_request_lobby_list_filter_slots_available(
    this: *mut ISteamMatchmaking,
    slots_available: i32,
) {
    let mm = &*this;
    let mut manager = mm.manager.write();
    
    manager.add_slots_available_filter(slots_available);
}

unsafe extern "C" fn add_request_lobby_list_distance_filter(
    this: *mut ISteamMatchmaking,
    lobby_distance_filter: i32,
) {
    let mm = &*this;
    let mut manager = mm.manager.write();
    
    manager.add_distance_filter(lobby_distance_filter);
}

unsafe extern "C" fn add_request_lobby_list_result_count_filter(
    this: *mut ISteamMatchmaking,
    max_results: i32,
) {
    let mm = &*this;
    let mut manager = mm.manager.write();
    
    manager.set_result_count_filter(max_results);
}

unsafe extern "C" fn add_request_lobby_list_compatible_members_filter(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
) {
    let mm = &*this;
    let mut manager = mm.manager.write();
    
    manager.add_compatible_members_filter(steamid_lobby);
}

unsafe extern "C" fn get_lobby_by_index(this: *mut ISteamMatchmaking, lobby_index: i32) -> u64 {
    let mm = &*this;
    mm.manager.read().get_lobby_by_index(lobby_index)
}

unsafe extern "C" fn create_lobby(
    this: *mut ISteamMatchmaking,
    lobby_type: i32,
    max_members: i32,
) -> u64 {
    let mm = &*this;
    let manager_clone = mm.manager.clone();
    
    let call_handle = oracle_core::callbacks::generate_api_call_handle();
    
    std::thread::spawn(move || {
        let mut manager = manager_clone.write();
        
        match manager.create_lobby(lobby_type, max_members) {
            Ok(lobby_id) => {
                oracle_core::callbacks::complete_api_call(
                    call_handle,
                    LobbyCreated_t {
                        result: 1, // k_EResultOK
                        steamid_lobby: lobby_id,
                    },
                );
            }
            Err(e) => {
                log::error!("Failed to create lobby: {}", e);
                oracle_core::callbacks::complete_api_call(
                    call_handle,
                    LobbyCreated_t {
                        result: 2, // k_EResultFail
                        steamid_lobby: 0,
                    },
                );
            }
        }
    });
    
    call_handle
}

unsafe extern "C" fn join_lobby(this: *mut ISteamMatchmaking, steamid_lobby: u64) -> u64 {
    let mm = &*this;
    let manager_clone = mm.manager.clone();
    
    let call_handle = oracle_core::callbacks::generate_api_call_handle();
    
    std::thread::spawn(move || {
        let mut manager = manager_clone.write();
        
        match manager.join_lobby(steamid_lobby) {
            Ok(_) => {
                oracle_core::callbacks::complete_api_call(
                    call_handle,
                    LobbyEnter_t {
                        steamid_lobby,
                        chat_permissions: 0,
                        blocked: false,
                        chat_room_enter_response: 1, // k_EChatRoomEnterResponseSuccess
                    },
                );
            }
            Err(e) => {
                log::error!("Failed to join lobby: {}", e);
                oracle_core::callbacks::complete_api_call(
                    call_handle,
                    LobbyEnter_t {
                        steamid_lobby,
                        chat_permissions: 0,
                        blocked: false,
                        chat_room_enter_response: 2, // k_EChatRoomEnterResponseDoesntExist
                    },
                );
            }
        }
    });
    
    call_handle
}

unsafe extern "C" fn leave_lobby(this: *mut ISteamMatchmaking, steamid_lobby: u64) {
    let mm = &*this;
    let mut manager = mm.manager.write();
    
    manager.leave_lobby(steamid_lobby);
}

unsafe extern "C" fn invite_user_to_lobby(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
    steamid_invitee: u64,
) -> bool {
    let mm = &*this;
    let manager = mm.manager.read();
    
    manager.invite_user_to_lobby(steamid_lobby, steamid_invitee)
}

unsafe extern "C" fn get_num_lobby_members(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
) -> i32 {
    let mm = &*this;
    mm.manager.read().get_num_lobby_members(steamid_lobby)
}

unsafe extern "C" fn get_lobby_member_by_index(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
    member_index: i32,
) -> u64 {
    let mm = &*this;
    mm.manager.read().get_lobby_member_by_index(steamid_lobby, member_index)
}

unsafe extern "C" fn get_lobby_data(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
    key: *const i8,
) -> *const i8 {
    let mm = &*this;
    let manager = mm.manager.read();
    
    let key_str = if key.is_null() {
        return std::ptr::null();
    } else {
        CStr::from_ptr(key).to_string_lossy().into_owned()
    };
    
    manager.get_lobby_data_ptr(steamid_lobby, &key_str)
}

unsafe extern "C" fn set_lobby_data(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
    key: *const i8,
    value: *const i8,
) -> bool {
    let mm = &*this;
    let mut manager = mm.manager.write();
    
    let key_str = if key.is_null() {
        return false;
    } else {
        CStr::from_ptr(key).to_string_lossy().into_owned()
    };
    
    let value_str = if value.is_null() {
        String::new()
    } else {
        CStr::from_ptr(value).to_string_lossy().into_owned()
    };
    
    manager.set_lobby_data(steamid_lobby, key_str, value_str)
}

unsafe extern "C" fn get_lobby_data_count(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
) -> i32 {
    let mm = &*this;
    mm.manager.read().get_lobby_data_count(steamid_lobby)
}

unsafe extern "C" fn get_lobby_data_by_index(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
    lobby_data_index: i32,
    key: *mut i8,
    key_size: i32,
    value: *mut i8,
    value_size: i32,
) -> bool {
    let mm = &*this;
    let manager = mm.manager.read();
    
    if key.is_null() || value.is_null() || key_size <= 0 || value_size <= 0 {
        return false;
    }
    
    match manager.get_lobby_data_by_index(steamid_lobby, lobby_data_index) {
        Some((k, v)) => {
            let key_bytes = k.as_bytes();
            let key_copy = std::cmp::min(key_bytes.len(), (key_size - 1) as usize);
            std::ptr::copy_nonoverlapping(key_bytes.as_ptr(), key as *mut u8, key_copy);
            *(key as *mut u8).add(key_copy) = 0;
            
            let value_bytes = v.as_bytes();
            let value_copy = std::cmp::min(value_bytes.len(), (value_size - 1) as usize);
            std::ptr::copy_nonoverlapping(value_bytes.as_ptr(), value as *mut u8, value_copy);
            *(value as *mut u8).add(value_copy) = 0;
            
            true
        }
        None => false,
    }
}

unsafe extern "C" fn delete_lobby_data(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
    key: *const i8,
) -> bool {
    let mm = &*this;
    let mut manager = mm.manager.write();
    
    let key_str = if key.is_null() {
        return false;
    } else {
        CStr::from_ptr(key).to_string_lossy().into_owned()
    };
    
    manager.delete_lobby_data(steamid_lobby, key_str)
}

unsafe extern "C" fn get_lobby_member_data(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
    steamid_user: u64,
    key: *const i8,
) -> *const i8 {
    let mm = &*this;
    let manager = mm.manager.read();
    
    let key_str = if key.is_null() {
        return std::ptr::null();
    } else {
        CStr::from_ptr(key).to_string_lossy().into_owned()
    };
    
    manager.get_lobby_member_data_ptr(steamid_lobby, steamid_user, &key_str)
}

unsafe extern "C" fn set_lobby_member_data(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
    key: *const i8,
    value: *const i8,
) {
    let mm = &*this;
    let mut manager = mm.manager.write();
    
    let key_str = if key.is_null() {
        return;
    } else {
        CStr::from_ptr(key).to_string_lossy().into_owned()
    };
    
    let value_str = if value.is_null() {
        String::new()
    } else {
        CStr::from_ptr(value).to_string_lossy().into_owned()
    };
    
    manager.set_lobby_member_data(steamid_lobby, key_str, value_str);
}

unsafe extern "C" fn send_lobby_chat_msg(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
    msg_body: *const c_void,
    msg_body_size: i32,
) -> bool {
    let mm = &*this;
    let mut manager = mm.manager.write();
    
    if msg_body.is_null() || msg_body_size <= 0 {
        return false;
    }
    
    let data = std::slice::from_raw_parts(msg_body as *const u8, msg_body_size as usize);
    
    manager.send_lobby_chat_msg(steamid_lobby, data.to_vec())
}

unsafe extern "C" fn get_lobby_chat_entry(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
    chat_id: i32,
    steamid_user: *mut u64,
    msg_body: *mut c_void,
    msg_body_size: i32,
    chat_entry_type: *mut i32,
) -> i32 {
    let mm = &*this;
    let manager = mm.manager.read();
    
    match manager.get_lobby_chat_entry(steamid_lobby, chat_id) {
        Some(entry) => {
            if !steamid_user.is_null() {
                *steamid_user = entry.sender;
            }
            if !chat_entry_type.is_null() {
                *chat_entry_type = entry.entry_type;
            }
            if !msg_body.is_null() && msg_body_size > 0 {
                let copy_len = std::cmp::min(entry.data.len(), msg_body_size as usize);
                std::ptr::copy_nonoverlapping(
                    entry.data.as_ptr(),
                    msg_body as *mut u8,
                    copy_len,
                );
                return copy_len as i32;
            }
            entry.data.len() as i32
        }
        None => 0,
    }
}

unsafe extern "C" fn request_lobby_data(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
) -> bool {
    let mm = &*this;
    let manager = mm.manager.read();
    
    manager.request_lobby_data(steamid_lobby)
}

unsafe extern "C" fn set_lobby_game_server(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
    game_server_ip: u32,
    game_server_port: u16,
    steamid_game_server: u64,
) {
    let mm = &*this;
    let mut manager = mm.manager.write();
    
    manager.set_lobby_game_server(steamid_lobby, game_server_ip, game_server_port, steamid_game_server);
}

unsafe extern "C" fn get_lobby_game_server(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
    game_server_ip: *mut u32,
    game_server_port: *mut u16,
    steamid_game_server: *mut u64,
) -> bool {
    let mm = &*this;
    let manager = mm.manager.read();
    
    match manager.get_lobby_game_server(steamid_lobby) {
        Some((ip, port, server_id)) => {
            if !game_server_ip.is_null() {
                *game_server_ip = ip;
            }
            if !game_server_port.is_null() {
                *game_server_port = port;
            }
            if !steamid_game_server.is_null() {
                *steamid_game_server = server_id;
            }
            true
        }
        None => false,
    }
}

unsafe extern "C" fn set_lobby_member_limit(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
    max_members: i32,
) -> bool {
    let mm = &*this;
    let mut manager = mm.manager.write();
    
    manager.set_lobby_member_limit(steamid_lobby, max_members)
}

unsafe extern "C" fn get_lobby_member_limit(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
) -> i32 {
    let mm = &*this;
    mm.manager.read().get_lobby_member_limit(steamid_lobby)
}

unsafe extern "C" fn set_lobby_type(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
    lobby_type: i32,
) -> bool {
    let mm = &*this;
    let mut manager = mm.manager.write();
    
    manager.set_lobby_type(steamid_lobby, lobby_type)
}

unsafe extern "C" fn set_lobby_joinable(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
    joinable: bool,
) -> bool {
    let mm = &*this;
    let mut manager = mm.manager.write();
    
    manager.set_lobby_joinable(steamid_lobby, joinable)
}

unsafe extern "C" fn get_lobby_owner(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
) -> u64 {
    let mm = &*this;
    mm.manager.read().get_lobby_owner(steamid_lobby)
}

unsafe extern "C" fn set_lobby_owner(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
    steamid_new_owner: u64,
) -> bool {
    let mm = &*this;
    let mut manager = mm.manager.write();
    
    manager.set_lobby_owner(steamid_lobby, steamid_new_owner)
}

unsafe extern "C" fn set_linked_lobby(
    this: *mut ISteamMatchmaking,
    steamid_lobby: u64,
    steamid_lobby_dependent: u64,
) -> bool {
    let mm = &*this;
    let mut manager = mm.manager.write();
    
    manager.set_linked_lobby(steamid_lobby, steamid_lobby_dependent)
}

static STEAM_MATCHMAKING_VTABLE: ISteamMatchmakingVTable = ISteamMatchmakingVTable {
    get_favorite_game_count,
    get_favorite_game,
    add_favorite_game,
    remove_favorite_game,
    request_lobby_list,
    add_request_lobby_list_string_filter,
    add_request_lobby_list_numerical_filter,
    add_request_lobby_list_near_value_filter,
    add_request_lobby_list_filter_slots_available,
    add_request_lobby_list_distance_filter,
    add_request_lobby_list_result_count_filter,
    add_request_lobby_list_compatible_members_filter,
    get_lobby_by_index,
    create_lobby,
    join_lobby,
    leave_lobby,
    invite_user_to_lobby,
    get_num_lobby_members,
    get_lobby_member_by_index,
    get_lobby_data,
    set_lobby_data,
    get_lobby_data_count,
    get_lobby_data_by_index,
    delete_lobby_data,
    get_lobby_member_data,
    set_lobby_member_data,
    send_lobby_chat_msg,
    get_lobby_chat_entry,
    request_lobby_data,
    set_lobby_game_server,
    get_