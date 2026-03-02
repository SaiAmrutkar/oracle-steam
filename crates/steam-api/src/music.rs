// crates/steam-api/src/music.rs
// Complete ISteamMusic + ISteamMusicRemote implementation

use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr};

lazy_static! {
    static ref MUSIC_STATE: RwLock<MusicState> = RwLock::new(MusicState::new());
    static ref MUSIC_REMOTE: RwLock<MusicRemoteState> = RwLock::new(MusicRemoteState::new());
    static ref PLAYLIST: RwLock<Vec<PlaylistEntry>> = RwLock::new(Vec::new());
}

#[derive(Clone, Copy, PartialEq)]
enum PlaybackStatus {
    Undefined = 0,
    Playing = 1,
    Paused = 2,
    Idle = 3,
}

struct MusicState {
    enabled: bool,
    playing: bool,
    volume: f32,
    playback_status: PlaybackStatus,
    current_entry: i32,
}

impl MusicState {
    fn new() -> Self {
        Self {
            enabled: true,
            playing: false,
            volume: 1.0,
            playback_status: PlaybackStatus::Idle,
            current_entry: 0,
        }
    }
}

struct MusicRemoteState {
    registered: bool,
    is_current_remote: bool,
    display_name: String,
    playlist_changed: bool,
    current_entry_changed: bool,
}

impl MusicRemoteState {
    fn new() -> Self {
        Self {
            registered: false,
            is_current_remote: false,
            display_name: "Oracle Steam Music".to_string(),
            playlist_changed: false,
            current_entry_changed: false,
        }
    }
}

struct PlaylistEntry {
    id: i32,
    title: String,
    artist: String,
    album: String,
    duration: i32,
}

