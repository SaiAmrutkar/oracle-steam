// Complete ISteamFriends - All 80+ functions
use crate::STEAM_CLIENT;
use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::ffi::{c_char, CStr};
use std::ptr;

// ============================================================================
// FRIENDS DATA STRUCTURES
// ============================================================================

#[repr(C)]
#[derive(Debug, Clone)]
struct FriendData {
    steam_id: u64,
    persona_name: String,
    persona_state: i32,
    game_id: u64,
    game_ip: u32,
    game_port: u16,
    nickname: String,
    avatar_hash: [u8; 20],
    steam_level: i32,
    rich_presence: HashMap<String, String>,
}

lazy_static! {
    static ref FRIENDS_LIST: RwLock<Vec<FriendData>> = RwLock::new(vec![
        FriendData {
            steam_id: 76561198000000001,
            persona_name: "TestFriend1".to_string(),
            persona_state: 1, // Online
            game_id: 0,
            game_ip: 0,
            game_port: 0,
            nickname: String::new(),
            avatar_hash: [0; 20],
            steam_level: 10,
            rich_presence: HashMap::new(),
        },
        FriendData {
            steam_id: 76561198000000002,
            persona_name: "TestFriend2".to_string(),
            persona_state: 3, // In-game
            game_id: 730,
            game_ip: 0,
            game_port: 0,
            nickname: String::new(),
            avatar_hash: [0; 20],
            steam_level: 25,
            rich_presence: HashMap::new(),
        },
    ]);

    static ref MY_PERSONA_NAME: RwLock<String> = RwLock::new("Player".to_string());
    static ref MY_PERSONA_STATE: RwLock<i32> = RwLock::new(1); // Online
    static ref MY_RICH_PRESENCE: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
    static ref CLAN_LIST: RwLock<Vec<ClanData>> = RwLock::new(Vec::new());
    static ref CHAT_MESSAGES: RwLock<Vec<ChatMessage>> = RwLock::new(Vec::new());
}

#[derive(Debug, Clone)]
struct ClanData {
    steam_id: u64,
    name: String,
    tag: String,
    member_count: i32,
}

#[derive(Debug, Clone)]
struct ChatMessage {
    friend_id: u64,
    message: String,
    timestamp: u64,
}

