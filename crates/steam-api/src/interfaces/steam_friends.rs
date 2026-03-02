use crate::callbacks::*;
use oracle_core::friends::FriendsManager;
use parking_lot::RwLock;
use std::ffi::{c_void, CStr, CString};
use std::sync::Arc;

pub const STEAMFRIENDS_INTERFACE_VERSION: &str = "SteamFriends017";

#[repr(C)]
pub struct ISteamFriends {
    vtable: *const ISteamFriendsVTable,
    manager: Arc<RwLock<FriendsManager>>,
}

#[repr(C)]
pub struct ISteamFriendsVTable {
    pub get_persona_name: unsafe extern "C" fn(*mut ISteamFriends) -> *const i8,
    pub set_persona_name: unsafe extern "C" fn(*mut ISteamFriends, *const i8) -> u64,
    pub get_persona_state: unsafe extern "C" fn(*mut ISteamFriends) -> i32,
    pub get_friend_count: unsafe extern "C" fn(*mut ISteamFriends, i32) -> i32,
    pub get_friend_by_index: unsafe extern "C" fn(*mut ISteamFriends, i32, i32) -> u64,
    pub get_friend_relationship: unsafe extern "C" fn(*mut ISteamFriends, u64) -> i32,
    pub get_friend_persona_state: unsafe extern "C" fn(*mut ISteamFriends, u64) -> i32,
    pub get_friend_persona_name: unsafe extern "C" fn(*mut ISteamFriends, u64) -> *const i8,
    pub get_friend_game_played:
        unsafe extern "C" fn(*mut ISteamFriends, u64, *mut FriendGameInfo_t) -> bool,
    pub get_friend_persona_name_history:
        unsafe extern "C" fn(*mut ISteamFriends, u64, i32) -> *const i8,
    pub get_friend_steam_level: unsafe extern "C" fn(*mut ISteamFriends, u64) -> i32,
    pub get_player_nickname: unsafe extern "C" fn(*mut ISteamFriends, u64) -> *const i8,
    pub get_friends_group_count: unsafe extern "C" fn(*mut ISteamFriends) -> i32,
    pub get_friends_group_id_by_index: unsafe extern "C" fn(*mut ISteamFriends, i32) -> i16,
    pub get_friends_group_name: unsafe extern "C" fn(*mut ISteamFriends, i16) -> *const i8,
    pub get_friends_group_members_count: unsafe extern "C" fn(*mut ISteamFriends, i16) -> i32,
    pub get_friends_group_members_list:
        unsafe extern "C" fn(*mut ISteamFriends, i16, *mut u64, i32),
    pub has_friend: unsafe extern "C" fn(*mut ISteamFriends, u64, i32) -> bool,
    pub get_clan_count: unsafe extern "C" fn(*mut ISteamFriends) -> i32,
    pub get_clan_by_index: unsafe extern "C" fn(*mut ISteamFriends, i32) -> u64,
    pub get_clan_name: unsafe extern "C" fn(*mut ISteamFriends, u64) -> *const i8,
    pub get_clan_tag: unsafe extern "C" fn(*mut ISteamFriends, u64) -> *const i8,
    pub get_clan_activity_counts:
        unsafe extern "C" fn(*mut ISteamFriends, u64, *mut i32, *mut i32, *mut i32) -> bool,
    pub download_clan_activity_counts:
        unsafe extern "C" fn(*mut ISteamFriends, *mut u64, i32) -> u64,
    pub get_friend_count_from_source: unsafe extern "C" fn(*mut ISteamFriends, u64) -> i32,
    pub get_friend_from_source_by_index: unsafe extern "C" fn(*mut ISteamFriends, u64, i32) -> u64,
    pub is_user_in_source: unsafe extern "C" fn(*mut ISteamFriends, u64, u64) -> bool,
    pub set_in_game_voice_speaking: unsafe extern "C" fn(*mut ISteamFriends, u64, bool),
    pub activate_game_overlay: unsafe extern "C" fn(*mut ISteamFriends, *const i8),
    pub activate_game_overlay_to_user: unsafe extern "C" fn(*mut ISteamFriends, *const i8, u64),
    pub activate_game_overlay_to_web_page: unsafe extern "C" fn(*mut ISteamFriends, *const i8, i32),
    pub activate_game_overlay_to_store: unsafe extern "C" fn(*mut ISteamFriends, u32, i32),
    pub set_play_group: unsafe extern "C" fn(*mut ISteamFriends, u64),
    pub leave_play_group: unsafe extern "C" fn(*mut ISteamFriends),
    pub get_play_group_count: unsafe extern "C" fn(*mut ISteamFriends) -> i32,
    pub get_play_group_by_index: unsafe extern "C" fn(*mut ISteamFriends, i32) -> u64,
    pub activate_game_overlay_invite_dialog: unsafe extern "C" fn(*mut ISteamFriends, u64),
    pub get_small_friend_avatar: unsafe extern "C" fn(*mut ISteamFriends, u64) -> i32,
    pub get_medium_friend_avatar: unsafe extern "C" fn(*mut ISteamFriends, u64) -> i32,
    pub get_large_friend_avatar: unsafe extern "C" fn(*mut ISteamFriends, u64) -> i32,
    pub request_user_information: unsafe extern "C" fn(*mut ISteamFriends, u64, bool) -> bool,
    pub request_clan_officer_list: unsafe extern "C" fn(*mut ISteamFriends, u64) -> u64,
    pub get_clan_owner: unsafe extern "C" fn(*mut ISteamFriends, u64) -> u64,
    pub get_clan_officer_count: unsafe extern "C" fn(*mut ISteamFriends, u64) -> i32,
    pub get_clan_officer_by_index: unsafe extern "C" fn(*mut ISteamFriends, u64, i32) -> u64,
    pub get_user_restriction: unsafe extern "C" fn(*mut ISteamFriends) -> u32,
    pub set_rich_presence: unsafe extern "C" fn(*mut ISteamFriends, *const i8, *const i8) -> bool,
    pub clear_rich_presence: unsafe extern "C" fn(*mut ISteamFriends),
    pub get_friend_rich_presence:
        unsafe extern "C" fn(*mut ISteamFriends, u64, *const i8) -> *const i8,
    pub get_friend_rich_presence_key_count: unsafe extern "C" fn(*mut ISteamFriends, u64) -> i32,
    pub get_friend_rich_presence_key_by_index:
        unsafe extern "C" fn(*mut ISteamFriends, u64, i32) -> *const i8,
    pub request_friend_rich_presence: unsafe extern "C" fn(*mut ISteamFriends, u64),
    pub invite_user_to_game: unsafe extern "C" fn(*mut ISteamFriends, u64, *const i8) -> bool,
    pub get_coplay_friend_count: unsafe extern "C" fn(*mut ISteamFriends) -> i32,
    pub get_coplay_friend: unsafe extern "C" fn(*mut ISteamFriends, i32) -> u64,
    pub get_friend_coplay_time: unsafe extern "C" fn(*mut ISteamFriends, u64) -> i32,
    pub get_friend_coplay_game: unsafe extern "C" fn(*mut ISteamFriends, u64) -> u32,
    pub join_clan_chat_room: unsafe extern "C" fn(*mut ISteamFriends, u64) -> u64,
    pub leave_clan_chat_room: unsafe extern "C" fn(*mut ISteamFriends, u64) -> bool,
    pub get_clan_chat_member_count: unsafe extern "C" fn(*mut ISteamFriends, u64) -> i32,
    pub get_chat_member_by_index: unsafe extern "C" fn(*mut ISteamFriends, u64, i32) -> u64,
    pub send_clan_chat_message: unsafe extern "C" fn(*mut ISteamFriends, u64, *const i8) -> bool,
    pub get_clan_chat_message: unsafe extern "C" fn(
        *mut ISteamFriends,
        u64,
        i32,
        *mut c_void,
        i32,
        *mut i32,
        *mut u64,
    ) -> i32,
    pub is_clan_chat_admin: unsafe extern "C" fn(*mut ISteamFriends, u64, u64) -> bool,
    pub is_clan_chat_window_open_in_steam: unsafe extern "C" fn(*mut ISteamFriends, u64) -> bool,
    pub open_clan_chat_window_in_steam: unsafe extern "C" fn(*mut ISteamFriends, u64) -> bool,
    pub close_clan_chat_window_in_steam: unsafe extern "C" fn(*mut ISteamFriends, u64) -> bool,
    pub set_listen_for_friends_messages: unsafe extern "C" fn(*mut ISteamFriends, bool) -> bool,
    pub reply_to_friend_message: unsafe extern "C" fn(*mut ISteamFriends, u64, *const i8) -> bool,
    pub get_friend_message:
        unsafe extern "C" fn(*mut ISteamFriends, u64, i32, *mut c_void, i32, *mut i32) -> i32,
    pub get_follower_count: unsafe extern "C" fn(*mut ISteamFriends, u64) -> u64,
    pub is_following: unsafe extern "C" fn(*mut ISteamFriends, u64) -> u64,
    pub enumerate_following_list: unsafe extern "C" fn(*mut ISteamFriends, u32) -> u64,
    pub is_clan_public: unsafe extern "C" fn(*mut ISteamFriends, u64) -> bool,
    pub is_clan_official_game_group: unsafe extern "C" fn(*mut ISteamFriends, u64) -> bool,
    pub get_num_chats_with_unread_priority_messages:
        unsafe extern "C" fn(*mut ISteamFriends) -> i32,
    pub activate_game_overlay_remove_friend_dialog: unsafe extern "C" fn(*mut ISteamFriends, u64),
    pub activate_game_overlay_invite_dialog_connections:
        unsafe extern "C" fn(*mut ISteamFriends, u64),
    pub register_protocol_in_overlay_browser:
        unsafe extern "C" fn(*mut ISteamFriends, *const i8) -> bool,
    pub activate_game_overlay_invite_dialog_steam: unsafe extern "C" fn(*mut ISteamFriends, u64),
    pub activate_game_overlay_to_user_generated_content:
        unsafe extern "C" fn(*mut ISteamFriends, *const i8, u32),
}

