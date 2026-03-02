pub mod api;
pub mod exports;
pub mod flat;
pub mod interfaces;

use once_cell::sync::Lazy;
use parking_lot::RwLock;

static INITIALIZED: Lazy<RwLock<bool>> = Lazy::new(|| RwLock::new(false));

#[no_mangle]
pub extern "C" fn SteamAPI_Init() -> bool {
    let mut init = INITIALIZED.write();
    if *init {
        return true;
    }
    *init = true;
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_Shutdown() {
    *INITIALIZED.write() = false;
}

#[no_mangle]
pub extern "C" fn SteamAPI_RunCallbacks() {}

#[no_mangle]
pub extern "C" fn SteamAPI_IsSteamRunning() -> bool {
    true
}
