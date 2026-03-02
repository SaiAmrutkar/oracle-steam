use anyhow::{bail, Result};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::ffi::CString;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct FavoriteGame {
    pub app_id: u32,
    pub ip: u32,
    pub conn_port: u16,
    pub query_port: u16,
    pub flags: u32,
    pub last_played: u32,
}

#[derive(Debug, Clone)]
pub struct Lobby {
    pub steamid_lobby: u64,
    pub owner: u64,
    pub lobby_type: i32,
    pub max_members: i32,
    pub joinable: bool,
    pub members: Vec<u64>,
    pub data: HashMap<String, String>,
    pub member_data: HashMap<u64, HashMap<String, String>>,
    pub chat_messages: Vec<ChatEntry>,
    pub game_server: Option<GameServerInfo>,
    pub linked_lobby: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct GameServerInfo {
    pub ip: u32,
    pub port: u16,
    pub steamid: u64,
}

#[derive(Debug, Clone)]
pub struct ChatEntry {
    pub sender: u64,
    pub data: Vec<u8>,
    pub entry_type: i32,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
enum LobbyFilter {
    StringFilter {
        key: String,
        value: String,
        comparison: i32,
    },
    NumericalFilter {
        key: String,
        value: i32,
        comparison: i32,
    },
    NearValueFilter {
        key: String,
        value: i32,
    },
    SlotsAvailable(i32),
    Distance(i32),
    CompatibleMembers(u64),
}

pub struct MatchmakingManager {
    my_steamid: u64,
    favorite_games: Vec<FavoriteGame>,
    lobbies: HashMap<u64, Lobby>,
    my_lobbies: Vec<u64>,
    search_results: Vec<u64>,
    search_filters: Vec<LobbyFilter>,
    max_search_results: i32,
    next_lobby_id: u64,
}

impl MatchmakingManager {
    pub fn new(my_steamid: u64) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            my_steamid,
            favorite_games: Vec::new(),
            lobbies: HashMap::new(),
            my_lobbies: Vec::new(),
            search_results: Vec::new(),
            search_filters: Vec::new(),
            max_search_results: 50,
            next_lobby_id: 0x0110000100000000, // Start of lobby ID range
        }))
    }

    pub fn get_favorite_game_count(&self) -> i32 {
        self.favorite_games.len() as i32
    }

    pub fn get_favorite_game(&self, index: i32) -> Option<FavoriteGame> {
        self.favorite_games.get(index as usize).cloned()
    }

    pub fn add_favorite_game(
        &mut self,
        app_id: u32,
        ip: u32,
        conn_port: u16,
        query_port: u16,
        flags: u32,
        last_played: u32,
    ) -> i32 {
        let favorite = FavoriteGame {
            app_id,
            ip,
            conn_port,
            query_port,
            flags,
            last_played,
        };

        self.favorite_games.push(favorite);
        (self.favorite_games.len() - 1) as i32
    }

    pub fn remove_favorite_game(
        &mut self,
        app_id: u32,
        ip: u32,
        conn_port: u16,
        query_port: u16,
        _flags: u32,
    ) -> bool {
        let original_len = self.favorite_games.len();

        self.favorite_games.retain(|fav| {
            !(fav.app_id == app_id
                && fav.ip == ip
                && fav.conn_port == conn_port
                && fav.query_port == query_port)
        });

        self.favorite_games.len() < original_len
    }

    pub fn add_string_filter(&mut self, key: String, value: String, comparison: i32) {
        self.search_filters.push(LobbyFilter::StringFilter {
            key,
            value,
            comparison,
        });
    }

    pub fn add_numerical_filter(&mut self, key: String, value: i32, comparison: i32) {
        self.search_filters.push(LobbyFilter::NumericalFilter {
            key,
            value,
            comparison,
        });
    }

    pub fn add_near_value_filter(&mut self, key: String, value: i32) {
        self.search_filters
            .push(LobbyFilter::NearValueFilter { key, value });
    }

    pub fn add_slots_available_filter(&mut self, slots: i32) {
        self.search_filters.push(LobbyFilter::SlotsAvailable(slots));
    }

    pub fn add_distance_filter(&mut self, distance: i32) {
        self.search_filters.push(LobbyFilter::Distance(distance));
    }

    pub fn add_compatible_members_filter(&mut self, steamid_lobby: u64) {
        self.search_filters
            .push(LobbyFilter::CompatibleMembers(steamid_lobby));
    }

    pub fn set_result_count_filter(&mut self, max_results: i32) {
        self.max_search_results = max_results;
    }

    pub fn request_lobby_list(&mut self) -> Vec<u64> {
        // Query lobbies from network or local database
        let mut matching_lobbies = Vec::new();

        for (lobby_id, lobby) in &self.lobbies {
            if self.lobby_matches_filters(lobby) {
                matching_lobbies.push(*lobby_id);
            }
        }

        // Query from network relay servers
        if let Ok(network_lobbies) = self.query_network_lobbies() {
            for lobby in network_lobbies {
                if self.lobby_matches_filters(&lobby) {
                    let lobby_id = lobby.steamid_lobby;
                    self.lobbies.insert(lobby_id, lobby);
                    matching_lobbies.push(lobby_id);
                }
            }
        }

        // Apply max results filter
        matching_lobbies.truncate(self.max_search_results as usize);

        self.search_results = matching_lobbies.clone();
        self.search_filters.clear();

        matching_lobbies
    }

    pub fn get_lobby_by_index(&self, index: i32) -> u64 {
        self.search_results
            .get(index as usize)
            .copied()
            .unwrap_or(0)
    }

    pub fn create_lobby(&mut self, lobby_type: i32, max_members: i32) -> Result<u64> {
        let lobby_id = self.allocate_lobby_id();

        let lobby = Lobby {
            steamid_lobby: lobby_id,
            owner: self.my_steamid,
            lobby_type,
            max_members,
            joinable: true,
            members: vec![self.my_steamid],
            data: HashMap::new(),
            member_data: HashMap::new(),
            chat_messages: Vec::new(),
            game_server: None,
            linked_lobby: None,
        };

        self.lobbies.insert(lobby_id, lobby);
        self.my_lobbies.push(lobby_id);

        // Broadcast lobby creation to network
        oracle_networking::relay::broadcast_lobby_created(lobby_id, self.my_steamid)?;

        log::info!("Created lobby: {}", lobby_id);
        Ok(lobby_id)
    }

    pub fn join_lobby(&mut self, steamid_lobby: u64) -> Result<()> {
        // Check if lobby exists locally
        if !self.lobbies.contains_key(&steamid_lobby) {
            // Request from network
            if let Ok(lobby) = oracle_networking::relay::request_lobby_info(steamid_lobby) {
                self.lobbies.insert(steamid_lobby, lobby);
            } else {
                bail!("Lobby not found");
            }
        }

        let lobby = self
            .lobbies
            .get_mut(&steamid_lobby)
            .ok_or_else(|| anyhow::anyhow!("Lobby not found"))?;

        if !lobby.joinable {
            bail!("Lobby not joinable");
        }

        if lobby.members.len() >= lobby.max_members as usize {
            bail!("Lobby full");
        }

        if !lobby.members.contains(&self.my_steamid) {
            lobby.members.push(self.my_steamid);
            self.my_lobbies.push(steamid_lobby);

            // Notify network
            oracle_networking::relay::send_lobby_join(steamid_lobby, self.my_steamid)?;

            // Queue callback for other members
            oracle_core::callbacks::queue_callback(LobbyChatUpdate_t {
                steamid_lobby,
                steamid_user_changed: self.my_steamid,
                steamid_making_change: self.my_steamid,
                chat_member_state_change: 0x0001, // k_EChatMemberStateChangeEntered
            });
        }

        log::info!("Joined lobby: {}", steamid_lobby);
        Ok(())
    }

    pub fn leave_lobby(&mut self, steamid_lobby: u64) {
        if let Some(lobby) = self.lobbies.get_mut(&steamid_lobby) {
            lobby.members.retain(|&member| member != self.my_steamid);

            // Notify network
            let _ = oracle_networking::relay::send_lobby_leave(steamid_lobby, self.my_steamid);

            // Queue callback
            oracle_core::callbacks::queue_callback(LobbyChatUpdate_t {
                steamid_lobby,
                steamid_user_changed: self.my_steamid,
                steamid_making_change: self.my_steamid,
                chat_member_state_change: 0x0002, // k_EChatMemberStateChangeLeft
            });
        }

        self.my_lobbies.retain(|&id| id != steamid_lobby);
        log::info!("Left lobby: {}", steamid_lobby);
    }

    pub fn invite_user_to_lobby(&self, steamid_lobby: u64, steamid_invitee: u64) -> bool {
        if !self.lobbies.contains_key(&steamid_lobby) {
            return false;
        }

        // Send invitation through network
        match oracle_networking::relay::send_lobby_invite(
            steamid_lobby,
            steamid_invitee,
            self.my_steamid,
        ) {
            Ok(_) => {
                log::info!("Invited {} to lobby {}", steamid_invitee, steamid_lobby);
                true
            }
            Err(e) => {
                log::error!("Failed to send lobby invite: {}", e);
                false
            }
        }
    }

    pub fn get_num_lobby_members(&self, steamid_lobby: u64) -> i32 {
        self.lobbies
            .get(&steamid_lobby)
            .map(|lobby| lobby.members.len() as i32)
            .unwrap_or(0)
    }

    pub fn get_lobby_member_by_index(&self, steamid_lobby: u64, index: i32) -> u64 {
        self.lobbies
            .get(&steamid_lobby)
            .and_then(|lobby| lobby.members.get(index as usize))
            .copied()
            .unwrap_or(0)
    }

    pub fn get_lobby_data_ptr(&self, steamid_lobby: u64, key: &str) -> *const i8 {
        self.lobbies
            .get(&steamid_lobby)
            .and_then(|lobby| lobby.data.get(key))
            .and_then(|value| CString::new(value.clone()).ok())
            .map(|s| s.into_raw() as *const i8)
            .unwrap_or(std::ptr::null())
    }

    pub fn set_lobby_data(&mut self, steamid_lobby: u64, key: String, value: String) -> bool {
        if let Some(lobby) = self.lobbies.get_mut(&steamid_lobby) {
            if lobby.owner != self.my_steamid {
                return false;
            }

            lobby.data.insert(key.clone(), value.clone());

            // Broadcast to network
            let _ =
                oracle_networking::relay::broadcast_lobby_data_update(steamid_lobby, key, value);

            // Queue callback
            oracle_core::callbacks::queue_callback(LobbyDataUpdate_t {
                steamid_lobby,
                steamid_member: steamid_lobby,
                success: 1,
            });

            true
        } else {
            false
        }
    }

    pub fn get_lobby_data_count(&self, steamid_lobby: u64) -> i32 {
        self.lobbies
            .get(&steamid_lobby)
            .map(|lobby| lobby.data.len() as i32)
            .unwrap_or(0)
    }

    pub fn get_lobby_data_by_index(
        &self,
        steamid_lobby: u64,
        index: i32,
    ) -> Option<(String, String)> {
        self.lobbies.get(&steamid_lobby).and_then(|lobby| {
            lobby
                .data
                .iter()
                .nth(index as usize)
                .map(|(k, v)| (k.clone(), v.clone()))
        })
    }

    pub fn delete_lobby_data(&mut self, steamid_lobby: u64, key: String) -> bool {
        if let Some(lobby) = self.lobbies.get_mut(&steamid_lobby) {
            if lobby.owner != self.my_steamid {
                return false;
            }

            lobby.data.remove(&key).is_some()
        } else {
            false
        }
    }

    pub fn get_lobby_member_data_ptr(
        &self,
        steamid_lobby: u64,
        steamid_user: u64,
        key: &str,
    ) -> *const i8 {
        self.lobbies
            .get(&steamid_lobby)
            .and_then(|lobby| lobby.member_data.get(&steamid_user))
            .and_then(|data| data.get(key))
            .and_then(|value| CString::new(value.clone()).ok())
            .map(|s| s.into_raw() as *const i8)
            .unwrap_or(std::ptr::null())
    }

    pub fn set_lobby_member_data(&mut self, steamid_lobby: u64, key: String, value: String) {
        if let Some(lobby) = self.lobbies.get_mut(&steamid_lobby) {
            lobby
                .member_data
                .entry(self.my_steamid)
                .or_insert_with(HashMap::new)
                .insert(key.clone(), value.clone());

            // Broadcast to network
            let _ = oracle_networking::relay::broadcast_lobby_member_data_update(
                steamid_lobby,
                self.my_steamid,
                key,
                value,
            );

            // Queue callback
            oracle_core::callbacks::queue_callback(LobbyDataUpdate_t {
                steamid_lobby,
                steamid_member: self.my_steamid,
                success: 1,
            });
        }
    }

    pub fn send_lobby_chat_msg(&mut self, steamid_lobby: u64, data: Vec<u8>) -> bool {
        if let Some(lobby) = self.lobbies.get_mut(&steamid_lobby) {
            let entry = ChatEntry {
                sender: self.my_steamid,
                data: data.clone(),
                entry_type: 1, // k_EChatEntryTypeChatMsg
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };

            lobby.chat_messages.push(entry);

            // Broadcast to network
            let _ = oracle_networking::relay::broadcast_lobby_chat(
                steamid_lobby,
                self.my_steamid,
                data,
            );

            true
        } else {
            false
        }
    }

    pub fn get_lobby_chat_entry(&self, steamid_lobby: u64, chat_id: i32) -> Option<ChatEntry> {
        self.lobbies
            .get(&steamid_lobby)
            .and_then(|lobby| lobby.chat_messages.get(chat_id as usize))
            .cloned()
    }

    pub fn request_lobby_data(&self, steamid_lobby: u64) -> bool {
        // Request lobby data from network
        match oracle_networking::relay::request_lobby_data(steamid_lobby) {
            Ok(_) => {
                oracle_core::callbacks::queue_callback(LobbyDataUpdate_t {
                    steamid_lobby,
                    steamid_member: steamid_lobby,
                    success: 1,
                });
                true
            }
            Err(_) => false,
        }
    }

    pub fn set_lobby_game_server(
        &mut self,
        steamid_lobby: u64,
        ip: u32,
        port: u16,
        steamid_server: u64,
    ) {
        if let Some(lobby) = self.lobbies.get_mut(&steamid_lobby) {
            lobby.game_server = Some(GameServerInfo {
                ip,
                port,
                steamid: steamid_server,
            });

            // Broadcast to network
            let _ = oracle_networking::relay::broadcast_lobby_game_server(
                steamid_lobby,
                ip,
                port,
                steamid_server,
            );
        }
    }

    pub fn get_lobby_game_server(&self, steamid_lobby: u64) -> Option<(u32, u16, u64)> {
        self.lobbies
            .get(&steamid_lobby)
            .and_then(|lobby| lobby.game_server.as_ref())
            .map(|server| (server.ip, server.port, server.steamid))
    }

    pub fn set_lobby_member_limit(&mut self, steamid_lobby: u64, max_members: i32) -> bool {
        if let Some(lobby) = self.lobbies.get_mut(&steamid_lobby) {
            if lobby.owner != self.my_steamid {
                return false;
            }

            lobby.max_members = max_members;
            true
        } else {
            false
        }
    }

    pub fn get_lobby_member_limit(&self, steamid_lobby: u64) -> i32 {
        self.lobbies
            .get(&steamid_lobby)
            .map(|lobby| lobby.max_members)
            .unwrap_or(0)
    }

    pub fn set_lobby_type(&mut self, steamid_lobby: u64, lobby_type: i32) -> bool {
        if let Some(lobby) = self.lobbies.get_mut(&steamid_lobby) {
            if lobby.owner != self.my_steamid {
                return false;
            }

            lobby.lobby_type = lobby_type;
            true
        } else {
            false
        }
    }

    pub fn set_lobby_joinable(&mut self, steamid_lobby: u64, joinable: bool) -> bool {
        if let Some(lobby) = self.lobbies.get_mut(&steamid_lobby) {
            if lobby.owner != self.my_steamid {
                return false;
            }

            lobby.joinable = joinable;
            true
        } else {
            false
        }
    }

    pub fn get_lobby_owner(&self, steamid_lobby: u64) -> u64 {
        self.lobbies
            .get(&steamid_lobby)
            .map(|lobby| lobby.owner)
            .unwrap_or(0)
    }

    pub fn set_lobby_owner(&mut self, steamid_lobby: u64, new_owner: u64) -> bool {
        if let Some(lobby) = self.lobbies.get_mut(&steamid_lobby) {
            if lobby.owner != self.my_steamid {
                return false;
            }

            if !lobby.members.contains(&new_owner) {
                return false;
            }

            lobby.owner = new_owner;

            // Broadcast to network
            let _ =
                oracle_networking::relay::broadcast_lobby_owner_change(steamid_lobby, new_owner);

            true
        } else {
            false
        }
    }

    pub fn set_linked_lobby(&mut self, steamid_lobby: u64, steamid_linked: u64) -> bool {
        if let Some(lobby) = self.lobbies.get_mut(&steamid_lobby) {
            lobby.linked_lobby = Some(steamid_linked);
            true
        } else {
            false
        }
    }

    fn allocate_lobby_id(&mut self) -> u64 {
        let id = self.next_lobby_id;
        self.next_lobby_id += 1;
        id
    }

    fn lobby_matches_filters(&self, lobby: &Lobby) -> bool {
        for filter in &self.search_filters {
            match filter {
                LobbyFilter::StringFilter {
                    key,
                    value,
                    comparison,
                } => {
                    if let Some(lobby_value) = lobby.data.get(key) {
                        if !self.compare_strings(lobby_value, value, *comparison) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                LobbyFilter::NumericalFilter {
                    key,
                    value,
                    comparison,
                } => {
                    if let Some(lobby_value) = lobby.data.get(key) {
                        if let Ok(num) = lobby_value.parse::<i32>() {
                            if !self.compare_numbers(num, *value, *comparison) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                LobbyFilter::SlotsAvailable(slots) => {
                    let available = lobby.max_members - lobby.members.len() as i32;
                    if available < *slots {
                        return false;
                    }
                }
                LobbyFilter::Distance(_distance) => {
                    // Distance filtering would require geolocation
                    // For now, accept all
                }
                _ => {}
            }
        }

        true
    }

    fn compare_strings(&self, a: &str, b: &str, comparison: i32) -> bool {
        match comparison {
            0 => a == b, // k_ELobbyComparisonEqual
            1 => a != b, // k_ELobbyComparisonNotEqual
            2 => a > b,  // k_ELobbyComparisonGreaterThan
            3 => a >= b, // k_ELobbyComparisonGreaterThanOrEqual
            4 => a < b,  // k_ELobbyComparisonLessThan
            5 => a <= b, // k_ELobbyComparisonLessThanOrEqual
            _ => false,
        }
    }

    fn compare_numbers(&self, a: i32, b: i32, comparison: i32) -> bool {
        match comparison {
            -2 => a == b,
            -1 => a > b,
            0 => a < b,
            1 => a >= b,
            2 => a <= b,
            _ => false,
        }
    }

    fn query_network_lobbies(&self) -> Result<Vec<Lobby>> {
        // Query lobbies from Oracle relay servers
        oracle_networking::relay::query_lobbies()
    }
}
