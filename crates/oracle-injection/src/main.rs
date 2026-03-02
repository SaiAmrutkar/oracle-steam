use anyhow::Result;
use clap::{Parser, Subcommand};
use oracle_injection::{
    inject_into_pid, inject_into_process, inject_into_steam, list_processes, replace_steam_dlls,
    restore_steam_dlls,
};
use serde_json::json;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "oracle-injector")]
#[command(author = "Oracle Steam Project")]
#[command(version = "1.0.0")]
#[command(about = "Oracle Steam injection and installation tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Inject oracle_hook.dll into Steam.exe
    Steam {
        #[arg(short, long)]
        dll: Option<PathBuf>,
    },

    /// Inject DLL into process by PID
    Pid {
        pid: u32,
        #[arg(short, long)]
        dll: Option<PathBuf>,
    },

    /// Inject DLL into process by name
    Process {
        name: String,
        #[arg(short, long)]
        dll: Option<PathBuf>,
    },

    /// List all running processes
    List {
        #[arg(short, long)]
        filter: Option<String>,
    },

    /// Install Oracle Steam to game directory
    Install {
        game_dir: PathBuf,
        #[arg(short, long)]
        app_id: Option<u32>,
        #[arg(long)]
        no_achievements: bool,
    },

    /// Uninstall Oracle Steam from game directory
    Uninstall {
        game_dir: PathBuf,
        #[arg(long)]
        keep_data: bool,
    },

    /// Generate or update config
    Config {
        game_dir: PathBuf,
        #[arg(short, long)]
        username: Option<String>,
        #[arg(long)]
        steam_id: Option<u64>,
    },

    /// Show info about Oracle Steam installation
    Info { game_dir: PathBuf },

    /// Test overlay (future)
    Test,
    
    /// Start Steam with early injection (GreenLuma-style)
    /// Starts Steam suspended, injects hook, then resumes
    Launch {
        #[arg(short, long)]
        dll: Option<PathBuf>,
        #[arg(short, long)]
        steam_path: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    print_banner();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Steam { dll }) => inject_steam_cmd(dll),
        Some(Commands::Pid { pid, dll }) => inject_pid_cmd(pid, dll),
        Some(Commands::Process { name, dll }) => inject_process_cmd(name, dll),
        Some(Commands::List { filter }) => list_processes_cmd(filter),
        Some(Commands::Install {
            game_dir,
            app_id,
            no_achievements,
        }) => install_cmd(game_dir, app_id, !no_achievements),
        Some(Commands::Uninstall {
            game_dir,
            keep_data,
        }) => uninstall_cmd(game_dir, keep_data),
        Some(Commands::Config {
            game_dir,
            username,
            steam_id,
        }) => config_cmd(game_dir, username, steam_id),
        Some(Commands::Info { game_dir }) => info_cmd(game_dir),
        Some(Commands::Test) => test_cmd(),
        Some(Commands::Launch { dll, steam_path }) => launch_cmd(dll, steam_path),
        None => auto_inject(),
    }
}

fn print_banner() {
    println!("═══════════════════════════════════════════════════════");
    println!("  ██████╗ ██████╗  █████╗  ██████╗██╗     ███████╗   ");
    println!("  ██╔═══██╗██╔══██╗██╔══██╗██╔════╝██║     ██╔════╝   ");
    println!("  ██║   ██║██████╔╝███████║██║     ██║     █████╗     ");
    println!("  ██║   ██║██╔══██╗██╔══██║██║     ██║     ██╔══╝     ");
    println!("  ╚██████╔╝██║  ██║██║  ██║╚██████╗███████╗███████╗   ");
    println!("   ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚══════╝╚══════╝   ");
    println!("                                                       ");
    println!("          Oracle Steam Injector v1.0.0                ");
    println!("═══════════════════════════════════════════════════════\n");
}

fn auto_inject() -> Result<()> {
    println!("🎯 Auto-detecting Steam...\n");
    let dll_path = get_hook_dll_path()?;
    inject_into_steam(&dll_path)?;
    println!("\n✅ Injection complete!");
    println!("\n📋 To unlock games:");
    println!("   1. Create Steam/config/lua/ folder");
    println!("   2. Add {{appid}}.lua files with depot keys");
    println!("   3. Example: Steam/config/lua/3595270.lua");
    println!("   4. Check logs: Steam/config/logs/oracle_hook.log");
    Ok(())
}

