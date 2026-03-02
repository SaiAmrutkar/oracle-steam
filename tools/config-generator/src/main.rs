use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("Oracle Steam - Configuration Generator");
    println!("======================================\n");

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let game_dir = PathBuf::from(&args[1]);

    if !game_dir.exists() {
        eprintln!("Error: Directory does not exist: {}", game_dir.display());
        return Ok(());
    }

    println!("Generating configuration for: {}\n", game_dir.display());

    // Generate main config
    generate_oracle_config(&game_dir)?;

    // Generate achievements template
    generate_achievements_template(&game_dir)?;

    // Generate stats template
    generate_stats_template(&game_dir)?;

    println!("\n✅ Configuration files generated successfully!");
    println!("\nFiles created:");
    println!("  - oracle_config.json");
    println!("  - achievements_template.json");
    println!("  - stats_template.json");

    Ok(())
}

fn print_usage() {
    println!("Usage: config-gen <game_directory>");
    println!("\nExample:");
    println!("  config-gen \"C:/Games/MyGame\"");
}

fn generate_oracle_config(game_dir: &PathBuf) -> Result<()> {
    let config = json!({
        "user": {
            "username": format!("Player{}", rand::random::<u32>() % 10000),
            "steam_id": 76561198000000000u64 + (rand::random::<u64>() % 1000000),
            "language": "english",
            "offline_mode": false
        },
        "networking": {
            "enable_multiplayer": true,
            "server_address": "127.0.0.1",
            "port": 27015,
            "use_lan": true,
            "enable_nat_traversal": true
        },
        "overlay": {
            "enabled": true,
            "hotkey": "Shift+Tab",
            "show_fps": true,
            "show_network_stats": false,
            "notification_duration": 5,
            "pause_game_when_open": true
        },
        "advanced": {
            "log_level": "info",
            "save_interval": 60,
            "backup_saves": true
        }
    });

    let path = game_dir.join("oracle_config.json");
    std::fs::write(&path, serde_json::to_string_pretty(&config)?)?;

    println!("✓ Generated oracle_config.json");
    Ok(())
}

fn generate_achievements_template(game_dir: &PathBuf) -> Result<()> {
    let achievements = json!({
        "app_id": 0,
        "achievements": [
            {
                "id": "ACH_FIRST_GAME",
                "name": "First Game",
                "description": "Complete your first match",
                "icon": "first_game.jpg",
                "icon_gray": "first_game_gray.jpg",
                "hidden": false
            },
            {
                "id": "ACH_WIN_10",
                "name": "Winner",
                "description": "Win 10 matches",
                "icon": "winner.jpg",
                "icon_gray": "winner_gray.jpg",
                "hidden": false
            },
            {
                "id": "ACH_WIN_100",
                "name": "Champion",
                "description": "Win 100 matches",
                "icon": "champion.jpg",
                "icon_gray": "champion_gray.jpg",
                "hidden": false
            },
            {
                "id": "ACH_PERFECT",
                "name": "Perfect",
                "description": "Complete a perfect run",
                "icon": "perfect.jpg",
                "icon_gray": "perfect_gray.jpg",
                "hidden": true
            }
        ]
    });

    let path = game_dir.join("achievements_template.json");
    std::fs::write(&path, serde_json::to_string_pretty(&achievements)?)?;

    println!("✓ Generated achievements_template.json");
    Ok(())
}

fn generate_stats_template(game_dir: &PathBuf) -> Result<()> {
    let stats = json!({
        "app_id": 0,
        "stats": {
            "int_stats": [
                {
                    "name": "games_played",
                    "default_value": 0,
                    "display_name": "Games Played"
                },
                {
                    "name": "wins",
                    "default_value": 0,
                    "display_name": "Wins"
                },
                {
                    "name": "losses",
                    "default_value": 0,
                    "display_name": "Losses"
                },
                {
                    "name": "kills",
                    "default_value": 0,
                    "display_name": "Total Kills"
                }
            ],
            "float_stats": [
                {
                    "name": "win_rate",
                    "default_value": 0.0,
                    "display_name": "Win Rate"
                },
                {
                    "name": "playtime_hours",
                    "default_value": 0.0,
                    "display_name": "Playtime (Hours)"
                }
            ]
        }
    });

    let path = game_dir.join("stats_template.json");
    std::fs::write(&path, serde_json::to_string_pretty(&stats)?)?;

    println!("✓ Generated stats_template.json");
    Ok(())
}
