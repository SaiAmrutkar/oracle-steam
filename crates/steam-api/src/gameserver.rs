// crates/steam-api/src/gameserver.rs
// Complete ISteamGameServer + ISteamGameServerStats implementation

use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr};
use std::net::Ipv4Addr;

lazy_static! {
    static ref SERVER_STATE: RwLock<GameServerState> = RwLock::new(GameServerState::new());
    static ref SERVER_RULES: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
    static ref PLAYER_STATS: RwLock<HashMap<u64, PlayerServerStats>> = RwLock::new(HashMap::new());
}

struct GameServerState {
    steam_id: u64,
    logged_on: bool,
    secure: bool,
    dedicated: bool,
    product: String,
    game_description: String,
    server_name: String,
    map_name: String,
    spectator_port: u16,
    spectator_server_name: String,
    max_players: i32,
    bot_count: i32,
    password_protected: bool,
    game_tags: String,
    game_data: String,
    region: i32,
    public_ip: u32,
    query_port: u16,
    game_port: u16,
}

impl GameServerState {
    fn new() -> Self {
        Self {
            steam_id: 90071992547409920, // Fake game server SteamID
            logged_on: false,
            secure: false,
            dedicated: true,
            product: "oracle_steam".to_string(),
            game_description: "Oracle Steam Game Server".to_string(),
            server_name: "Oracle Server".to_string(),
            map_name: "default".to_string(),
            spectator_port: 0,
            spectator_server_name: String::new(),
            max_players: 32,
            bot_count: 0,
            password_protected: false,
            game_tags: String::new(),
            game_data: String::new(),
            region: 255, // World
            public_ip: 0,
            query_port: 27015,
            game_port: 27015,
        }
    }
}

struct PlayerServerStats {
    steam_id: u64,
    stats: HashMap<String, i32>,
    achievements: Vec<String>,
}