fn inject_steam_cmd(dll: Option<PathBuf>) -> Result<()> {
    println!("🎯 Target: Steam.exe\n");
    let dll_path = dll.unwrap_or_else(|| get_hook_dll_path().unwrap());
    inject_into_steam(&dll_path)?;
    println!("\n✅ Injection complete!");
    Ok(())
}

fn inject_pid_cmd(pid: u32, dll: Option<PathBuf>) -> Result<()> {
    println!("🎯 Target PID: {}\n", pid);
    let dll_path = dll.unwrap_or_else(|| get_hook_dll_path().unwrap());
    inject_into_pid(pid, &dll_path)?;
    println!("\n✅ Injection complete!");
    Ok(())
}

fn inject_process_cmd(name: String, dll: Option<PathBuf>) -> Result<()> {
    println!("🎯 Target: {}\n", name);
    let dll_path = dll.unwrap_or_else(|| get_hook_dll_path().unwrap());
    inject_into_process(&name, &dll_path)?;
    println!("\n✅ Injection complete!");
    Ok(())
}

fn list_processes_cmd(filter: Option<String>) -> Result<()> {
    println!("📋 Running Processes:\n");
    let processes = list_processes()?;
    let filtered: Vec<_> = if let Some(f) = filter {
        processes
            .into_iter()
            .filter(|p| p.name.to_lowercase().contains(&f.to_lowercase()))
            .collect()
    } else {
        processes
    };
    for process in filtered.iter().take(50) {
        println!("   {} (PID: {})", process.name, process.pid);
    }
    println!("\n   Total: {} processes", filtered.len());
    Ok(())
}

fn install_cmd(game_dir: PathBuf, app_id: Option<u32>, with_achievements: bool) -> Result<()> {
    println!("🎮 Installing Oracle Steam to: {}\n", game_dir.display());

    if !game_dir.exists() {
        anyhow::bail!("Game directory does not exist");
    }

    println!("📦 [1/4] Copying steam_api64.dll...");
    replace_steam_dlls(&game_dir)?;

    if let Some(id) = app_id {
        println!("📝 [2/4] Creating steam_appid.txt...");
        std::fs::write(game_dir.join("steam_appid.txt"), id.to_string())?;
        log::info!("✓ Created steam_appid.txt ({})", id);
    } else {
        println!("⏭️  [2/4] Skipping steam_appid.txt");
    }

    println!("⚙️  [3/4] Generating configuration...");
    generate_default_config(&game_dir)?;

    if with_achievements {
        println!("🏆 [4/4] Setting up achievements...");
        setup_achievements(&game_dir, app_id.unwrap_or(480))?;
    } else {
        println!("⏭️  [4/4] Skipping achievements");
    }

    println!("\n✅ Installation complete!");
    println!("\n📖 Next steps:");
    println!("   1. Launch your game normally");
    println!("   2. Press Shift+Tab to open Oracle Steam overlay");
    println!("\n💡 Tip: Edit oracle_config.json to customize settings");
    Ok(())
}

fn uninstall_cmd(game_dir: PathBuf, keep_data: bool) -> Result<()> {
    println!(
        "🗑️  Uninstalling Oracle Steam from: {}\n",
        game_dir.display()
    );

    restore_steam_dlls(&game_dir)?;

    for file in &["steam_appid.txt", "oracle_config.json", "achievements.json"] {
        let path = game_dir.join(file);
        if path.exists() {
            std::fs::remove_file(&path)?;
            log::info!("✓ Removed {}", file);
        }
    }

    if !keep_data {
        let data_dir = game_dir.join("oracle_data");
        if data_dir.exists() {
            std::fs::remove_dir_all(&data_dir)?;
            log::info!("✓ Removed oracle_data");
        }
    }

    println!("\n✅ Uninstallation complete!");
    Ok(())
}

fn config_cmd(game_dir: PathBuf, username: Option<String>, steam_id: Option<u64>) -> Result<()> {
    let username = username.unwrap_or_else(|| format!("Player{}", rand::random::<u32>() % 10000));
    let steam_id =
        steam_id.unwrap_or_else(|| 76561198000000000u64 + (rand::random::<u64>() % 1000000));

    let config = json!({
        "user": {
            "username": username,
            "steam_id": steam_id,
            "language": "english"
        },
        "networking": {
            "enable_multiplayer": true,
            "server_address": "127.0.0.1",
            "port": 27015
        },
        "overlay": {
            "enabled": true,
            "hotkey": "Shift+Tab"
        }
    });

    std::fs::write(
        game_dir.join("oracle_config.json"),
        serde_json::to_string_pretty(&config)?,
    )?;
    println!(
        "✅ Configuration saved\n   Username: {}\n   Steam ID: {}",
        username, steam_id
    );
    Ok(())
}

