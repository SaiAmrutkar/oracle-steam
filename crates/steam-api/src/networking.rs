use std::ffi::c_void;

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworking_SendP2PPacket(
    steam_id_remote: u64,
    data: *const c_void,
    data_size: u32,
    _send_type: i32,
    _channel: i32,
) -> bool {
    if data.is_null() {
        return false;
    }

    println!(
        "[SteamEmu] P2P packet sent to {} ({} bytes)",
        steam_id_remote, data_size
    );
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworking_IsP2PPacketAvailable(
    msg_size: *mut u32,
    _channel: i32,
) -> bool {
    if !msg_size.is_null() {
        unsafe {
            *msg_size = 0;
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworking_ReadP2PPacket(
    _dest: *mut c_void,
    _dest_size: u32,
    _msg_size: *mut u32,
    _steam_id_remote: *mut u64,
    _channel: i32,
) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworking_AcceptP2PSessionWithUser(steam_id_remote: u64) -> bool {
    println!("[SteamEmu] Accepted P2P session with {}", steam_id_remote);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworking_CloseP2PSessionWithUser(steam_id_remote: u64) -> bool {
    println!("[SteamEmu] Closed P2P session with {}", steam_id_remote);
    true
}