#[repr(C)]
pub struct FriendGameInfo_t {
    pub game_id: u64,
    pub game_ip: u32,
    pub game_port: u16,
    pub query_port: u16,
    pub steamid_lobby: u64,
}

impl ISteamFriends {
    pub fn new(friends_manager: Arc<RwLock<FriendsManager>>) -> Box<Self> {
        Box::new(ISteamFriends {
            vtable: &STEAM_FRIENDS_VTABLE,
            manager: friends_manager,
        })
    }
}

unsafe extern "C" fn get_persona_name(this: *mut ISteamFriends) -> *const i8 {
    let friends = &*this;
    let manager = friends.manager.read();
    manager.get_persona_name_ptr()
}

unsafe extern "C" fn set_persona_name(this: *mut ISteamFriends, name: *const i8) -> u64 {
    let friends = &*this;
    let mut manager = friends.manager.write();

    let name_str = if name.is_null() {
        String::new()
    } else {
        CStr::from_ptr(name).to_string_lossy().into_owned()
    };

    let call_handle = oracle_core::callbacks::generate_api_call_handle();

    match manager.set_persona_name(name_str) {
        Ok(_) => {
            oracle_core::callbacks::complete_api_call(
                call_handle,
                SetPersonaNameResponse_t {
                    success: true,
                    local_success: true,
                    result: 1,
                },
            );
        }
        Err(e) => {
            log::error!("Failed to set persona name: {}", e);
            oracle_core::callbacks::complete_api_call(
                call_handle,
                SetPersonaNameResponse_t {
                    success: false,
                    local_success: false,
                    result: 2,
                },
            );
        }
    }

    call_handle
}

