pub mod avatar;
pub mod clan;
pub mod messaging;
pub mod rich_presence;

use anyhow::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::ffi::CString;
use std::sync::Arc;

pub use avatar::AvatarManager;
pub use clan::{ChatMessage, ClanManager};
pub use messaging::MessageManager;
pub use rich_presence::RichPresenceManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum EPersonaState {
    Offline = 0,
    Online = 1,
    Busy = 2,
    Away = 3,
    Snooze = 4,
    LookingToTrade = 5,
    LookingToPlay = 6,
    Invisible = 7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum EFriendRelationship {
    None = 0,
    Blocked = 1,
    RequestRecipient = 2,
    Friend = 3,
    RequestInitiator = 4,
    Ignored = 5,
    IgnoredFriend = 6,
}

#[derive(Debug, Clone)]
pub struct FriendInfo {
    pub steamid: u64,
    pub persona_name: String,
    pub nickname: Option<String>,
    pub persona_state: EPersonaState,
    pub relationship: EFriendRelationship,
    pub steam_level: i32,
    pub game_info: Option<GameInfo>,
    pub name_history: Vec<String>,
    pub last_seen: u64,
}

#[derive(Debug, Clone)]
pub struct GameInfo {
    pub game_id: u64,
    pub game_ip: u32,
    pub game_port: u16,
    pub query_port: u16,
    pub lobby_id: u64,
}

pub struct FriendsManager {
    my_steamid: u64,
    persona_name: String,
    persona_name_cstring: CString,
    persona_state: EPersonaState,
    friends: HashMap<u64, FriendInfo>,
    friend_groups: HashMap<i16, FriendGroup>,
    avatar_manager: Arc<AvatarManager>,
    clan_manager: Arc<ClanManager>,
    rich_presence: Arc<RichPresenceManager>,
    message_manager: Arc<RwLock<MessageManager>>,
    voice_speaking: HashMap<u64, bool>,
    play_group: Option<u64>,
    listen_for_messages: bool,
    coplay_data: HashMap<u64, (u32, i32)>,
}

#[derive(Debug, Clone)]
struct FriendGroup {
    id: i16,
    name: String,
    members: Vec<u64>,
}

impl FriendsManager {
    pub fn new(my_steamid: u64, persona_name: String) -> Arc<RwLock<Self>> {
        let persona_name_cstring = CString::new(persona_name.clone()).unwrap();

        Arc::new(RwLock::new(Self {
            my_steamid,
            persona_name,
            persona_name_cstring,
            persona_state: EPersonaState::Online,
            friends: HashMap::new(),
            friend_groups: HashMap::new(),
            avatar_manager: AvatarManager::new(),
            clan_manager: ClanManager::new(my_steamid),
            rich_presence: RichPresenceManager::new(my_steamid),
            message_manager: Arc::new(RwLock::new(MessageManager::new(my_steamid))),
            voice_speaking: HashMap::new(),
            play_group: None,
            listen_for_messages: true,
            coplay_data: HashMap::new(),
        }))
    }

    pub fn get_persona_name_ptr(&self) -> *const i8 {
        self.persona_name_cstring.as_ptr()
    }

    pub fn set_persona_name(&mut self, name: String) -> Result<()> {
        self.persona_name = name.clone();
        self.persona_name_cstring = CString::new(name)?;
        Ok(())
    }

    pub fn get_persona_state(&self) -> i32 {
        self.persona_state as i32
    }

    pub fn get_friend_count(&self, friend_flags: i32) -> i32 {
        self.friends
            .values()
            .filter(|f| self.matches_friend_flags(f, friend_flags))
            .count() as i32
    }

    pub fn get_friend_by_index(&self, index: i32, friend_flags: i32) -> u64 {
        self.friends
            .values()
            .filter(|f| self.matches_friend_flags(f, friend_flags))
            .nth(index as usize)
            .map(|f| f.steamid)
            .unwrap_or(0)
    }

    pub fn get_friend_relationship(&self, steamid: u64) -> i32 {
        self.friends
            .get(&steamid)
            .map(|f| f.relationship as i32)
            .unwrap_or(EFriendRelationship::None as i32)
    }

    pub fn get_friend_persona_state(&self, steamid: u64) -> i32 {
        self.friends
            .get(&steamid)
            .map(|f| f.persona_state as i32)
            .unwrap_or(EPersonaState::Offline as i32)
    }

    pub fn get_friend_persona_name_ptr(&self, steamid: u64) -> *const i8 {
        self.friends
            .get(&steamid)
            .and_then(|f| CString::new(f.persona_name.clone()).ok())
            .map(|s| s.into_raw() as *const i8)
            .unwrap_or(std::ptr::null())
    }

    pub fn get_friend_game_played(&self, steamid: u64) -> Option<GameInfo> {
        self.friends.get(&steamid).and_then(|f| f.game_info.clone())
    }

    pub fn get_friend_persona_name_history_ptr(&self, steamid: u64, index: i32) -> *const i8 {
        self.friends
            .get(&steamid)
            .and_then(|f| f.name_history.get(index as usize))
            .and_then(|name| CString::new(name.clone()).ok())
            .map(|s| s.into_raw() as *const i8)
            .unwrap_or(std::ptr::null())
    }

    pub fn get_friend_steam_level(&self, steamid: u64) -> i32 {
        self.friends
            .get(&steamid)
            .map(|f| f.steam_level)
            .unwrap_or(0)
    }

    pub fn get_player_nickname_ptr(&self, steamid: u64) -> *const i8 {
        self.friends
            .get(&steamid)
            .and_then(|f| f.nickname.as_ref())
            .and_then(|nick| CString::new(nick.clone()).ok())
            .map(|s| s.into_raw() as *const i8)
            .unwrap_or(std::ptr::null())
    }

    pub fn get_friends_group_count(&self) -> i32 {
        self.friend_groups.len() as i32
    }

    pub fn get_friends_group_id_by_index(&self, index: i32) -> i16 {
        self.friend_groups
            .keys()
            .nth(index as usize)
            .copied()
            .unwrap_or(0)
    }

    pub fn get_friends_group_name_ptr(&self, group_id: i16) -> *const i8 {
        self.friend_groups
            .get(&group_id)
            .and_then(|g| CString::new(g.name.clone()).ok())
            .map(|s| s.into_raw() as *const i8)
            .unwrap_or(std::ptr::null())
    }

    pub fn get_friends_group_members_count(&self, group_id: i16) -> i32 {
        self.friend_groups
            .get(&group_id)
            .map(|g| g.members.len() as i32)
            .unwrap_or(0)
    }

    pub fn get_friends_group_members(&self, group_id: i16) -> Vec<u64> {
        self.friend_groups
            .get(&group_id)
            .map(|g| g.members.clone())
            .unwrap_or_default()
    }

    pub fn has_friend(&self, steamid: u64, friend_flags: i32) -> bool {
        self.friends
            .get(&steamid)
            .map(|f| self.matches_friend_flags(f, friend_flags))
            .unwrap_or(false)
    }

    pub fn get_clan_count(&self) -> i32 {
        self.clan_manager.get_clan_count()
    }

    pub fn get_clan_by_index(&self, index: i32) -> u64 {
        self.clan_manager.get_clan_by_index(index)
    }

    pub fn get_clan_name_ptr(&self, steamid_clan: u64) -> *const i8 {
        self.clan_manager.get_clan_name_ptr(steamid_clan)
    }

    pub fn get_clan_tag_ptr(&self, steamid_clan: u64) -> *const i8 {
        self.clan_manager.get_clan_tag_ptr(steamid_clan)
    }

    pub fn get_clan_activity_counts(&self, steamid_clan: u64) -> Option<(i32, i32, i32)> {
        self.clan_manager.get_activity_counts(steamid_clan)
    }

    pub fn get_friend_count_from_source(&self, _steamid_source: u64) -> i32 {
        0
    }

    pub fn get_friend_from_source_by_index(&self, _steamid_source: u64, _index: i32) -> u64 {
        0
    }

    pub fn is_user_in_source(&self, _steamid_user: u64, _steamid_source: u64) -> bool {
        false
    }

    pub fn set_in_game_voice_speaking(&mut self, steamid: u64, speaking: bool) {
        self.voice_speaking.insert(steamid, speaking);
    }

    pub fn set_play_group(&mut self, steamid_play_group: u64) {
        self.play_group = Some(steamid_play_group);
    }

    pub fn leave_play_group(&mut self) {
        self.play_group = None;
    }

    pub fn get_play_group_count(&self) -> i32 {
        if self.play_group.is_some() {
            1
        } else {
            0
        }
    }

    pub fn get_play_group_by_index(&self, index: i32) -> u64 {
        if index == 0 {
            self.play_group.unwrap_or(0)
        } else {
            0
        }
    }

    pub fn get_small_friend_avatar(&self, steamid: u64) -> i32 {
        self.avatar_manager.get_small_avatar(steamid)
    }

    pub fn get_medium_friend_avatar(&self, steamid: u64) -> i32 {
        self.avatar_manager.get_medium_avatar(steamid)
    }

    pub fn get_large_friend_avatar(&self, steamid: u64) -> i32 {
        self.avatar_manager.get_large_avatar(steamid)
    }

    pub fn request_user_information(&mut self, steamid: u64, _require_name_only: bool) -> bool {
        !self.friends.contains_key(&steamid)
    }

    pub fn get_clan_owner(&self, steamid_clan: u64) -> u64 {
        self.clan_manager.get_clan_owner(steamid_clan)
    }

    pub fn get_clan_officer_count(&self, steamid_clan: u64) -> i32 {
        self.clan_manager.get_officer_count(steamid_clan)
    }

    pub fn get_clan_officer_by_index(&self, steamid_clan: u64, index: i32) -> u64 {
        self.clan_manager.get_officer_by_index(steamid_clan, index)
    }

    pub fn set_rich_presence(&mut self, key: String, value: String) -> bool {
        self.rich_presence.set(key, value)
    }

    pub fn clear_rich_presence(&mut self) {
        self.rich_presence.clear();
    }

    pub fn get_friend_rich_presence_ptr(&self, steamid: u64, key: &str) -> *const i8 {
        self.rich_presence.get_friend_presence_ptr(steamid, key)
    }

    pub fn get_friend_rich_presence_key_count(&self, steamid: u64) -> i32 {
        self.rich_presence.get_friend_key_count(steamid)
    }

    pub fn get_friend_rich_presence_key_by_index_ptr(&self, steamid: u64, index: i32) -> *const i8 {
        self.rich_presence
            .get_friend_key_by_index_ptr(steamid, index)
    }

    pub fn request_friend_rich_presence(&mut self, _steamid: u64) {
        // Request rich presence data from network
    }

    pub fn get_coplay_friend_count(&self) -> i32 {
        self.coplay_data.len() as i32
    }

    pub fn get_coplay_friend(&self, index: i32) -> u64 {
        self.coplay_data
            .keys()
            .nth(index as usize)
            .copied()
            .unwrap_or(0)
    }

    pub fn get_friend_coplay_time(&self, steamid: u64) -> i32 {
        self.coplay_data
            .get(&steamid)
            .map(|(_, time)| *time)
            .unwrap_or(0)
    }

    pub fn get_friend_coplay_game(&self, steamid: u64) -> u32 {
        self.coplay_data
            .get(&steamid)
            .map(|(game, _)| *game)
            .unwrap_or(0)
    }

    pub fn join_clan_chat_room(&mut self, steamid_clan: u64) -> Result<()> {
        Arc::get_mut(&mut self.clan_manager)
            .unwrap()
            .join_clan_chat_room(steamid_clan)
    }

    pub fn leave_clan_chat_room(&mut self, steamid_clan: u64) -> bool {
        Arc::get_mut(&mut self.clan_manager)
            .unwrap()
            .leave_clan_chat_room(steamid_clan)
    }

    pub fn get_clan_chat_member_count(&self, steamid_clan: u64) -> i32 {
        self.clan_manager.get_clan_chat_member_count(steamid_clan)
    }

    pub fn get_chat_member_by_index(&self, steamid_clan: u64, index: i32) -> u64 {
        self.clan_manager
            .get_chat_member_by_index(steamid_clan, index)
    }

    pub fn send_clan_chat_message(&mut self, steamid_clan_chat: u64, text: String) -> bool {
        Arc::get_mut(&mut self.clan_manager)
            .unwrap()
            .send_clan_chat_message(steamid_clan_chat, text)
    }

    pub fn get_clan_chat_message(
        &self,
        steamid_clan_chat: u64,
        message_index: i32,
    ) -> Option<ChatMessage> {
        self.clan_manager
            .get_clan_chat_message(steamid_clan_chat, message_index)
    }

    pub fn is_clan_chat_admin(&self, steamid_clan_chat: u64, steamid_user: u64) -> bool {
        self.clan_manager
            .is_clan_chat_admin(steamid_clan_chat, steamid_user)
    }

    pub fn set_listen_for_friends_messages(&mut self, intercept_enabled: bool) {
        self.listen_for_messages = intercept_enabled;
    }

    pub fn reply_to_friend_message(&mut self, steamid: u64, msg_to_send: String) -> bool {
        self.message_manager
            .write()
            .send_message(steamid, msg_to_send)
    }

    pub fn get_friend_message(&self, steamid: u64, message_index: i32) -> Option<ChatMessage> {
        self.message_manager
            .read()
            .get_message(steamid, message_index)
    }

    fn matches_friend_flags(&self, friend: &FriendInfo, flags: i32) -> bool {
        if flags == 0xFFFF {
            return true;
        }

        if flags & 0x04 != 0 && friend.relationship == EFriendRelationship::Friend {
            return true;
        }

        false
    }
}
