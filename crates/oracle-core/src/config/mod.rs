use anyhow::{Context, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;

static CONFIG: OnceLock<RwLock<OracleConfig>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleConfig {
    pub app_id: u32,
    pub steam_id: u64,
    pub username: String,
    pub offline_mode: bool,
    pub data_dir: PathBuf,
    pub user_data_folder: PathBuf,
    pub language: String,
    pub enable_overlay: bool,
    pub enable_voice_chat: bool,
    pub use_real_steam: bool,
    pub machine_id: String,
}

impl Default for OracleConfig {
    fn default() -> Self {
        let data_dir = get_default_data_dir();
        let user_data_folder = data_dir.join("userdata");

        Self {
            app_id: 0,
            steam_id: 0,
            username: String::from("OracleUser"),
            offline_mode: true,
            data_dir,
            user_data_folder,
            language: String::from("english"),
            enable_overlay: true,
            enable_voice_chat: true,
            use_real_steam: false,
            machine_id: generate_machine_id(),
        }
    }
}

pub fn init_config() -> Result<()> {
    let config_path = get_config_path()?;

    let config = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        OracleConfig::default()
    };

    CONFIG
        .set(RwLock::new(config))
        .map_err(|_| anyhow::anyhow!("Config already initialized"))?;

    Ok(())
}

pub fn load_config_from_file(path: &std::path::Path) -> Result<()> {
    let content = std::fs::read_to_string(path)?;
    let config: OracleConfig = serde_json::from_str(&content)?;

    *get_config_lock().write() = config;
    Ok(())
}

pub fn save_config() -> Result<()> {
    let config = get_config_lock().read().clone();
    let config_path = get_config_path()?;

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(&config)?;
    std::fs::write(config_path, json)?;

    Ok(())
}

pub fn get_app_id() -> u32 {
    if let Ok(env_app_id) = std::env::var("SteamAppId") {
        env_app_id
            .parse()
            .unwrap_or_else(|_| get_config_lock().read().app_id)
    } else {
        get_config_lock().read().app_id
    }
}

pub fn set_app_id(app_id: u32) {
    get_config_lock().write().app_id = app_id;
}

pub fn get_steam_id() -> u64 {
    get_config_lock().read().steam_id
}

pub fn set_steam_id(steam_id: u64) {
    get_config_lock().write().steam_id = steam_id;
}

pub fn get_username() -> String {
    get_config_lock().read().username.clone()
}

pub fn set_username(username: String) {
    get_config_lock().write().username = username;
}

pub fn is_offline_mode() -> bool {
    get_config_lock().read().offline_mode
}

pub fn set_offline_mode(offline: bool) {
    get_config_lock().write().offline_mode = offline;
}

pub fn get_data_dir() -> Result<PathBuf> {
    Ok(get_config_lock().read().data_dir.clone())
}

pub fn get_user_data_path() -> String {
    get_config_lock()
        .read()
        .user_data_folder
        .to_string_lossy()
        .to_string()
}

pub fn get_language() -> String {
    get_config_lock().read().language.clone()
}

pub fn get_machine_id() -> String {
    get_config_lock().read().machine_id.clone()
}

pub fn use_real_steam() -> bool {
    get_config_lock().read().use_real_steam
}

pub fn set_use_real_steam(use_real: bool) {
    get_config_lock().write().use_real_steam = use_real;
}

fn get_config_lock() -> &'static RwLock<OracleConfig> {
    CONFIG.get_or_init(|| RwLock::new(OracleConfig::default()))
}

fn get_config_path() -> Result<PathBuf> {
    let mut path = get_default_data_dir();
    path.push("oracle_config.json");
    Ok(path)
}

fn get_default_data_dir() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("OracleSteam");
    path
}

fn generate_machine_id() -> String {
    use sha2::{Digest, Sha256};

    let hostname = hostname::get()
        .unwrap_or_else(|_| std::ffi::OsString::from("unknown"))
        .to_string_lossy()
        .to_string();

    let mut hasher = Sha256::new();
    hasher.update(hostname.as_bytes());
    hasher.update(b"OracleSteamMachineID");

    hex::encode(hasher.finalize())
}