unsafe extern "C" fn get_persona_state(this: *mut ISteamFriends) -> i32 {
    let friends = &*this;
    friends.manager.read().get_persona_state()
}

unsafe extern "C" fn get_friend_count(this: *mut ISteamFriends, friend_flags: i32) -> i32 {
    let friends = &*this;
    friends.manager.read().get_friend_count(friend_flags)
}

unsafe extern "C" fn get_friend_by_index(
    this: *mut ISteamFriends,
    friend_index: i32,
    friend_flags: i32,
) -> u64 {
    let friends = &*this;
    friends
        .manager
        .read()
        .get_friend_by_index(friend_index, friend_flags)
}

unsafe extern "C" fn get_friend_relationship(this: *mut ISteamFriends, steamid: u64) -> i32 {
    let friends = &*this;
    friends.manager.read().get_friend_relationship(steamid)
}

unsafe extern "C" fn get_friend_persona_state(this: *mut ISteamFriends, steamid: u64) -> i32 {
    let friends = &*this;
    friends.manager.read().get_friend_persona_state(steamid)
}

unsafe extern "C" fn get_friend_persona_name(this: *mut ISteamFriends, steamid: u64) -> *const i8 {
    let friends = &*this;
    friends.manager.read().get_friend_persona_name_ptr(steamid)
}

