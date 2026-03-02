use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

// Obfuscated constants (values XORed with random keys)
const MAGIC_KEY_1: u64 = 0xDEADBEEFCAFEBABE ^ 0x1234567890ABCDEF;
const MAGIC_KEY_2: u64 = 0xFEEDFACEBADC0FFE ^ 0xABCDEF1234567890;
const HEARTBEAT_INTERVAL: u64 = 5000; // ms

lazy_static! {
    static ref PROTECTION_STATE: RwLock<ProtectionState> = RwLock::new(ProtectionState::new());
    static ref LAST_HEARTBEAT: AtomicU64 = AtomicU64::new(0);
    static ref ORACLE_APP_RUNNING: AtomicBool = AtomicBool::new(false);
}

#[derive(Clone)]
struct ProtectionState {
    session_key: u64,
    init_time: u64,
    validation_counter: u32,
    dll_checksum: u64,
}

impl ProtectionState {
    fn new() -> Self {
        Self {
            session_key: Self::generate_session_key(),
            init_time: current_timestamp(),
            validation_counter: 0,
            dll_checksum: 0,
        }
    }

    fn generate_session_key() -> u64 {
        let t = current_timestamp();
        (t ^ MAGIC_KEY_1).wrapping_mul(MAGIC_KEY_2)
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

// Called by game on startup - validates Oracle Steam is running
#[no_mangle]
pub extern "C" fn OracleSteam_Init() -> bool {
    let mut state = PROTECTION_STATE.write();
    state.session_key = ProtectionState::generate_session_key();
    state.init_time = current_timestamp();

    // Validate Oracle app is running
    if !check_oracle_app_running() {
        return false;
    }

    ORACLE_APP_RUNNING.store(true, Ordering::SeqCst);
    LAST_HEARTBEAT.store(current_timestamp(), Ordering::SeqCst);

    println!("[OracleSteam Protection] Initialized");
    true
}

// Validates session key - must be called periodically by game
#[no_mangle]
pub extern "C" fn OracleSteam_Validate(key: u64) -> bool {
    let mut state = PROTECTION_STATE.write();

    // Check heartbeat
    let now = current_timestamp();
    let last = LAST_HEARTBEAT.load(Ordering::SeqCst);
    if now - last > HEARTBEAT_INTERVAL * 3 {
        return false;
    }

    // Validate key
    let expected = calculate_validation_key(state.session_key, state.validation_counter);

    if key != expected {
        return false;
    }

    state.validation_counter = state.validation_counter.wrapping_add(1);
    LAST_HEARTBEAT.store(now, Ordering::SeqCst);

    // Check Oracle app still running
    check_oracle_app_running()
}

// Called by Oracle app to send heartbeat
#[no_mangle]
pub extern "C" fn OracleSteam_Heartbeat() {
    LAST_HEARTBEAT.store(current_timestamp(), Ordering::SeqCst);
    ORACLE_APP_RUNNING.store(true, Ordering::SeqCst);
}

// Generate next validation key for game to use
#[no_mangle]
pub extern "C" fn OracleSteam_GetNextKey() -> u64 {
    let state = PROTECTION_STATE.read();
    calculate_validation_key(state.session_key, state.validation_counter)
}

fn calculate_validation_key(session: u64, counter: u32) -> u64 {
    let c = counter as u64;
    (session ^ MAGIC_KEY_1)
        .wrapping_add(c.wrapping_mul(MAGIC_KEY_2))
        .rotate_left((c % 64) as u32)
}

// Check if Oracle app is running (implement based on your needs)
fn check_oracle_app_running() -> bool {
    // Option 1: Check for specific process name
    #[cfg(windows)]
    {
        if find_process_by_name("oracle_steam.exe").is_some() {
            return true;
        }
    }

    // Option 2: Check for IPC endpoint (named pipe/socket)
    #[cfg(windows)]
    {
        use std::fs;
        let pipe_path = r"\\.\pipe\oracle_steam_heartbeat";
        if fs::metadata(pipe_path).is_ok() {
            return true;
        }
    }

    // Option 3: Check shared memory flag
    ORACLE_APP_RUNNING.load(Ordering::SeqCst)
}

#[cfg(windows)]
fn find_process_by_name(name: &str) -> Option<u32> {
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::tlhelp32::{
        CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
    };

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot.is_null() {
            return None;
        }

        let mut entry: PROCESSENTRY32 = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        if Process32First(snapshot, &mut entry) != 0 {
            loop {
                let bytes: Vec<u8> = entry
                    .szExeFile
                    .iter()
                    .take_while(|&&c| c != 0)
                    .map(|&c| c as u8)
                    .collect::<Vec<u8>>();
                let process_name = String::from_utf8_lossy(&bytes).into_owned();

                if process_name.to_lowercase() == name.to_lowercase() {
                    let pid = entry.th32ProcessID;
                    CloseHandle(snapshot);
                    return Some(pid);
                }

                if Process32Next(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
    }

    None
}

// Checksum verification for DLL integrity
#[no_mangle]
pub extern "C" fn OracleSteam_VerifyIntegrity(expected_checksum: u64) -> bool {
    // Read current checksum
    let current_checksum = {
        let state = PROTECTION_STATE.read();
        state.dll_checksum
    };

    // If first call, store the expected checksum
    if current_checksum == 0 {
        PROTECTION_STATE.write().dll_checksum = expected_checksum;
        return true;
    }

    // Verify checksum matches
    current_checksum == expected_checksum
}

// Anti-debugging checks
#[cfg(windows)]
#[no_mangle]
pub extern "C" fn OracleSteam_CheckDebugger() -> bool {
    use winapi::um::debugapi::IsDebuggerPresent;
    unsafe { IsDebuggerPresent() == 0 }
}

#[cfg(not(windows))]
#[no_mangle]
pub extern "C" fn OracleSteam_CheckDebugger() -> bool {
    true
}

// Game should call this in main loop
#[inline(always)]
pub fn validate_runtime() -> bool {
    let now = current_timestamp();
    let last = LAST_HEARTBEAT.load(Ordering::SeqCst);

    // If heartbeat timeout, kill game
    if now - last > HEARTBEAT_INTERVAL * 5 {
        return false;
    }

    // Check Oracle app still running
    if !ORACLE_APP_RUNNING.load(Ordering::SeqCst) {
        return false;
    }

    true
}

// Obfuscated failure - crashes game on tampering
#[inline(never)]
pub fn protection_failure() -> ! {
    // Overwrite stack
    let mut buffer = [0u8; 1024];
    for i in 0..1024 {
        buffer[i] = 0xFF;
    }

    // Cause segfault
    unsafe {
        let ptr = std::ptr::null_mut::<u32>();
        *ptr = 0xDEADC0DE;
    }

    unreachable!()
}
