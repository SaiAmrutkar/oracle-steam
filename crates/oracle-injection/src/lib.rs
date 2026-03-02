pub mod early_injector;
pub mod injector;
pub mod loader;
pub mod process;

pub use early_injector::EarlyInjector;
pub use injector::Injector;
pub use loader::load_dll;
pub use process::{find_process, list_processes, Process};

use anyhow::Result;
use std::path::Path;

/// Inject DLL into process by name
pub fn inject_into_process(process_name: &str, dll_path: &Path) -> Result<()> {
    log::info!("Target process: {}", process_name);
    log::info!("DLL path: {}", dll_path.display());

    let process = find_process(process_name)?;
    let injector = Injector::new(process.pid, dll_path.to_path_buf());
    injector.inject()?;

    log::info!("✓ Injection successful");
    Ok(())
}

/// Inject DLL into process by PID
pub fn inject_into_pid(pid: u32, dll_path: &Path) -> Result<()> {
    log::info!("Target PID: {}", pid);
    log::info!("DLL path: {}", dll_path.display());

    let injector = Injector::new(pid, dll_path.to_path_buf());
    injector.inject()?;

    log::info!("✓ Injection successful");
    Ok(())
}

/// Auto-detect and inject into Steam
pub fn inject_into_steam(dll_path: &Path) -> Result<()> {
    inject_into_process("Steam.exe", dll_path)
        .or_else(|_| inject_into_process("steam.exe", dll_path))
}

/// Launch Steam with early injection (GreenLuma-style)
/// Starts Steam suspended, injects hook, then resumes
pub fn launch_steam_with_hook(steam_path: &Path, dll_path: &Path) -> Result<u32> {
    log::info!("Launching Steam with early injection...");
    log::info!("Steam: {}", steam_path.display());
    log::info!("Hook DLL: {}", dll_path.display());

    let injector = EarlyInjector::new(steam_path.to_path_buf(), dll_path.to_path_buf());
    let pid = injector.inject_early()?;

    log::info!("✓ Steam launched with hook (PID: {})", pid);
    Ok(pid)
}

/// Replace Steam DLLs in game folder
pub fn replace_steam_dlls(game_dir: &Path) -> Result<()> {
    log::info!("Replacing Steam DLLs in: {}", game_dir.display());

    let exe_dir = std::env::current_exe()?.parent().unwrap().to_path_buf();

    // Backup and replace steam_api64.dll
    let api64_target = game_dir.join("steam_api64.dll");
    if api64_target.exists() {
        let backup = game_dir.join("steam_api64.dll.backup");
        if !backup.exists() {
            std::fs::copy(&api64_target, backup)?;
            log::info!("✓ Backed up steam_api64.dll");
        }
    }

    let api64_source = exe_dir.join("steam_api64.dll");
    if api64_source.exists() {
        std::fs::copy(&api64_source, &api64_target)?;
        log::info!("✓ Replaced steam_api64.dll");
    }

    // Backup and replace steam_api.dll (32-bit)
    let api32_target = game_dir.join("steam_api.dll");
    if api32_target.exists() {
        let backup = game_dir.join("steam_api.dll.backup");
        if !backup.exists() {
            std::fs::copy(&api32_target, backup)?;
            log::info!("✓ Backed up steam_api.dll");
        }
    }

    let api32_source = exe_dir.join("steam_api.dll");
    if api32_source.exists() {
        std::fs::copy(&api32_source, &api32_target)?;
        log::info!("✓ Replaced steam_api.dll");
    }

    Ok(())
}

/// Restore original Steam DLLs
pub fn restore_steam_dlls(game_dir: &Path) -> Result<()> {
    log::info!("Restoring original Steam DLLs in: {}", game_dir.display());

    let api64_backup = game_dir.join("steam_api64.dll.backup");
    if api64_backup.exists() {
        let target = game_dir.join("steam_api64.dll");
        std::fs::copy(&api64_backup, &target)?;
        std::fs::remove_file(&api64_backup)?;
        log::info!("✓ Restored steam_api64.dll");
    }

    let api32_backup = game_dir.join("steam_api.dll.backup");
    if api32_backup.exists() {
        let target = game_dir.join("steam_api.dll");
        std::fs::copy(&api32_backup, &target)?;
        std::fs::remove_file(&api32_backup)?;
        log::info!("✓ Restored steam_api.dll");
    }

    Ok(())
}
