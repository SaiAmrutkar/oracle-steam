use parking_lot::RwLock;
use std::collections::HashMap;
use std::ffi::CString;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub struct UtilsManager {
    app_start_time: Instant,
    computer_active_time: Instant,
    ip_country: String,
    ip_country_cstring: CString,
    images: HashMap<i32, ImageData>,
    overlay_position: i32,
    overlay_inset: (i32, i32),
    warning_hook: Option<usize>,
    vr_mode: bool,
    big_picture_mode: bool,
    vr_streaming_enabled: bool,
    text_filter_initialized: bool,
    text_filter_options: u32,
    gamepad_text_input: Option<GamepadTextInput>,
    ipv6_state: i32,
}

#[derive(Debug, Clone)]
struct ImageData {
    width: u32,
    height: u32,
    rgba_data: Vec<u8>,
}

#[derive(Debug, Clone)]
struct GamepadTextInput {
    text: String,
    mode: i32,
    floating_mode: i32,
}

impl UtilsManager {
    pub fn new() -> Arc<RwLock<Self>> {
        let ip_country = Self::detect_country();
        let ip_country_cstring = CString::new(ip_country.clone()).unwrap();

        Arc::new(RwLock::new(Self {
            app_start_time: Instant::now(),
            computer_active_time: Instant::now(),
            ip_country,
            ip_country_cstring,
            images: HashMap::new(),
            overlay_position: 0, // k_EPositionTopLeft
            overlay_inset: (0, 0),
            warning_hook: None,
            vr_mode: false,
            big_picture_mode: false,
            vr_streaming_enabled: false,
            text_filter_initialized: false,
            text_filter_options: 0,
            gamepad_text_input: None,
            ipv6_state: 0, // k_ESteamIPv6ConnectivityState_Unknown
        }))
    }

    pub fn get_seconds_since_app_active(&self) -> u32 {
        self.app_start_time.elapsed().as_secs() as u32
    }

    pub fn get_seconds_since_computer_active(&self) -> u32 {
        self.computer_active_time.elapsed().as_secs() as u32
    }

    pub fn get_ip_country_ptr(&self) -> *const i8 {
        self.ip_country_cstring.as_ptr()
    }

    pub fn get_image_size(&self, image: i32) -> Option<(u32, u32)> {
        self.images.get(&image).map(|img| (img.width, img.height))
    }

    pub fn get_image_data(&self, image: i32) -> Option<Vec<u8>> {
        self.images.get(&image).map(|img| img.rgba_data.clone())
    }

    pub fn add_image(&mut self, handle: i32, width: u32, height: u32, rgba_data: Vec<u8>) {
        self.images.insert(
            handle,
            ImageData {
                width,
                height,
                rgba_data,
            },
        );
    }

    pub fn get_battery_power(&self) -> u8 {
        // Query system battery status
        #[cfg(target_os = "windows")]
        {
            unsafe {
                use winapi::um::winbase::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
                let mut status: SYSTEM_POWER_STATUS = std::mem::zeroed();
                if GetSystemPowerStatus(&mut status) != 0 {
                    return status.BatteryLifePercent;
                }
            }
        }

        255 // Unknown
    }

    pub fn set_overlay_notification_position(&mut self, position: i32) {
        self.overlay_position = position;
        oracle_overlay::set_notification_position(position);
    }

    pub fn set_warning_message_hook(&mut self, function: usize) {
        self.warning_hook = Some(function);
    }

    pub fn is_vr_mode(&self) -> bool {
        self.vr_mode
    }

    pub fn set_overlay_notification_inset(&mut self, horizontal: i32, vertical: i32) {
        self.overlay_inset = (horizontal, vertical);
        oracle_overlay::set_notification_inset(horizontal, vertical);
    }

    pub fn is_big_picture_mode(&self) -> bool {
        self.big_picture_mode
    }

    pub fn is_vr_streaming_enabled(&self) -> bool {
        self.vr_streaming_enabled
    }

    pub fn set_vr_streaming_enabled(&mut self, enabled: bool) {
        self.vr_streaming_enabled = enabled;
    }

    pub fn init_text_filter(&mut self, options: u32) -> bool {
        self.text_filter_initialized = true;
        self.text_filter_options = options;
        true
    }

    pub fn filter_text(&self, _context: i32, _source_steamid: u64, input: &str) -> Option<String> {
        if !self.text_filter_initialized {
            return None;
        }

        // Apply profanity filter based on options
        let filtered = oracle_core::text_filter::filter(input, self.text_filter_options);
        Some(filtered)
    }

    pub fn get_ipv6_connectivity_state(&self, _protocol: i32) -> i32 {
        self.ipv6_state
    }

    pub fn show_gamepad_text_input(
        &mut self,
        input_mode: i32,
        line_input_mode: i32,
        description: String,
        max_chars: u32,
        existing_text: String,
    ) -> bool {
        self.gamepad_text_input = Some(GamepadTextInput {
            text: existing_text,
            mode: input_mode,
            floating_mode: line_input_mode,
        });

        oracle_overlay::show_gamepad_text_input(description, max_chars);
        true
    }

    pub fn get_gamepad_text_length(&self) -> u32 {
        self.gamepad_text_input
            .as_ref()
            .map(|input| input.text.len() as u32)
            .unwrap_or(0)
    }

    pub fn get_gamepad_text(&self) -> Option<String> {
        self.gamepad_text_input
            .as_ref()
            .map(|input| input.text.clone())
    }

    pub fn show_floating_gamepad_input(
        &mut self,
        keyboard_mode: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> bool {
        oracle_overlay::show_floating_gamepad_input(keyboard_mode, x, y, width, height)
    }

    pub fn set_gamepad_text_input_mode(&mut self, mode: i32) {
        if let Some(ref mut input) = self.gamepad_text_input {
            input.mode = mode;
        }
    }

    pub fn set_floating_gamepad_text_input_mode(&mut self, mode: i32) {
        if let Some(ref mut input) = self.gamepad_text_input {
            input.floating_mode = mode;
        }
    }

    pub fn dismiss_floating_gamepad_text_input(&mut self) -> bool {
        oracle_overlay::dismiss_floating_gamepad_input();
        true
    }

    pub fn dismiss_gamepad_text_input(&mut self) -> bool {
        self.gamepad_text_input = None;
        oracle_overlay::dismiss_gamepad_input();
        true
    }

    fn detect_country() -> String {
        // Try to detect country from IP or system settings
        #[cfg(target_os = "windows")]
        {
            // Query Windows locale
            use winapi::um::winnls::GetUserDefaultLocaleName;
            unsafe {
                let mut locale = [0u16; 85];
                if GetUserDefaultLocaleName(locale.as_mut_ptr(), 85) > 0 {
                    let locale_str = String::from_utf16_lossy(&locale);
                    if locale_str.len() >= 5 {
                        return locale_str[3..5].to_uppercase();
                    }
                }
            }
        }

        String::from("US")
    }
}