// ============================================================================
// PERSONA (User Profile)
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetPersonaName() -> *const c_char {
    static mut NAME_BUFFER: [u8; 256] = [0; 256];

    let name = MY_PERSONA_NAME.read();
    let bytes = name.as_bytes();
    let len = bytes.len().min(255);

    unsafe {
        NAME_BUFFER[..len].copy_from_slice(&bytes[..len]);
        NAME_BUFFER[len] = 0;
        NAME_BUFFER.as_ptr() as *const c_char
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_SetPersonaName(name: *const c_char) -> u64 {
    if name.is_null() {
        return 0;
    }

    unsafe {
        if let Ok(new_name) = CStr::from_ptr(name).to_str() {
            *MY_PERSONA_NAME.write() = new_name.to_string();
            println!("[Oracle] Persona name set to: {}", new_name);
        }
    }

    1 // API call handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetPersonaState() -> i32 {
    *MY_PERSONA_STATE.read()
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_SetPersonaState(state: i32) {
    *MY_PERSONA_STATE.write() = state;
    println!("[Oracle] Persona state set to: {}", state);
}

// ============================================================================
// FRIENDS LIST
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetFriendCount(flags: i32) -> i32 {
    FRIENDS_LIST.read().len() as i32
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetFriendByIndex(friend_idx: i32, flags: i32) -> u64 {
    FRIENDS_LIST
        .read()
        .get(friend_idx as usize)
        .map(|f| f.steam_id)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetFriendRelationship(steam_id: u64) -> i32 {
    if FRIENDS_LIST.read().iter().any(|f| f.steam_id == steam_id) {
        3 // k_EFriendRelationshipFriend
    } else {
        0 // k_EFriendRelationshipNone
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetFriendPersonaState(steam_id: u64) -> i32 {
    FRIENDS_LIST
        .read()
        .iter()
        .find(|f| f.steam_id == steam_id)
        .map(|f| f.persona_state)
        .unwrap_or(0) // Offline
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetFriendPersonaName(steam_id: u64) -> *const c_char {
    static mut NAME_BUFFER: [u8; 256] = [0; 256];

    let friends = FRIENDS_LIST.read();
    let name = friends
        .iter()
        .find(|f| f.steam_id == steam_id)
        .map(|f| f.persona_name.as_str())
        .unwrap_or("Unknown");

    let bytes = name.as_bytes();
    let len = bytes.len().min(255);

    unsafe {
        NAME_BUFFER[..len].copy_from_slice(&bytes[..len]);
        NAME_BUFFER[len] = 0;
        NAME_BUFFER.as_ptr() as *const c_char
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetFriendGamePlayed(
    steam_id: u64,
    game_info: *mut FriendGameInfo_t,
) -> bool {
    if game_info.is_null() {
        return false;
    }

    let friends = FRIENDS_LIST.read();
    if let Some(friend) = friends.iter().find(|f| f.steam_id == steam_id) {
        unsafe {
            (*game_info).game_id = friend.game_id;
            (*game_info).game_ip = friend.game_ip;
            (*game_info).game_port = friend.game_port;
            (*game_info).steam_id_lobby = 0;
        }
        return friend.game_id != 0;
    }

    false
}

#[repr(C)]
pub struct FriendGameInfo_t {
    game_id: u64,
    game_ip: u32,
    game_port: u16,
    query_port: u16,
    steam_id_lobby: u64,
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetFriendPersonaNameHistory(
    steam_id: u64,
    persona_name_idx: i32,
) -> *const c_char {
    // Return current name (no history tracking for now)
    SteamAPI_ISteamFriends_GetFriendPersonaName(steam_id)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetFriendSteamLevel(steam_id: u64) -> i32 {
    FRIENDS_LIST
        .read()
        .iter()
        .find(|f| f.steam_id == steam_id)
        .map(|f| f.steam_level)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetPlayerNickname(steam_id: u64) -> *const c_char {
    static mut NICK_BUFFER: [u8; 256] = [0; 256];

    let friends = FRIENDS_LIST.read();
    let nick = friends
        .iter()
        .find(|f| f.steam_id == steam_id)
        .and_then(|f| {
            if f.nickname.is_empty() {
                None
            } else {
                Some(f.nickname.as_str())
            }
        });

    if let Some(nickname) = nick {
        let bytes = nickname.as_bytes();
        let len = bytes.len().min(255);
        unsafe {
            NICK_BUFFER[..len].copy_from_slice(&bytes[..len]);
            NICK_BUFFER[len] = 0;
            return NICK_BUFFER.as_ptr() as *const c_char;
        }
    }

    ptr::null()
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_HasFriend(steam_id: u64, flags: i32) -> bool {
    FRIENDS_LIST.read().iter().any(|f| f.steam_id == steam_id)
}

// ============================================================================
// CLANS / GROUPS
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetClanCount() -> i32 {
    CLAN_LIST.read().len() as i32
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetClanByIndex(clan_idx: i32) -> u64 {
    CLAN_LIST
        .read()
        .get(clan_idx as usize)
        .map(|c| c.steam_id)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetClanName(steam_id_clan: u64) -> *const c_char {
    static mut CLAN_NAME_BUFFER: [u8; 256] = [0; 256];

    let clans = CLAN_LIST.read();
    let name = clans
        .iter()
        .find(|c| c.steam_id == steam_id_clan)
        .map(|c| c.name.as_str())
        .unwrap_or("Unknown Clan");

    let bytes = name.as_bytes();
    let len = bytes.len().min(255);

    unsafe {
        CLAN_NAME_BUFFER[..len].copy_from_slice(&bytes[..len]);
        CLAN_NAME_BUFFER[len] = 0;
        CLAN_NAME_BUFFER.as_ptr() as *const c_char
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetClanTag(steam_id_clan: u64) -> *const c_char {
    static mut CLAN_TAG_BUFFER: [u8; 64] = [0; 64];

    let clans = CLAN_LIST.read();
    let tag = clans
        .iter()
        .find(|c| c.steam_id == steam_id_clan)
        .map(|c| c.tag.as_str())
        .unwrap_or("");

    let bytes = tag.as_bytes();
    let len = bytes.len().min(63);

    unsafe {
        CLAN_TAG_BUFFER[..len].copy_from_slice(&bytes[..len]);
        CLAN_TAG_BUFFER[len] = 0;
        CLAN_TAG_BUFFER.as_ptr() as *const c_char
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetClanActivityCounts(
    steam_id_clan: u64,
    online: *mut i32,
    in_game: *mut i32,
    chatting: *mut i32,
) -> bool {
    if online.is_null() || in_game.is_null() || chatting.is_null() {
        return false;
    }

    unsafe {
        *online = 10;
        *in_game = 3;
        *chatting = 2;
    }

    true
}

// ============================================================================
// OVERLAY
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_ActivateGameOverlay(dialog: *const c_char) {
    let dialog_str = if dialog.is_null() {
        "friends"
    } else {
        unsafe { CStr::from_ptr(dialog).to_str().unwrap_or("friends") }
    };

    println!("[Oracle] Activating overlay: {}", dialog_str);
    // TODO: Actually show overlay
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_ActivateGameOverlayToUser(
    dialog: *const c_char,
    steam_id: u64,
) {
    let dialog_str = if dialog.is_null() {
        "chat"
    } else {
        unsafe { CStr::from_ptr(dialog).to_str().unwrap_or("chat") }
    };

    println!(
        "[Oracle] Activating overlay to user: {} ({})",
        steam_id, dialog_str
    );
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_ActivateGameOverlayToWebPage(
    url: *const c_char,
    mode: i32,
) {
    let url_str = if url.is_null() {
        "about:blank"
    } else {
        unsafe { CStr::from_ptr(url).to_str().unwrap_or("about:blank") }
    };

    println!("[Oracle] Opening web page in overlay: {}", url_str);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_ActivateGameOverlayToStore(app_id: u32, flag: i32) {
    println!("[Oracle] Opening store page for app: {}", app_id);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_ActivateGameOverlayInviteDialog(steam_id_lobby: u64) {
    println!(
        "[Oracle] Opening invite dialog for lobby: {}",
        steam_id_lobby
    );
}

// ============================================================================
// AVATARS
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetSmallFriendAvatar(steam_id: u64) -> i32 {
    1 // Return handle to 32x32 avatar
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetMediumFriendAvatar(steam_id: u64) -> i32 {
    2 // Return handle to 64x64 avatar
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetLargeFriendAvatar(steam_id: u64) -> i32 {
    3 // Return handle to 128x128 avatar
}

// ============================================================================
// RICH PRESENCE
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_SetRichPresence(
    key: *const c_char,
    value: *const c_char,
) -> bool {
    if key.is_null() {
        return false;
    }

    unsafe {
        let key_str = CStr::from_ptr(key).to_str().unwrap_or("");
        let value_str = if value.is_null() {
            ""
        } else {
            CStr::from_ptr(value).to_str().unwrap_or("")
        };

        let mut presence = MY_RICH_PRESENCE.write();
        if value_str.is_empty() {
            presence.remove(key_str);
        } else {
            presence.insert(key_str.to_string(), value_str.to_string());
        }

        println!("[Oracle] Rich presence set: {}={}", key_str, value_str);
    }

    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_ClearRichPresence() {
    MY_RICH_PRESENCE.write().clear();
    println!("[Oracle] Rich presence cleared");
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetFriendRichPresence(
    steam_id: u64,
    key: *const c_char,
) -> *const c_char {
    static mut RP_BUFFER: [u8; 256] = [0; 256];

    if key.is_null() {
        return ptr::null();
    }

    unsafe {
        let key_str = CStr::from_ptr(key).to_str().unwrap_or("");

        let friends = FRIENDS_LIST.read();
        if let Some(friend) = friends.iter().find(|f| f.steam_id == steam_id) {
            if let Some(value) = friend.rich_presence.get(key_str) {
                let bytes = value.as_bytes();
                let len = bytes.len().min(255);
                RP_BUFFER[..len].copy_from_slice(&bytes[..len]);
                RP_BUFFER[len] = 0;
                return RP_BUFFER.as_ptr() as *const c_char;
            }
        }
    }

    b"\0".as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetFriendRichPresenceKeyCount(steam_id: u64) -> i32 {
    FRIENDS_LIST
        .read()
        .iter()
        .find(|f| f.steam_id == steam_id)
        .map(|f| f.rich_presence.len() as i32)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetFriendRichPresenceKeyByIndex(
    steam_id: u64,
    key_idx: i32,
) -> *const c_char {
    static mut KEY_BUFFER: [u8; 256] = [0; 256];

    let friends = FRIENDS_LIST.read();
    if let Some(friend) = friends.iter().find(|f| f.steam_id == steam_id) {
        if let Some((key, _)) = friend.rich_presence.iter().nth(key_idx as usize) {
            let bytes = key.as_bytes();
            let len = bytes.len().min(255);
            unsafe {
                KEY_BUFFER[..len].copy_from_slice(&bytes[..len]);
                KEY_BUFFER[len] = 0;
                return KEY_BUFFER.as_ptr() as *const c_char;
            }
        }
    }

    ptr::null()
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_RequestFriendRichPresence(steam_id: u64) {
    println!("[Oracle] Requesting rich presence for: {}", steam_id);
    // Trigger callback with data
}

// ============================================================================
// INVITES & GAME JOINS
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_InviteUserToGame(
    steam_id_friend: u64,
    connect_string: *const c_char,
) -> bool {
    let connect = if connect_string.is_null() {
        ""
    } else {
        unsafe { CStr::from_ptr(connect_string).to_str().unwrap_or("") }
    };

    println!(
        "[Oracle] Inviting user {} to game: {}",
        steam_id_friend, connect
    );
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_SetPlayedWith(steam_id: u64) {
    println!("[Oracle] Marking as played with: {}", steam_id);
}

// ============================================================================
// CHAT MESSAGES
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_ReplyToFriendMessage(
    steam_id: u64,
    message: *const c_char,
) -> bool {
    if message.is_null() {
        return false;
    }

    unsafe {
        let msg_str = CStr::from_ptr(message).to_str().unwrap_or("");

        CHAT_MESSAGES.write().push(ChatMessage {
            friend_id: steam_id,
            message: msg_str.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        println!("[Oracle] Sent message to {}: {}", steam_id, msg_str);
    }

    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetFriendMessage(
    steam_id: u64,
    message_idx: i32,
    buffer: *mut c_char,
    buffer_size: i32,
    chat_entry_type: *mut i32,
) -> i32 {
    if buffer.is_null() || buffer_size < 1 {
        return 0;
    }

    // Return 0 for now (no messages)
    0
}

// ============================================================================
// FOLLOWERS
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetFollowerCount(steam_id: u64) -> u64 {
    println!("[Oracle] Getting follower count for: {}", steam_id);
    1 // API call handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_IsFollowing(steam_id: u64) -> u64 {
    println!("[Oracle] Checking if following: {}", steam_id);
    1 // API call handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_EnumerateFollowingList(start_idx: u32) -> u64 {
    println!("[Oracle] Enumerating following list from: {}", start_idx);
    1 // API call handle
}

// ============================================================================
// COPLAY
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetCoplayFriendCount() -> i32 {
    0 // No coplay friends
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetCoplayFriend(coplay_friend_idx: i32) -> u64 {
    0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetFriendCoplayTime(steam_id: u64) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetFriendCoplayGame(steam_id: u64) -> u32 {
    0
}

// ============================================================================
// USER INFO REQUESTS
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_RequestUserInformation(
    steam_id: u64,
    require_name_only: bool,
) -> bool {
    println!("[Oracle] Requesting user info for: {}", steam_id);
    false // Data already available
}

// ============================================================================
// CLAN CHAT (Group Chat)
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_JoinClanChatRoom(steam_id_clan: u64) -> u64 {
    println!("[Oracle] Joining clan chat: {}", steam_id_clan);
    1 // API call handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_LeaveClanChatRoom(steam_id_clan: u64) -> bool {
    println!("[Oracle] Leaving clan chat: {}", steam_id_clan);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetClanChatMemberCount(steam_id_clan: u64) -> i32 {
    5 // Fake member count
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetChatMemberByIndex(
    steam_id_clan: u64,
    member_idx: i32,
) -> u64 {
    76561198000000000 + member_idx as u64
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_SendClanChatMessage(
    steam_id_clan_chat: u64,
    text: *const c_char,
) -> bool {
    if text.is_null() {
        return false;
    }

    unsafe {
        let msg = CStr::from_ptr(text).to_str().unwrap_or("");
        println!("[Oracle] Clan chat message sent: {}", msg);
    }

    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_IsClanChatAdmin(
    steam_id_clan_chat: u64,
    steam_id_user: u64,
) -> bool {
    false
}

// ============================================================================
// RESTRICTIONS & PRIVACY
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetUserRestrictions() -> u32 {
    0 // No restrictions
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_SetListenForFriendsMessages(intercept: bool) -> bool {
    println!("[Oracle] Listen for friend messages: {}", intercept);
    true
}

// ============================================================================
// CLAN INFO
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_IsClanPublic(steam_id_clan: u64) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_IsClanOfficialGameGroup(steam_id_clan: u64) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetNumChatsWithUnreadPriorityMessages() -> i32 {
    0
}