unsafe extern "C" fn get_friend_game_played(
    this: *mut ISteamFriends,
    steamid: u64,
    friend_game_info: *mut FriendGameInfo_t,
) -> bool {
    let friends = &*this;
    let manager = friends.manager.read();

    if friend_game_info.is_null() {
        return false;
    }

    match manager.get_friend_game_played(steamid) {
        Some(game_info) => {
            *friend_game_info = FriendGameInfo_t {
                game_id: game_info.game_id,
                game_ip: game_info.game_ip,
                game_port: game_info.game_port,
                query_port: game_info.query_port,
                steamid_lobby: game_info.lobby_id,
            };
            true
        }
        None => false,
    }
}

unsafe extern "C" fn get_friend_persona_name_history(
    this: *mut ISteamFriends,
    steamid: u64,
    index: i32,
) -> *const i8 {
    let friends = &*this;
    friends
        .manager
        .read()
        .get_friend_persona_name_history_ptr(steamid, index)
}

unsafe extern "C" fn get_friend_steam_level(this: *mut ISteamFriends, steamid: u64) -> i32 {
    let friends = &*this;
    friends.manager.read().get_friend_steam_level(steamid)
}

unsafe extern "C" fn get_player_nickname(this: *mut ISteamFriends, steamid: u64) -> *const i8 {
    let friends = &*this;
    friends.manager.read().get_player_nickname_ptr(steamid)
}

unsafe extern "C" fn get_friends_group_count(this: *mut ISteamFriends) -> i32 {
    let friends = &*this;
    friends.manager.read().get_friends_group_count()
}

unsafe extern "C" fn get_friends_group_id_by_index(this: *mut ISteamFriends, index: i32) -> i16 {
    let friends = &*this;
    friends.manager.read().get_friends_group_id_by_index(index)
}

unsafe extern "C" fn get_friends_group_name(this: *mut ISteamFriends, group_id: i16) -> *const i8 {
    let friends = &*this;
    friends.manager.read().get_friends_group_name_ptr(group_id)
}

unsafe extern "C" fn get_friends_group_members_count(
    this: *mut ISteamFriends,
    group_id: i16,
) -> i32 {
    let friends = &*this;
    friends
        .manager
        .read()
        .get_friends_group_members_count(group_id)
}

unsafe extern "C" fn get_friends_group_members_list(
    this: *mut ISteamFriends,
    group_id: i16,
    out_steamid_members: *mut u64,
    members_count: i32,
) {
    let friends = &*this;
    let manager = friends.manager.read();

    if out_steamid_members.is_null() || members_count <= 0 {
        return;
    }

    let members = manager.get_friends_group_members(group_id);
    let copy_count = std::cmp::min(members.len(), members_count as usize);

    std::ptr::copy_nonoverlapping(members.as_ptr(), out_steamid_members, copy_count);
}

unsafe extern "C" fn has_friend(this: *mut ISteamFriends, steamid: u64, friend_flags: i32) -> bool {
    let friends = &*this;
    friends.manager.read().has_friend(steamid, friend_flags)
}

unsafe extern "C" fn get_clan_count(this: *mut ISteamFriends) -> i32 {
    let friends = &*this;
    friends.manager.read().get_clan_count()
}

unsafe extern "C" fn get_clan_by_index(this: *mut ISteamFriends, index: i32) -> u64 {
    let friends = &*this;
    friends.manager.read().get_clan_by_index(index)
}

unsafe extern "C" fn get_clan_name(this: *mut ISteamFriends, steamid_clan: u64) -> *const i8 {
    let friends = &*this;
    friends.manager.read().get_clan_name_ptr(steamid_clan)
}

unsafe extern "C" fn get_clan_tag(this: *mut ISteamFriends, steamid_clan: u64) -> *const i8 {
    let friends = &*this;
    friends.manager.read().get_clan_tag_ptr(steamid_clan)
}

unsafe extern "C" fn get_clan_activity_counts(
    this: *mut ISteamFriends,
    steamid_clan: u64,
    online: *mut i32,
    in_game: *mut i32,
    chatting: *mut i32,
) -> bool {
    let friends = &*this;
    let manager = friends.manager.read();

    match manager.get_clan_activity_counts(steamid_clan) {
        Some((online_count, in_game_count, chatting_count)) => {
            if !online.is_null() {
                *online = online_count;
            }
            if !in_game.is_null() {
                *in_game = in_game_count;
            }
            if !chatting.is_null() {
                *chatting = chatting_count;
            }
            true
        }
        None => false,
    }
}

