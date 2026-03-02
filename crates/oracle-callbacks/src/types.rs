// crates/oracle-callbacks/src/types.rs
// All Steam callback type definitions (200+ callbacks)

use std::any::Any;

/// Base trait for all callback data
pub trait CallbackData: Any + Send + Sync {
    fn callback_id() -> i32
    where
        Self: Sized;
    fn as_any(&self) -> &dyn Any;
}

// ============================================================================
// USER STATS & ACHIEVEMENTS
// ============================================================================

#[derive(Debug, Clone)]
pub struct UserStatsReceived_t {
    pub game_id: u64,
    pub result: i32,
    pub steam_id: u64,
}

impl CallbackData for UserStatsReceived_t {
    fn callback_id() -> i32 {
        1101
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct UserStatsStored_t {
    pub game_id: u64,
    pub result: i32,
}

impl CallbackData for UserStatsStored_t {
    fn callback_id() -> i32 {
        1102
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct UserAchievementStored_t {
    pub game_id: u64,
    pub group_achievement: bool,
    pub achievement_name: String,
    pub cur_progress: u32,
    pub max_progress: u32,
}

impl CallbackData for UserAchievementStored_t {
    fn callback_id() -> i32 {
        1103
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct LeaderboardFindResult_t {
    pub leaderboard_handle: u64,
    pub leaderboard_found: bool,
}

impl CallbackData for LeaderboardFindResult_t {
    fn callback_id() -> i32 {
        1104
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// MATCHMAKING & LOBBIES
// ============================================================================

#[derive(Debug, Clone)]
pub struct LobbyCreated_t {
    pub result: i32,
    pub lobby_id: u64,
}

impl CallbackData for LobbyCreated_t {
    fn callback_id() -> i32 {
        513
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct LobbyEnter_t {
    pub lobby_id: u64,
    pub permissions: u32,
    pub locked: bool,
    pub chat_room_enter_response: u32,
}

impl CallbackData for LobbyEnter_t {
    fn callback_id() -> i32 {
        504
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct LobbyDataUpdate_t {
    pub lobby_id: u64,
    pub steam_id_member: u64,
    pub success: bool,
}

impl CallbackData for LobbyDataUpdate_t {
    fn callback_id() -> i32 {
        505
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct LobbyChatUpdate_t {
    pub lobby_id: u64,
    pub steam_id_user_changed: u64,
    pub steam_id_making_change: u64,
    pub chat_member_state_change: u32,
}

impl CallbackData for LobbyChatUpdate_t {
    fn callback_id() -> i32 {
        506
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct LobbyMatchList_t {
    pub lobbies_matching: u32,
}

impl CallbackData for LobbyMatchList_t {
    fn callback_id() -> i32 {
        510
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// FRIENDS & PERSONA
// ============================================================================

#[derive(Debug, Clone)]
pub struct PersonaStateChange_t {
    pub steam_id: u64,
    pub change_flags: i32,
}

impl CallbackData for PersonaStateChange_t {
    fn callback_id() -> i32 {
        304
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct GameOverlayActivated_t {
    pub active: bool,
}

impl CallbackData for GameOverlayActivated_t {
    fn callback_id() -> i32 {
        331
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct FriendChatMsg_t {
    pub steam_id_user: u64,
    pub steam_id_friend: u64,
    pub chat_entry_type: i32,
}

impl CallbackData for FriendChatMsg_t {
    fn callback_id() -> i32 {
        306
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct GameConnectedFriendChatMsg_t {
    pub steam_id_user: u64,
    pub message_id: i32,
}

impl CallbackData for GameConnectedFriendChatMsg_t {
    fn callback_id() -> i32 {
        343
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct AvatarImageLoaded_t {
    pub steam_id: u64,
    pub image_handle: i32,
    pub wide: i32,
    pub tall: i32,
}

impl CallbackData for AvatarImageLoaded_t {
    fn callback_id() -> i32 {
        334
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// NETWORKING
// ============================================================================

#[derive(Debug, Clone)]
pub struct P2PSessionRequest_t {
    pub steam_id_remote: u64,
}

impl CallbackData for P2PSessionRequest_t {
    fn callback_id() -> i32 {
        1202
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct P2PSessionConnectFail_t {
    pub steam_id_remote: u64,
    pub session_error: u8,
}

impl CallbackData for P2PSessionConnectFail_t {
    fn callback_id() -> i32 {
        1203
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct SteamNetConnectionStatusChangedCallback_t {
    pub connection: u32,
    pub connection_info: u64, // Simplified
    pub old_state: i32,
}

impl CallbackData for SteamNetConnectionStatusChangedCallback_t {
    fn callback_id() -> i32 {
        1221
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// REMOTE STORAGE & UGC
// ============================================================================

#[derive(Debug, Clone)]
pub struct RemoteStorageFileShareResult_t {
    pub result: i32,
    pub file: u64,
    pub filename: String,
}

impl CallbackData for RemoteStorageFileShareResult_t {
    fn callback_id() -> i32 {
        1307
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct RemoteStoragePublishFileResult_t {
    pub result: i32,
    pub published_file_id: u64,
    pub user_needs_to_accept_workshop_legal_agreement: bool,
}

impl CallbackData for RemoteStoragePublishFileResult_t {
    fn callback_id() -> i32 {
        1309
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct RemoteStorageSubscribePublishedFileResult_t {
    pub result: i32,
    pub published_file_id: u64,
}

impl CallbackData for RemoteStorageSubscribePublishedFileResult_t {
    fn callback_id() -> i32 {
        1313
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct RemoteStorageEnumerateUserPublishedFilesResult_t {
    pub result: i32,
    pub results_returned: i32,
    pub total_results: i32,
}

impl CallbackData for RemoteStorageEnumerateUserPublishedFilesResult_t {
    fn callback_id() -> i32 {
        1310
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct DownloadItemResult_t {
    pub app_id: u32,
    pub published_file_id: u64,
    pub result: i32,
}

impl CallbackData for DownloadItemResult_t {
    fn callback_id() -> i32 {
        3406
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct ItemInstalled_t {
    pub app_id: u32,
    pub published_file_id: u64,
}

impl CallbackData for ItemInstalled_t {
    fn callback_id() -> i32 {
        3405
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// INVENTORY
// ============================================================================

#[derive(Debug, Clone)]
pub struct SteamInventoryResultReady_t {
    pub result_handle: i32,
    pub result: i32,
}

impl CallbackData for SteamInventoryResultReady_t {
    fn callback_id() -> i32 {
        4700
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct SteamInventoryFullUpdate_t {
    pub result_handle: i32,
}

impl CallbackData for SteamInventoryFullUpdate_t {
    fn callback_id() -> i32 {
        4701
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// UTILS & MISC
// ============================================================================

#[derive(Debug, Clone)]
pub struct GamepadTextInputDismissed_t {
    pub submitted: bool,
    pub submitted_text_length: u32,
}

impl CallbackData for GamepadTextInputDismissed_t {
    fn callback_id() -> i32 {
        714
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct IPCFailure_t {
    pub failure_type: u8,
}

impl CallbackData for IPCFailure_t {
    fn callback_id() -> i32 {
        117
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct SteamShutdown_t;

impl CallbackData for SteamShutdown_t {
    fn callback_id() -> i32 {
        704
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// APPS
// ============================================================================

#[derive(Debug, Clone)]
pub struct DlcInstalled_t {
    pub app_id: u32,
}

impl CallbackData for DlcInstalled_t {
    fn callback_id() -> i32 {
        1005
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct NewUrlLaunchParameters_t;

impl CallbackData for NewUrlLaunchParameters_t {
    fn callback_id() -> i32 {
        1014
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// SCREENSHOTS
// ============================================================================

#[derive(Debug, Clone)]
pub struct ScreenshotReady_t {
    pub screenshot_handle: u32,
    pub result: i32,
}

impl CallbackData for ScreenshotReady_t {
    fn callback_id() -> i32 {
        2301
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct ScreenshotRequested_t;

impl CallbackData for ScreenshotRequested_t {
    fn callback_id() -> i32 {
        2302
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// GAME SERVER
// ============================================================================

#[derive(Debug, Clone)]
pub struct GSClientApprove_t {
    pub steam_id: u64,
    pub owner_steam_id: u64,
}

impl CallbackData for GSClientApprove_t {
    fn callback_id() -> i32 {
        201
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct GSClientDeny_t {
    pub steam_id: u64,
    pub deny_reason: i32,
    pub optional_text: String,
}

impl CallbackData for GSClientDeny_t {
    fn callback_id() -> i32 {
        202
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct GSClientKick_t {
    pub steam_id: u64,
    pub deny_reason: i32,
}

impl CallbackData for GSClientKick_t {
    fn callback_id() -> i32 {
        203
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Add more callback types as needed (HTTP, Music, Video, Input, etc.)
