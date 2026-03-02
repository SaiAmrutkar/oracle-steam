use crate::STEAM_CLIENT;
use std::ffi::c_char;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref NAME_BUFFER: Mutex<[u8; 256]> = Mutex::new([0; 256]);
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetPersonaName() -> *const c_char {
    if let Some(client) = STEAM_CLIENT.read().as_ref() {
        let name = client.get_username();
        let bytes = name.as_bytes();
        let len = bytes.len().min(255);

        let mut buffer = NAME_BUFFER.lock().unwrap();
        buffer[..len].copy_from_slice(&bytes[..len]);
        buffer[len] = 0;

        buffer.as_ptr() as *const c_char
    } else {
        b"Unknown\0".as_ptr() as *const c_char
    }
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_GetFriendCount(_flags: i32) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamFriends_SetPersonaName(_name: *const c_char) -> bool {
    true
}
