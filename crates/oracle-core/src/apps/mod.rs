use anyhow::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::ffi::CString;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DLCInfo {
    pub app_id: u32,
    pub name: String,
    pub available: bool,
    pub installed: bool,
    pub download_progress: Option<(u64, u64)>,
}

#[derive(Debug, Clone)]
pub struct BetaInfo {
    pub app_id: u32,
    pub build_id: String,
    pub description: String,
    pub flags: u32,
    pub password_required: bool,
}

pub struct AppsManager {
    my_steamid: u64,
    current_app_id: u32,
    subscribed: bool,
    low_violence: bool,
    current_language: String,
    current_language_cstring: CString,
    available_languages: String,
    available_languages_cstring: CString,
    dlc_list: Vec<DLCInfo>,
    installed_apps: HashMap<u32, AppInstallInfo>,
    launch_params: HashMap<String, String>,
    build_id: i32,
    current_beta: Option<String>,
    betas: Vec<BetaInfo>,
    dlc_context: Option<u32>,
    timed_trial: Option<(u32, u32)>,
}

#[derive(Debug, Clone)]
struct AppInstallInfo {
    app_id: u32,
    install_dir: PathBuf,
    depots: Vec<u32>,
    purchase_time: u32,
}

impl AppsManager {
    pub fn new(steamid: u64, app_id: u32) -> Arc<RwLock<Self>> {
        let current_language = oracle_core::config::get_language();
        let current_language_cstring = CString::new(current_language.clone()).unwrap();

        let available_languages =
            String::from("english,french,german,spanish,russian,japanese,korean,schinese,tchinese");
        let available_languages_cstring = CString::new(available_languages.clone()).unwrap();

        Arc::new(RwLock::new(Self {
            my_steamid: steamid,
            current_app_id: app_id,
            subscribed: true,
            low_violence: false,
            current_language,
            current_language_cstring,
            available_languages,
            available_languages_cstring,
            dlc_list: Vec::new(),
            installed_apps: HashMap::new(),
            launch_params: HashMap::new(),
            build_id: 1000,
            current_beta: None,
            betas: Vec::new(),
            dlc_context: None,
            timed_trial: None,
        }))
    }

    pub fn is_subscribed(&self) -> bool {
        self.subscribed
    }

    pub fn is_low_violence(&self) -> bool {
        self.low_violence
    }

    pub fn get_current_language_ptr(&self) -> *const i8 {
        self.current_language_cstring.as_ptr()
    }

    pub fn get_available_languages_ptr(&self) -> *const i8 {
        self.available_languages_cstring.as_ptr()
    }

    pub fn is_subscribed_app(&self, app_id: u32) -> bool {
        if app_id == self.current_app_id {
            return self.subscribed;
        }

        oracle_core::licenses::has_license(self.my_steamid, app_id)
    }

    pub fn is_dlc_installed(&self, app_id: u32) -> bool {
        self.dlc_list
            .iter()
            .any(|dlc| dlc.app_id == app_id && dlc.installed)
    }

    pub fn get_earliest_purchase_time(&self, app_id: u32) -> u32 {
        self.installed_apps
            .get(&app_id)
            .map(|info| info.purchase_time)
            .unwrap_or(0)
    }

    pub fn get_dlc_count(&self) -> i32 {
        self.dlc_list.len() as i32
    }

    pub fn get_dlc_by_index(&self, index: i32) -> Option<&DLCInfo> {
        self.dlc_list.get(index as usize)
    }

    pub fn install_dlc(&mut self, app_id: u32) -> Result<()> {
        if let Some(dlc) = self.dlc_list.iter_mut().find(|d| d.app_id == app_id) {
            dlc.installed = true;
            dlc.download_progress = Some((0, 100000000)); // Start download

            // Queue callback
            oracle_core::callbacks::queue_callback(DlcInstalled_t { app_id });
        }

        Ok(())
    }

    pub fn uninstall_dlc(&mut self, app_id: u32) -> Result<()> {
        if let Some(dlc) = self.dlc_list.iter_mut().find(|d| d.app_id == app_id) {
            dlc.installed = false;
            dlc.download_progress = None;
        }

        Ok(())
    }

    pub fn get_current_beta_name(&self) -> Option<String> {
        self.current_beta.clone()
    }

    pub fn mark_content_corrupt(&mut self, _missing_files_only: bool) -> bool {
        // Mark content as corrupt for verification
        true
    }

    pub fn get_installed_depots(&self, app_id: u32) -> Vec<u32> {
        self.installed_apps
            .get(&app_id)
            .map(|info| info.depots.clone())
            .unwrap_or_default()
    }

    pub fn get_app_install_dir(&self, app_id: u32) -> Option<String> {
        self.installed_apps
            .get(&app_id)
            .map(|info| info.install_dir.to_string_lossy().to_string())
    }

    pub fn is_app_installed(&self, app_id: u32) -> bool {
        self.installed_apps.contains_key(&app_id)
    }

    pub fn get_app_owner(&self) -> u64 {
        self.my_steamid
    }

    pub fn get_launch_query_param_ptr(&self, key: &str) -> *const i8 {
        self.launch_params
            .get(key)
            .and_then(|value| CString::new(value.clone()).ok())
            .map(|s| s.into_raw() as *const i8)
            .unwrap_or(std::ptr::null())
    }

    pub fn get_dlc_download_progress(&self, app_id: u32) -> Option<(u64, u64)> {
        self.dlc_list
            .iter()
            .find(|dlc| dlc.app_id == app_id)
            .and_then(|dlc| dlc.download_progress)
    }

    pub fn get_app_build_id(&self) -> i32 {
        self.build_id
    }

    pub fn get_launch_command_line(&self) -> Option<String> {
        std::env::args()
            .skip(1)
            .reduce(|a, b| format!("{} {}", a, b))
    }

    pub fn get_timed_trial_info(&self) -> Option<(u32, u32)> {
        self.timed_trial
    }

    pub fn set_dlc_context(&mut self, app_id: u32) -> bool {
        self.dlc_context = Some(app_id);
        true
    }

    pub fn get_beta_counts(&self) -> (i32, i32) {
        let available = self.betas.iter().filter(|b| !b.password_required).count() as i32;
        let private = self.betas.iter().filter(|b| b.password_required).count() as i32;

        (available, private)
    }

    pub fn get_beta_info(&self, index: i32) -> Option<&BetaInfo> {
        self.betas.get(index as usize)
    }

    pub fn set_active_beta(&mut self, beta_name: String) -> bool {
        if beta_name.is_empty() {
            self.current_beta = None;
            return true;
        }

        if self.betas.iter().any(|b| b.build_id == beta_name) {
            self.current_beta = Some(beta_name);
            true
        } else {
            false
        }
    }

    pub fn add_dlc(&mut self, dlc: DLCInfo) {
        self.dlc_list.push(dlc);
    }

    pub fn add_installed_app(&mut self, app_id: u32, install_dir: PathBuf, depots: Vec<u32>) {
        self.installed_apps.insert(
            app_id,
            AppInstallInfo {
                app_id,
                install_dir,
                depots,
                purchase_time: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as u32,
            },
        );
    }

    pub fn set_launch_param(&mut self, key: String, value: String) {
        self.launch_params.insert(key, value);
    }

    pub fn add_beta(&mut self, beta: BetaInfo) {
        self.betas.push(beta);
    }
}
