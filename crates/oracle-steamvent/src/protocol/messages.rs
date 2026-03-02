// crates/oracle-steamvent/src/protocol/messages.rs
// Complete Steam Protocol Message Definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Steam protocol message types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum EMsgType {
    ClientLogon = 701,
    ClientLogonResponse = 751,
    ClientLogOff = 702,
    ClientHeartBeat = 703,
    ClientFriendsList = 767,
    ClientPersonaState = 766,
    ClientChatMsg = 797,
    ClientGetUserStats = 5613,
    ClientStoreUserStats = 5611,
    ClientLobbyCreate = 9535,
    ClientLobbyJoin = 9537,
    Invalid = 0,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamMessage {
    pub msg_type: EMsgType,
    pub payload: Vec<u8>,
}

impl SteamMessage {
    pub fn new(msg_type: EMsgType, payload: Vec<u8>) -> Self {
        Self { msg_type, payload }
    }
}
