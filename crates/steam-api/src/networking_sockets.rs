// ISteamNetworkingSockets - Modern reliable UDP networking (2018+)
// This is Steam's NEW networking API that replaced the old P2P system

use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr};
use std::net::SocketAddr;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

pub type HSteamNetConnection = u32;
pub type HSteamListenSocket = u32;
pub type HSteamNetPollGroup = u32;

#[repr(C)]
pub struct SteamNetworkingIdentity {
    identity_type: i32,
    steam_id: u64,
    ip_addr: [u8; 16],
    generic_string: [u8; 128],
    generic_bytes: [u8; 32],
}

#[repr(C)]
pub struct SteamNetworkingIPAddr {
    ipv6: [u8; 16],
    port: u16,
}

#[repr(C)]
pub struct SteamNetworkingMessage_t {
    data: *mut c_void,
    data_size: u32,
    conn_handle: HSteamNetConnection,
    identity: SteamNetworkingIdentity,
    user_data: i64,
    time_received: i64,
    message_number: i64,
    release_func: *mut c_void,
    channel: i32,
    flags: i32,
}

#[repr(C)]
pub struct SteamNetConnectionInfo_t {
    identity_remote: SteamNetworkingIdentity,
    user_data: i64,
    listen_socket: HSteamListenSocket,
    addr_remote: SteamNetworkingIPAddr,
    pad: u16,
    pop_remote: u32,
    pop_relay: u32,
    state: i32,
    end_reason: i32,
    end_debug: [c_char; 128],
    debug_description: [c_char; 128],
}

lazy_static! {
    static ref CONNECTIONS: RwLock<HashMap<HSteamNetConnection, Connection>> =
        RwLock::new(HashMap::new());
    static ref LISTEN_SOCKETS: RwLock<HashMap<HSteamListenSocket, ListenSocket>> =
        RwLock::new(HashMap::new());
    static ref POLL_GROUPS: RwLock<HashMap<HSteamNetPollGroup, PollGroup>> =
        RwLock::new(HashMap::new());
    static ref NEXT_HANDLE: RwLock<u32> = RwLock::new(1);
}

struct Connection {
    handle: HSteamNetConnection,
    remote_identity: SteamNetworkingIdentity,
    remote_addr: SocketAddr,
    state: i32, // k_ESteamNetworkingConnectionState
    send_queue: Vec<Vec<u8>>,
    recv_queue: Vec<Vec<u8>>,
    user_data: i64,
}

struct ListenSocket {
    handle: HSteamListenSocket,
    local_port: u16,
    pending_connections: Vec<HSteamNetConnection>,
}

struct PollGroup {
    handle: HSteamNetPollGroup,
    connections: Vec<HSteamNetConnection>,
}