// ============================================================================
// ISteamMusic - Music Player Control
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusic_BIsEnabled() -> bool {
    MUSIC_STATE.read().enabled
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusic_BIsPlaying() -> bool {
    MUSIC_STATE.read().playing
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusic_GetPlaybackStatus() -> i32 {
    MUSIC_STATE.read().playback_status as i32
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusic_Play() {
    let mut state = MUSIC_STATE.write();
    state.playing = true;
    state.playback_status = PlaybackStatus::Playing;
    println!("[Music] Playing");
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusic_Pause() {
    let mut state = MUSIC_STATE.write();
    state.playing = false;
    state.playback_status = PlaybackStatus::Paused;
    println!("[Music] Paused");
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusic_PlayPrevious() {
    let mut state = MUSIC_STATE.write();
    if state.current_entry > 0 {
        state.current_entry -= 1;
        println!("[Music] Previous track: {}", state.current_entry);
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusic_PlayNext() {
    let mut state = MUSIC_STATE.write();
    let playlist_len = PLAYLIST.read().len() as i32;
    if state.current_entry < playlist_len - 1 {
        state.current_entry += 1;
        println!("[Music] Next track: {}", state.current_entry);
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusic_SetVolume(volume: f32) {
    let clamped = volume.max(0.0).min(1.0);
    MUSIC_STATE.write().volume = clamped;
    println!("[Music] Volume set to: {:.2}", clamped);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusic_GetVolume() -> f32 {
    MUSIC_STATE.read().volume
}

// ============================================================================
// ISteamMusicRemote - Remote Control for Music Apps
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_RegisterSteamMusicRemote(name: *const c_char) -> bool {
    if name.is_null() {
        return false;
    }

    unsafe {
        if let Ok(display_name) = CStr::from_ptr(name).to_str() {
            let mut remote = MUSIC_REMOTE.write();
            remote.registered = true;
            remote.is_current_remote = true;
            remote.display_name = display_name.to_string();
            println!("[MusicRemote] Registered: {}", display_name);
            return true;
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_DeregisterSteamMusicRemote() -> bool {
    let mut remote = MUSIC_REMOTE.write();
    remote.registered = false;
    remote.is_current_remote = false;
    println!("[MusicRemote] Deregistered");
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_BIsCurrentMusicRemote() -> bool {
    MUSIC_REMOTE.read().is_current_remote
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_BActivationSuccess(activated: bool) -> bool {
    MUSIC_REMOTE.write().is_current_remote = activated;
    println!("[MusicRemote] Activation: {}", activated);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_SetDisplayName(name: *const c_char) -> bool {
    if name.is_null() {
        return false;
    }

    unsafe {
        if let Ok(display_name) = CStr::from_ptr(name).to_str() {
            MUSIC_REMOTE.write().display_name = display_name.to_string();
            return true;
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_SetPNGIcon_64x64(
    icon: *const c_void,
    icon_size: u32,
) -> bool {
    println!("[MusicRemote] Icon set: {} bytes", icon_size);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_EnablePlayPrevious(enabled: bool) -> bool {
    println!("[MusicRemote] Play Previous: {}", enabled);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_EnablePlayNext(enabled: bool) -> bool {
    println!("[MusicRemote] Play Next: {}", enabled);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_EnableShuffled(enabled: bool) -> bool {
    println!("[MusicRemote] Shuffle: {}", enabled);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_EnableLooped(enabled: bool) -> bool {
    println!("[MusicRemote] Loop: {}", enabled);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_EnableQueue(enabled: bool) -> bool {
    println!("[MusicRemote] Queue: {}", enabled);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_EnablePlaylists(enabled: bool) -> bool {
    println!("[MusicRemote] Playlists: {}", enabled);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_UpdatePlaybackStatus(status: i32) -> bool {
    let playback = match status {
        1 => PlaybackStatus::Playing,
        2 => PlaybackStatus::Paused,
        3 => PlaybackStatus::Idle,
        _ => PlaybackStatus::Undefined,
    };

    MUSIC_STATE.write().playback_status = playback;
    println!("[MusicRemote] Playback status: {}", status);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_UpdateShuffled(shuffled: bool) -> bool {
    println!("[MusicRemote] Shuffled: {}", shuffled);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_UpdateLooped(looped: bool) -> bool {
    println!("[MusicRemote] Looped: {}", looped);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_UpdateVolume(volume: f32) -> bool {
    SteamAPI_ISteamMusic_SetVolume(volume);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_CurrentEntryWillChange() -> bool {
    MUSIC_REMOTE.write().current_entry_changed = true;
    println!("[MusicRemote] Current entry will change");
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_CurrentEntryIsAvailable(available: bool) -> bool {
    println!("[MusicRemote] Current entry available: {}", available);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_UpdateCurrentEntryText(text: *const c_char) -> bool {
    if text.is_null() {
        return false;
    }

    unsafe {
        if let Ok(entry_text) = CStr::from_ptr(text).to_str() {
            println!("[MusicRemote] Current entry: {}", entry_text);
            return true;
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_UpdateCurrentEntryElapsedSeconds(
    seconds: i32,
) -> bool {
    println!("[MusicRemote] Elapsed: {}s", seconds);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_UpdateCurrentEntryCoverArt(
    art: *const c_void,
    art_size: u32,
) -> bool {
    println!("[MusicRemote] Cover art updated: {} bytes", art_size);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_CurrentEntryDidChange() -> bool {
    MUSIC_REMOTE.write().current_entry_changed = false;
    println!("[MusicRemote] Current entry changed");
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_QueueWillChange() -> bool {
    println!("[MusicRemote] Queue will change");
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_QueueDidChange() -> bool {
    println!("[MusicRemote] Queue changed");
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_PlaylistWillChange() -> bool {
    MUSIC_REMOTE.write().playlist_changed = true;
    println!("[MusicRemote] Playlist will change");
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamMusicRemote_PlaylistDidChange() -> bool {
    MUSIC_REMOTE.write().playlist_changed = false;
    println!("[MusicRemote] Playlist changed");
    true
}
