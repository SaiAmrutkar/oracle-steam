use anyhow::Result;
use std::path::PathBuf;

pub struct SteamShortcutManager {
    shortcuts_vdf_path: PathBuf,
}

impl SteamShortcutManager {
    pub fn new() -> Result<Self> {
        // Steam shortcuts.vdf location
        let steam_path = Self::get_steam_install_path()?;
        let shortcuts_vdf_path = steam_path
            .join("userdata")
            .join("*")
            .join("config")
            .join("shortcuts.vdf");

        Ok(Self { shortcuts_vdf_path })
    }

    pub fn add_non_steam_game(&self, exe_path: &str, game_name: &str, app_id: u32) -> Result<()> {
        // Parse existing shortcuts.vdf
        let mut shortcuts = self.parse_shortcuts_vdf()?;

        // Add new entry
        shortcuts.push(ShortcutEntry {
            app_id,
            app_name: game_name.to_string(),
            exe: exe_path.to_string(),
            start_dir: std::path::Path::new(exe_path)
                .parent()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            icon: String::new(),
            shortcut_path: String::new(),
            launch_options: String::new(),
            is_hidden: false,
            allow_desktop_config: true,
            allow_overlay: true,
            openvr: false,
            devkit: false,
            devkit_game_id: String::new(),
            last_play_time: 0,
            tags: vec![],
        });

        // Write back to shortcuts.vdf
        self.write_shortcuts_vdf(&shortcuts)?;

        log::info!("Added non-Steam game: {} (AppID: {})", game_name, app_id);
        Ok(())
    }

    fn get_steam_install_path() -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            use winreg::enums::*;
            use winreg::RegKey;

            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let steam_key = hklm.open_subkey("SOFTWARE\\WOW6432Node\\Valve\\Steam")?;
            let install_path: String = steam_key.get_value("InstallPath")?;
            Ok(PathBuf::from(install_path))
        }

        #[cfg(not(target_os = "windows"))]
        {
            Ok(PathBuf::from(std::env::var("HOME")?).join(".steam"))
        }
    }

    fn parse_shortcuts_vdf(&self) -> Result<Vec<ShortcutEntry>> {
        // Parse binary VDF format
        // This is complex - use keyvalues-parser crate or implement VDF parser
        Ok(Vec::new())
    }

    fn write_shortcuts_vdf(&self, shortcuts: &[ShortcutEntry]) -> Result<()> {
        // Write binary VDF format
        Ok(())
    }
}

struct ShortcutEntry {
    app_id: u32,
    app_name: String,
    exe: String,
    start_dir: String,
    icon: String,
    shortcut_path: String,
    launch_options: String,
    is_hidden: bool,
    allow_desktop_config: bool,
    allow_overlay: bool,
    openvr: bool,
    devkit: bool,
    devkit_game_id: String,
    last_play_time: u32,
    tags: Vec<String>,
}