// ============================================================================
// CONNECTION MANAGEMENT
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_CreateListenSocketIP(
    local_address: *const SteamNetworkingIPAddr,
    options_count: i32,
    options: *const c_void,
) -> HSteamListenSocket {
    let port = unsafe {
        if local_address.is_null() {
            27015
        } else {
            (*local_address).port
        }
    };

    let handle = {
        let mut next = NEXT_HANDLE.write();
        let h = *next;
        *next += 1;
        h
    };

    let socket = ListenSocket {
        handle,
        local_port: port,
        pending_connections: Vec::new(),
    };

    LISTEN_SOCKETS.write().insert(handle, socket);

    println!(
        "[Oracle] Listen socket created on port {}: handle={}",
        port, handle
    );
    handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_ConnectByIPAddress(
    address: *const SteamNetworkingIPAddr,
    options_count: i32,
    options: *const c_void,
) -> HSteamNetConnection {
    if address.is_null() {
        return 0;
    }

    let handle = {
        let mut next = NEXT_HANDLE.write();
        let h = *next;
        *next += 1;
        h
    };

    // Parse address
    let addr_str = unsafe {
        let ipv6 = &(*address).ipv6;
        let port = (*address).port;

        // Try to parse as IPv4 first
        if ipv6[0..12] == [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff] {
            let ipv4 = format!(
                "{}.{}.{}.{}:{}",
                ipv6[12], ipv6[13], ipv6[14], ipv6[15], port
            );
            ipv4.parse().ok()
        } else {
            None
        }
    }
    .unwrap_or_else(|| "127.0.0.1:27015".parse().unwrap());

    let connection = Connection {
        handle,
        remote_identity: unsafe { std::mem::zeroed() },
        remote_addr: addr_str,
        state: 2, // k_ESteamNetworkingConnectionState_Connecting
        send_queue: Vec::new(),
        recv_queue: Vec::new(),
        user_data: 0,
    };

    CONNECTIONS.write().insert(handle, connection);

    println!("[Oracle] Connecting to {}: handle={}", addr_str, handle);
    handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_AcceptConnection(
    conn_handle: HSteamNetConnection,
) -> i32 {
    let mut connections = CONNECTIONS.write();
    if let Some(conn) = connections.get_mut(&conn_handle) {
        conn.state = 3; // k_ESteamNetworkingConnectionState_Connected
        println!("[Oracle] Connection accepted: {}", conn_handle);
        return 1; // k_EResultOK
    }
    2 // k_EResultFail
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_CloseConnection(
    peer: HSteamNetConnection,
    reason: i32,
    debug: *const c_char,
    enable_linger: bool,
) -> bool {
    let removed = CONNECTIONS.write().remove(&peer).is_some();

    if removed {
        println!("[Oracle] Connection closed: {} (reason: {})", peer, reason);
    }

    removed
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_CloseListenSocket(
    sock: HSteamListenSocket,
) -> bool {
    let removed = LISTEN_SOCKETS.write().remove(&sock).is_some();

    if removed {
        println!("[Oracle] Listen socket closed: {}", sock);
    }

    removed
}

// ============================================================================
// SENDING DATA
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_SendMessageToConnection(
    conn: HSteamNetConnection,
    data: *const c_void,
    data_size: u32,
    send_flags: i32,
    out_message_number: *mut i64,
) -> i32 {
    if data.is_null() || data_size == 0 {
        return 2; // k_EResultFail
    }

    let mut connections = CONNECTIONS.write();
    if let Some(connection) = connections.get_mut(&conn) {
        // Copy data to send queue
        let data_vec =
            unsafe { std::slice::from_raw_parts(data as *const u8, data_size as usize).to_vec() };

        connection.send_queue.push(data_vec);

        if !out_message_number.is_null() {
            unsafe {
                *out_message_number = connection.send_queue.len() as i64;
            }
        }

        println!("[Oracle] Sent {} bytes to connection {}", data_size, conn);
        return 1; // k_EResultOK
    }

    2 // k_EResultFail
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_FlushMessagesOnConnection(
    conn: HSteamNetConnection,
) -> i32 {
    // In real implementation, this would flush the UDP send buffer
    println!("[Oracle] Flushed messages on connection: {}", conn);
    1 // k_EResultOK
}

// ============================================================================
// RECEIVING DATA
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_ReceiveMessagesOnConnection(
    conn: HSteamNetConnection,
    messages: *mut *mut SteamNetworkingMessage_t,
    max_messages: i32,
) -> i32 {
    if messages.is_null() || max_messages < 1 {
        return 0;
    }

    let mut connections = CONNECTIONS.write();
    if let Some(connection) = connections.get_mut(&conn) {
        let count = connection.recv_queue.len().min(max_messages as usize);

        // TODO: Allocate SteamNetworkingMessage_t and populate
        // For now, return 0 (no messages)
        return 0;
    }

    0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_ReceiveMessagesOnPollGroup(
    poll_group: HSteamNetPollGroup,
    messages: *mut *mut SteamNetworkingMessage_t,
    max_messages: i32,
) -> i32 {
    // Similar to ReceiveMessagesOnConnection but for poll groups
    0
}

// ============================================================================
// POLL GROUPS (for multiplayer with multiple connections)
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_CreatePollGroup() -> HSteamNetPollGroup {
    let handle = {
        let mut next = NEXT_HANDLE.write();
        let h = *next;
        *next += 1;
        h
    };

    let group = PollGroup {
        handle,
        connections: Vec::new(),
    };

    POLL_GROUPS.write().insert(handle, group);

    println!("[Oracle] Poll group created: {}", handle);
    handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_DestroyPollGroup(
    poll_group: HSteamNetPollGroup,
) -> bool {
    POLL_GROUPS.write().remove(&poll_group).is_some()
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_SetConnectionPollGroup(
    conn: HSteamNetConnection,
    poll_group: HSteamNetPollGroup,
) -> bool {
    let mut groups = POLL_GROUPS.write();
    if let Some(group) = groups.get_mut(&poll_group) {
        if !group.connections.contains(&conn) {
            group.connections.push(conn);
        }
        return true;
    }
    false
}

// ============================================================================
// CONNECTION INFO
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_GetConnectionInfo(
    conn: HSteamNetConnection,
    info: *mut SteamNetConnectionInfo_t,
) -> bool {
    if info.is_null() {
        return false;
    }

    let connections = CONNECTIONS.read();
    if let Some(connection) = connections.get(&conn) {
        unsafe {
            (*info).identity_remote = connection.remote_identity;
            (*info).user_data = connection.user_data;
            (*info).state = connection.state;
            (*info).end_reason = 0;
        }
        return true;
    }

    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_GetConnectionRealTimeStatus(
    conn: HSteamNetConnection,
    status: *mut c_void, // SteamNetConnectionRealTimeStatus_t
    lane_count: i32,
    lanes: *mut c_void, // SteamNetConnectionRealTimeLaneStatus_t
) -> i32 {
    // Return connection quality metrics
    // For now, return k_EResultOK with no data
    1
}

// ============================================================================
// RELAY SERVERS (for NAT traversal)
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_GetHostedDedicatedServerAddress(
    routing_blob: *mut c_void,
    routing_blob_size: *mut i32,
) -> i32 {
    // Return relay server address for this client
    1 // k_EResultOK
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_CreateHostedDedicatedServerListenSocket(
    virtual_port: i32,
    options_count: i32,
    options: *const c_void,
) -> HSteamListenSocket {
    // Create listen socket that works through relay
    SteamAPI_ISteamNetworkingSockets_CreateListenSocketIP(std::ptr::null(), options_count, options)
}

// ============================================================================
// P2P CONNECTIONS (through relay or direct)
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_ConnectP2P(
    identity_remote: *const SteamNetworkingIdentity,
    virtual_port: i32,
    options_count: i32,
    options: *const c_void,
) -> HSteamNetConnection {
    if identity_remote.is_null() {
        return 0;
    }

    let handle = {
        let mut next = NEXT_HANDLE.write();
        let h = *next;
        *next += 1;
        h
    };

    let connection = Connection {
        handle,
        remote_identity: unsafe { *identity_remote },
        remote_addr: "0.0.0.0:0".parse().unwrap(),
        state: 2, // Connecting
        send_queue: Vec::new(),
        recv_queue: Vec::new(),
        user_data: 0,
    };

    CONNECTIONS.write().insert(handle, connection);

    println!("[Oracle] P2P connection initiated: {}", handle);
    handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_AcceptP2PSessionWithUser(
    identity_remote: *const SteamNetworkingIdentity,
) -> bool {
    println!("[Oracle] P2P session accepted");
    true
}

// ============================================================================
// CONFIGURATION
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_SetConnectionUserData(
    peer: HSteamNetConnection,
    user_data: i64,
) -> bool {
    let mut connections = CONNECTIONS.write();
    if let Some(conn) = connections.get_mut(&peer) {
        conn.user_data = user_data;
        return true;
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_GetConnectionUserData(
    peer: HSteamNetConnection,
) -> i64 {
    CONNECTIONS
        .read()
        .get(&peer)
        .map(|c| c.user_data)
        .unwrap_or(0)
}

// ============================================================================
// UTILITIES
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_RunCallbacks() {
    // Process network events, state changes, etc.
    // In real implementation, this drives the network stack
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamNetworkingSockets_GetConnectionName(
    peer: HSteamNetConnection,
    name: *mut c_char,
    max_len: i32,
) -> bool {
    if name.is_null() || max_len < 1 {
        return false;
    }

    let connections = CONNECTIONS.read();
    if let Some(conn) = connections.get(&peer) {
        let conn_name = format!("Connection_{}", peer);
        let bytes = conn_name.as_bytes();
        let len = bytes.len().min((max_len - 1) as usize);

        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), name as *mut u8, len);
            *name.add(len) = 0;
        }

        return true;
    }

    false
}