fn info_cmd(game_dir: PathBuf) -> Result<()> {
    println!("📊 Oracle Steam Information");
    println!("   Directory: {}\n", game_dir.display());

    let dll = game_dir.join("steam_api64.dll");
    println!(
        "   {} Steam API DLL",
        if dll.exists() { "✅" } else { "❌" }
    );

    let config = game_dir.join("oracle_config.json");
    if config.exists() {
        println!("   ✅ Configuration");
        if let Ok(content) = std::fs::read_to_string(&config) {
            if let Ok(cfg) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(user) = cfg.get("user") {
                    if let Some(name) = user.get("username") {
                        println!("      Username: {}", name);
                    }
                }
            }
        }
    } else {
        println!("   ❌ Configuration");
    }

    let appid = game_dir.join("steam_appid.txt");
    if let Ok(id) = std::fs::read_to_string(&appid) {
        println!("   ✅ App ID: {}", id.trim());
    }

    println!();
    Ok(())
}

fn test_cmd() -> Result<()> {
    println!("🧪 Test mode not yet implemented");
    Ok(())
}

fn launch_cmd(dll: Option<PathBuf>, steam_path: Option<PathBuf>) -> Result<()> {
    println!("🚀 Launching Steam with early injection (GreenLuma-style)\n");
    
    let dll_path = dll.unwrap_or_else(|| get_hook_dll_path().unwrap());
    let steam_exe = steam_path.unwrap_or_else(|| PathBuf::from("C:\\Program Files (x86)\\Steam\\Steam.exe"));
    
    if !steam_exe.exists() {
        // Try alternative path
        let alt_path = PathBuf::from("C:\\Program Files\\Steam\\Steam.exe");
        if alt_path.exists() {
            return launch_cmd(Some(dll_path), Some(alt_path));
        }
        anyhow::bail!("Steam.exe not found at: {}. Use --steam-path to specify.", steam_exe.display());
    }
    
    use oracle_injection::launch_steam_with_hook;
    let pid = launch_steam_with_hook(&steam_exe, &dll_path)?;
    
    println!("\n✅ Steam launched with Oracle hook!");
    println!("   PID: {}", pid);
    println!("\n📋 The hook is now active from Steam startup.");
    println!("   Games should appear in your library.");
    println!("\n⚠️  Do NOT close this window - Steam is running!");
    
    // Wait for Steam to exit
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        
        // Check if Steam is still running
        let still_running = std::process::Command::new("tasklist")
            .args(&["/FI", &format!("PID eq {}", pid), "/NH"])
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).contains("Steam.exe"))
            .unwrap_or(false);
            
        if !still_running {
            println!("\n🛑 Steam has exited.");
            break;
        }
    }
    
    Ok(())
}

fn get_hook_dll_path() -> Result<PathBuf> {
    let exe_dir = std::env::current_exe()?.parent().unwrap().to_path_buf();
    let dll_path = exe_dir.join("oracle_hook.dll");
    if !dll_path.exists() {
        anyhow::bail!("oracle_hook.dll not found at: {}", dll_path.display());
    }
    log::info!("Using DLL: {}", dll_path.display());
    Ok(dll_path)
}

fn generate_default_config(game_dir: &PathBuf) -> Result<()> {
    let config = json!({
        "user": {
            "username": format!("Player{}", rand::random::<u32>() % 10000),
            "steam_id": 76561198000000000u64 + (rand::random::<u64>() % 1000000),
            "language": "english"
        },
        "networking": {
            "enable_multiplayer": true,
            "server_address": "127.0.0.1",
            "port": 27015
        },
        "overlay": {
            "enabled": true,
            "hotkey": "Shift+Tab"
        }
    });
    std::fs::write(
        game_dir.join("oracle_config.json"),
        serde_json::to_string_pretty(&config)?,
    )?;
    log::info!("✓ Created oracle_config.json");
    Ok(())
}

fn setup_achievements(game_dir: &PathBuf, app_id: u32) -> Result<()> {
    let ach = json!({
        "app_id": app_id,
        "achievements": [
            {"id": "ACH_FIRST", "name": "First Steps", "description": "Complete first level"}
        ]
    });
    std::fs::write(
        game_dir.join("achievements.json"),
        serde_json::to_string_pretty(&ach)?,
    )?;
    std::fs::create_dir_all(game_dir.join("oracle_data/user_data"))?;
    log::info!("✓ Created achievements.json");
    Ok(())
}