// ============================================================================
// GAMESERVER INIT & AUTH
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_InitGameServer(
    ip: u32,
    game_port: u16,
    query_port: u16,
    flags: u32,
    app_id: u32,
    version: *const c_char,
) -> bool {
    let mut state = SERVER_STATE.write();

    state.public_ip = ip;
    state.game_port = game_port;
    state.query_port = query_port;
    state.logged_on = true;

    let version_str = if version.is_null() {
        "1.0.0.0"
    } else {
        unsafe { CStr::from_ptr(version).to_str().unwrap_or("1.0.0.0") }
    };

    println!(
        "[GameServer] Initialized - IP: {}, Port: {}, App: {}, Version: {}",
        Ipv4Addr::from(ip.to_be()),
        game_port,
        app_id,
        version_str
    );

    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_SetProduct(product: *const c_char) {
    if product.is_null() {
        return;
    }

    unsafe {
        if let Ok(prod) = CStr::from_ptr(product).to_str() {
            SERVER_STATE.write().product = prod.to_string();
        }
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_SetGameDescription(desc: *const c_char) {
    if desc.is_null() {
        return;
    }

    unsafe {
        if let Ok(description) = CStr::from_ptr(desc).to_str() {
            SERVER_STATE.write().game_description = description.to_string();
        }
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_SetModDir(mod_dir: *const c_char) {
    // Store mod directory if needed
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_SetDedicatedServer(dedicated: bool) {
    SERVER_STATE.write().dedicated = dedicated;
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_LogOn(token: *const c_char) {
    SERVER_STATE.write().logged_on = true;
    println!("[GameServer] Logged on to Steam");
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_LogOnAnonymous() {
    SERVER_STATE.write().logged_on = true;
    println!("[GameServer] Logged on anonymously");
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_LogOff() {
    SERVER_STATE.write().logged_on = false;
    println!("[GameServer] Logged off");
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_BLoggedOn() -> bool {
    SERVER_STATE.read().logged_on
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_BSecure() -> bool {
    SERVER_STATE.read().secure
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_GetSteamID() -> u64 {
    SERVER_STATE.read().steam_id
}

// ============================================================================
// SERVER INFO
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_SetServerName(name: *const c_char) {
    if name.is_null() {
        return;
    }

    unsafe {
        if let Ok(server_name) = CStr::from_ptr(name).to_str() {
            SERVER_STATE.write().server_name = server_name.to_string();
        }
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_SetMapName(map: *const c_char) {
    if map.is_null() {
        return;
    }

    unsafe {
        if let Ok(map_name) = CStr::from_ptr(map).to_str() {
            SERVER_STATE.write().map_name = map_name.to_string();
        }
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_SetMaxPlayerCount(count: i32) {
    SERVER_STATE.write().max_players = count;
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_SetBotPlayerCount(count: i32) {
    SERVER_STATE.write().bot_count = count;
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_SetPasswordProtected(password: bool) {
    SERVER_STATE.write().password_protected = password;
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_SetSpectatorPort(port: u16) {
    SERVER_STATE.write().spectator_port = port;
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_SetSpectatorServerName(name: *const c_char) {
    if name.is_null() {
        return;
    }

    unsafe {
        if let Ok(spec_name) = CStr::from_ptr(name).to_str() {
            SERVER_STATE.write().spectator_server_name = spec_name.to_string();
        }
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_SetGameTags(tags: *const c_char) {
    if tags.is_null() {
        return;
    }

    unsafe {
        if let Ok(game_tags) = CStr::from_ptr(tags).to_str() {
            SERVER_STATE.write().game_tags = game_tags.to_string();
        }
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_SetGameData(data: *const c_char) {
    if data.is_null() {
        return;
    }

    unsafe {
        if let Ok(game_data) = CStr::from_ptr(data).to_str() {
            SERVER_STATE.write().game_data = game_data.to_string();
        }
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_SetRegion(region: *const c_char) {
    // Store region code
}

// ============================================================================
// SERVER RULES
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_SetKeyValue(key: *const c_char, value: *const c_char) {
    if key.is_null() || value.is_null() {
        return;
    }

    unsafe {
        if let (Ok(k), Ok(v)) = (CStr::from_ptr(key).to_str(), CStr::from_ptr(value).to_str()) {
            SERVER_RULES.write().insert(k.to_string(), v.to_string());
        }
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_ClearAllKeyValues() {
    SERVER_RULES.write().clear();
}

// ============================================================================
// PLAYER MANAGEMENT
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_SendUserConnectAndAuthenticate(
    ip: u32,
    auth_blob: *const c_void,
    auth_blob_size: u32,
    steam_id_user: *mut u64,
) -> bool {
    if steam_id_user.is_null() {
        return false;
    }

    unsafe {
        *steam_id_user = 76561198000000000 + rand::random::<u64>() % 1000000;
    }

    println!(
        "[GameServer] Player authenticated: IP={}",
        Ipv4Addr::from(ip.to_be())
    );
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_CreateUnauthenticatedUserConnection() -> u64 {
    76561198000000000 + rand::random::<u64>() % 1000000
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_SendUserDisconnect(steam_id: u64) {
    PLAYER_STATS.write().remove(&steam_id);
    println!("[GameServer] Player disconnected: {}", steam_id);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_BUpdateUserData(
    steam_id: u64,
    player_name: *const c_char,
    score: u32,
) -> bool {
    true
}

// ============================================================================
// HEARTBEAT & MASTER SERVER
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_EnableHeartbeats(active: bool) {
    println!(
        "[GameServer] Heartbeats: {}",
        if active { "enabled" } else { "disabled" }
    );
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_SetHeartbeatInterval(interval: i32) {
    println!("[GameServer] Heartbeat interval: {}s", interval);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_ForceHeartbeat() {
    println!("[GameServer] Forced heartbeat sent");
}

// ============================================================================
// AUTH TICKETS
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_GetAuthSessionTicket(
    ticket: *mut c_void,
    max_ticket: i32,
    ticket_size: *mut u32,
) -> u32 {
    if ticket.is_null() || ticket_size.is_null() {
        return 0;
    }

    // Generate fake server ticket
    let ticket_data = b"ORACLE_GAMESERVER_TICKET";
    let size = ticket_data.len().min(max_ticket as usize);

    unsafe {
        std::ptr::copy_nonoverlapping(ticket_data.as_ptr(), ticket as *mut u8, size);
        *ticket_size = size as u32;
    }

    1 // Ticket handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_BeginAuthSession(
    auth_ticket: *const c_void,
    auth_ticket_size: i32,
    steam_id: u64,
) -> i32 {
    0 // k_EBeginAuthSessionResultOK
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_EndAuthSession(steam_id: u64) {
    println!("[GameServer] Auth session ended: {}", steam_id);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServer_CancelAuthTicket(ticket: u32) {
    println!("[GameServer] Auth ticket cancelled: {}", ticket);
}

// ============================================================================
// GAMESERVERSTATS
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServerStats_RequestUserStats(steam_id: u64) -> u64 {
    println!("[GameServerStats] Requesting stats for: {}", steam_id);
    1 // API call handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServerStats_GetUserStat_Int(
    steam_id: u64,
    name: *const c_char,
    data: *mut i32,
) -> bool {
    if name.is_null() || data.is_null() {
        return false;
    }

    unsafe {
        if let Ok(stat_name) = CStr::from_ptr(name).to_str() {
            let stats = PLAYER_STATS.read();
            if let Some(player) = stats.get(&steam_id) {
                if let Some(&value) = player.stats.get(stat_name) {
                    *data = value;
                    return true;
                }
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServerStats_SetUserStat_Int(
    steam_id: u64,
    name: *const c_char,
    data: i32,
) -> bool {
    if name.is_null() {
        return false;
    }

    unsafe {
        if let Ok(stat_name) = CStr::from_ptr(name).to_str() {
            let mut stats = PLAYER_STATS.write();
            let player = stats.entry(steam_id).or_insert_with(|| PlayerServerStats {
                steam_id,
                stats: HashMap::new(),
                achievements: Vec::new(),
            });
            player.stats.insert(stat_name.to_string(), data);
            return true;
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServerStats_UpdateUserAvgRateStat(
    steam_id: u64,
    name: *const c_char,
    count: f32,
    session_length: f64,
) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServerStats_GetUserAchievement(
    steam_id: u64,
    name: *const c_char,
    achieved: *mut bool,
) -> bool {
    if name.is_null() || achieved.is_null() {
        return false;
    }

    unsafe {
        if let Ok(ach_name) = CStr::from_ptr(name).to_str() {
            let stats = PLAYER_STATS.read();
            if let Some(player) = stats.get(&steam_id) {
                *achieved = player.achievements.contains(&ach_name.to_string());
                return true;
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServerStats_SetUserAchievement(
    steam_id: u64,
    name: *const c_char,
) -> bool {
    if name.is_null() {
        return false;
    }

    unsafe {
        if let Ok(ach_name) = CStr::from_ptr(name).to_str() {
            let mut stats = PLAYER_STATS.write();
            let player = stats.entry(steam_id).or_insert_with(|| PlayerServerStats {
                steam_id,
                stats: HashMap::new(),
                achievements: Vec::new(),
            });

            if !player.achievements.contains(&ach_name.to_string()) {
                player.achievements.push(ach_name.to_string());
            }
            return true;
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServerStats_ClearUserAchievement(
    steam_id: u64,
    name: *const c_char,
) -> bool {
    if name.is_null() {
        return false;
    }

    unsafe {
        if let Ok(ach_name) = CStr::from_ptr(name).to_str() {
            let mut stats = PLAYER_STATS.write();
            if let Some(player) = stats.get_mut(&steam_id) {
                player.achievements.retain(|a| a != ach_name);
                return true;
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamGameServerStats_StoreUserStats(steam_id: u64) -> u64 {
    println!("[GameServerStats] Storing stats for: {}", steam_id);
    1 // API call handle
}