unsafe extern "C" fn download_clan_activity_counts(
    this: *mut ISteamFriends,
    steamid_clans: *mut u64,
    clans_to_request: i32,
) -> u64 {
    oracle_core::callbacks::generate_api_call_handle()
}

unsafe extern "C" fn get_friend_count_from_source(
    this: *mut ISteamFriends,
    steamid_source: u64,
) -> i32 {
    let friends = &*this;
    friends
        .manager
        .read()
        .get_friend_count_from_source(steamid_source)
}

unsafe extern "C" fn get_friend_from_source_by_index(
    this: *mut ISteamFriends,
    steamid_source: u64,
    index: i32,
) -> u64 {
    let friends = &*this;
    friends
        .manager
        .read()
        .get_friend_from_source_by_index(steamid_source, index)
}

unsafe extern "C" fn is_user_in_source(
    this: *mut ISteamFriends,
    steamid_user: u64,
    steamid_source: u64,
) -> bool {
    let friends = &*this;
    friends
        .manager
        .read()
        .is_user_in_source(steamid_user, steamid_source)
}

unsafe extern "C" fn set_in_game_voice_speaking(
    this: *mut ISteamFriends,
    steamid: u64,
    speaking: bool,
) {
    let friends = &*this;
    friends
        .manager
        .write()
        .set_in_game_voice_speaking(steamid, speaking);
}

unsafe extern "C" fn activate_game_overlay(this: *mut ISteamFriends, dialog: *const i8) {
    let dialog_str = if dialog.is_null() {
        String::new()
    } else {
        CStr::from_ptr(dialog).to_string_lossy().into_owned()
    };

    oracle_overlay::activate_overlay(&dialog_str);
}

unsafe extern "C" fn activate_game_overlay_to_user(
    this: *mut ISteamFriends,
    dialog: *const i8,
    steamid: u64,
) {
    let dialog_str = if dialog.is_null() {
        String::new()
    } else {
        CStr::from_ptr(dialog).to_string_lossy().into_owned()
    };

    oracle_overlay::activate_overlay_to_user(&dialog_str, steamid);
}

unsafe extern "C" fn activate_game_overlay_to_web_page(
    this: *mut ISteamFriends,
    url: *const i8,
    mode: i32,
) {
    let url_str = if url.is_null() {
        String::new()
    } else {
        CStr::from_ptr(url).to_string_lossy().into_owned()
    };

    oracle_overlay::activate_overlay_to_web_page(&url_str, mode);
}

unsafe extern "C" fn activate_game_overlay_to_store(
    this: *mut ISteamFriends,
    app_id: u32,
    flag: i32,
) {
    oracle_overlay::activate_overlay_to_store(app_id, flag);
}

unsafe extern "C" fn set_play_group(this: *mut ISteamFriends, steamid_play_group: u64) {
    let friends = &*this;
    friends.manager.write().set_play_group(steamid_play_group);
}

unsafe extern "C" fn leave_play_group(this: *mut ISteamFriends) {
    let friends = &*this;
    friends.manager.write().leave_play_group();
}

unsafe extern "C" fn get_play_group_count(this: *mut ISteamFriends) -> i32 {
    let friends = &*this;
    friends.manager.read().get_play_group_count()
}

unsafe extern "C" fn get_play_group_by_index(this: *mut ISteamFriends, index: i32) -> u64 {
    let friends = &*this;
    friends.manager.read().get_play_group_by_index(index)
}

unsafe extern "C" fn activate_game_overlay_invite_dialog(
    this: *mut ISteamFriends,
    steamid_lobby: u64,
) {
    oracle_overlay::activate_overlay_invite_dialog(steamid_lobby);
}

unsafe extern "C" fn get_small_friend_avatar(this: *mut ISteamFriends, steamid: u64) -> i32 {
    let friends = &*this;
    friends.manager.read().get_small_friend_avatar(steamid)
}

unsafe extern "C" fn get_medium_friend_avatar(this: *mut ISteamFriends, steamid: u64) -> i32 {
    let friends = &*this;
    friends.manager.read().get_medium_friend_avatar(steamid)
}

