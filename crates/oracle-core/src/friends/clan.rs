use anyhow::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::ffi::CString;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ClanInfo {
    pub steamid: u64,
    pub name: String,
    pub tag: String,
    pub owner: u64,
    pub officers: Vec<u64>,
    pub members: Vec<u64>,
    pub online_count: i32,
    pub in_game_count: i32,
    pub chatting_count: i32,
    pub chat_room_id: u64,
    pub is_public: bool,
    pub is_official_game_group: bool,
}

pub struct ClanManager {
    my_steamid: u64,
    clans: HashMap<u64, ClanInfo>,
    clan_chat_rooms: HashMap<u64, ClanChatRoom>,
}

#[derive(Debug, Clone)]
struct ClanChatRoom {
    clan_id: u64,
    members: Vec<u64>,
    messages: Vec<ChatMessage>,
    admins: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub sender_steamid: u64,
    pub text: String,
    pub entry_type: i32,
    pub timestamp: u64,
}

impl ClanManager {
    pub fn new(my_steamid: u64) -> Arc<Self> {
        Arc::new(Self {
            my_steamid,
            clans: HashMap::new(),
            clan_chat_rooms: HashMap::new(),
        })
    }

    pub fn get_clan_count(&self) -> i32 {
        self.clans.len() as i32
    }

    pub fn get_clan_by_index(&self, index: i32) -> u64 {
        self.clans.keys().nth(index as usize).copied().unwrap_or(0)
    }

    pub fn get_clan_name_ptr(&self, steamid_clan: u64) -> *const i8 {
        self.clans
            .get(&steamid_clan)
            .and_then(|c| CString::new(c.name.clone()).ok())
            .map(|s| s.into_raw() as *const i8)
            .unwrap_or(std::ptr::null())
    }

    pub fn get_clan_tag_ptr(&self, steamid_clan: u64) -> *const i8 {
        self.clans
            .get(&steamid_clan)
            .and_then(|c| CString::new(c.tag.clone()).ok())
            .map(|s| s.into_raw() as *const i8)
            .unwrap_or(std::ptr::null())
    }

    pub fn get_activity_counts(&self, steamid_clan: u64) -> Option<(i32, i32, i32)> {
        self.clans
            .get(&steamid_clan)
            .map(|c| (c.online_count, c.in_game_count, c.chatting_count))
    }

    pub fn get_clan_owner(&self, steamid_clan: u64) -> u64 {
        self.clans.get(&steamid_clan).map(|c| c.owner).unwrap_or(0)
    }

    pub fn get_officer_count(&self, steamid_clan: u64) -> i32 {
        self.clans
            .get(&steamid_clan)
            .map(|c| c.officers.len() as i32)
            .unwrap_or(0)
    }

    pub fn get_officer_by_index(&self, steamid_clan: u64, index: i32) -> u64 {
        self.clans
            .get(&steamid_clan)
            .and_then(|c| c.officers.get(index as usize))
            .copied()
            .unwrap_or(0)
    }

    pub fn join_clan_chat_room(&mut self, steamid_clan: u64) -> Result<()> {
        if !self.clan_chat_rooms.contains_key(&steamid_clan) {
            self.clan_chat_rooms.insert(
                steamid_clan,
                ClanChatRoom {
                    clan_id: steamid_clan,
                    members: vec![self.my_steamid],
                    messages: Vec::new(),
                    admins: Vec::new(),
                },
            );
        }
        Ok(())
    }

    pub fn leave_clan_chat_room(&mut self, steamid_clan: u64) -> bool {
        self.clan_chat_rooms.remove(&steamid_clan).is_some()
    }

    pub fn get_clan_chat_member_count(&self, steamid_clan: u64) -> i32 {
        self.clan_chat_rooms
            .get(&steamid_clan)
            .map(|room| room.members.len() as i32)
            .unwrap_or(0)
    }

    pub fn get_chat_member_by_index(&self, steamid_clan: u64, index: i32) -> u64 {
        self.clan_chat_rooms
            .get(&steamid_clan)
            .and_then(|room| room.members.get(index as usize))
            .copied()
            .unwrap_or(0)
    }

    pub fn send_clan_chat_message(&mut self, steamid_clan_chat: u64, text: String) -> bool {
        if let Some(room) = self.clan_chat_rooms.get_mut(&steamid_clan_chat) {
            room.messages.push(ChatMessage {
                sender_steamid: self.my_steamid,
                text,
                entry_type: 1, // k_EChatEntryTypeChatMsg
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
            true
        } else {
            false
        }
    }

    pub fn get_clan_chat_message(
        &self,
        steamid_clan_chat: u64,
        message_index: i32,
    ) -> Option<ChatMessage> {
        self.clan_chat_rooms
            .get(&steamid_clan_chat)
            .and_then(|room| room.messages.get(message_index as usize))
            .cloned()
    }

    pub fn is_clan_chat_admin(&self, steamid_clan_chat: u64, steamid_user: u64) -> bool {
        self.clan_chat_rooms
            .get(&steamid_clan_chat)
            .map(|room| room.admins.contains(&steamid_user))
            .unwrap_or(false)
    }

    pub fn add_clan(&mut self, clan: ClanInfo) {
        self.clans.insert(clan.steamid, clan);
    }
}
