use anyhow::{Context, Result};
use std::path::Path;

pub fn inject_into_steam(hook_dll_path: &Path) -> Result<()> {
    // Find Steam.exe process
    let steam_pid = find_steam_process()?;

    log::info!("Found Steam process: PID {}", steam_pid);

    // Inject oracle-hook.dll
    let injector = crate::inject::Injector::new(steam_pid, hook_dll_path.to_path_buf());
    injector.inject()?;

    log::info!("Successfully injected oracle-hook.dll into Steam");

    Ok(())
}

fn find_steam_process() -> Result<u32> {
    crate::inject::find_process_by_name("Steam.exe")
        .or_else(|_| crate::inject::find_process_by_name("steam.exe"))
        .context("Steam client not running")
}