unsafe extern "C" fn get_large_friend_avatar(this: *mut ISteamFriends, steamid: u64) -> i32 {
    let friends = &*this;
    friends.manager.read().get_large_friend_avatar(steamid)
}

unsafe extern "C" fn request_user_information(
    this: *mut ISteamFriends,
    steamid: u64,
    require_name_only: bool,
) -> bool {
    let friends = &*this;
    friends
        .manager
        .write()
        .request_user_information(steamid, require_name_only)
}

unsafe extern "C" fn request_clan_officer_list(this: *mut ISteamFriends, steamid_clan: u64) -> u64 {
    oracle_core::callbacks::generate_api_call_handle()
}

unsafe extern "C" fn get_clan_owner(this: *mut ISteamFriends, steamid_clan: u64) -> u64 {
    let friends = &*this;
    friends.manager.read().get_clan_owner(steamid_clan)
}

unsafe extern "C" fn get_clan_officer_count(this: *mut ISteamFriends, steamid_clan: u64) -> i32 {
    let friends = &*this;
    friends.manager.read().get_clan_officer_count(steamid_clan)
}

unsafe extern "C" fn get_clan_officer_by_index(
    this: *mut ISteamFriends,
    steamid_clan: u64,
    index: i32,
) -> u64 {
    let friends = &*this;
    friends
        .manager
        .read()
        .get_clan_officer_by_index(steamid_clan, index)
}

unsafe extern "C" fn get_user_restriction(_this: *mut ISteamFriends) -> u32 {
    0
}

unsafe extern "C" fn set_rich_presence(
    this: *mut ISteamFriends,
    key: *const i8,
    value: *const i8,
) -> bool {
    let friends = &*this;
    let mut manager = friends.manager.write();

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

    manager.set_rich_presence(key_str, value_str)
}

unsafe extern "C" fn clear_rich_presence(this: *mut ISteamFriends) {
    let friends = &*this;
    friends.manager.write().clear_rich_presence();
}

unsafe extern "C" fn get_friend_rich_presence(
    this: *mut ISteamFriends,
    steamid: u64,
    key: *const i8,
) -> *const i8 {
    let friends = &*this;
    let manager = friends.manager.read();

    let key_str = if key.is_null() {
        return std::ptr::null();
    } else {
        CStr::from_ptr(key).to_string_lossy().into_owned()
    };

    manager.get_friend_rich_presence_ptr(steamid, &key_str)
}

unsafe extern "C" fn get_friend_rich_presence_key_count(
    this: *mut ISteamFriends,
    steamid: u64,
) -> i32 {
    let friends = &*this;
    friends
        .manager
        .read()
        .get_friend_rich_presence_key_count(steamid)
}

unsafe extern "C" fn get_friend_rich_presence_key_by_index(
    this: *mut ISteamFriends,
    steamid: u64,
    index: i32,
) -> *const i8 {
    let friends = &*this;
    friends
        .manager
        .read()
        .get_friend_rich_presence_key_by_index_ptr(steamid, index)
}

unsafe extern "C" fn request_friend_rich_presence(this: *mut ISteamFriends, steamid: u64) {
    let friends = &*this;
    friends
        .manager
        .write()
        .request_friend_rich_presence(steamid);
}

unsafe extern "C" fn invite_user_to_game(
    this: *mut ISteamFriends,
    steamid: u64,
    connect_string: *const i8,
) -> bool {
    let connect_str = if connect_string.is_null() {
        String::new()
    } else {
        CStr::from_ptr(connect_string)
            .to_string_lossy()
            .into_owned()
    };

    oracle_networking::invite_user_to_game(steamid, &connect_str)
}

unsafe extern "C" fn get_coplay_friend_count(this: *mut ISteamFriends) -> i32 {
    let friends = &*this;
    friends.manager.read().get_coplay_friend_count()
}

unsafe extern "C" fn get_coplay_friend(this: *mut ISteamFriends, index: i32) -> u64 {
    let friends = &*this;
    friends.manager.read().get_coplay_friend(index)
}

unsafe extern "C" fn get_friend_coplay_time(this: *mut ISteamFriends, steamid: u64) -> i32 {
    let friends = &*this;
    friends.manager.read().get_friend_coplay_time(steamid)
}

unsafe extern "C" fn get_friend_coplay_game(this: *mut ISteamFriends, steamid: u64) -> u32 {
    let friends = &*this;
    friends.manager.read().get_friend_coplay_game(steamid)
}
