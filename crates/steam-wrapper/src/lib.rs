pub mod wrapper;

#[no_mangle]
pub extern "C" fn SteamAPI_Init() -> bool {
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_Shutdown() {}
