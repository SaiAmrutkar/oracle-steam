use anyhow::Result;
use std::io::{self, Write};

fn main() -> Result<()> {
    println!("╔════════════════════════════════════════╗");
    println!("║  Oracle Steam - App ID Finder          ║");
    println!("╚════════════════════════════════════════╝\n");

    print_menu();

    loop {
        print!("\nEnter choice (1-4): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" => search_by_name()?,
            "2" => find_from_executable()?,
            "3" => list_common_games(),
            "4" => break,
            _ => println!("Invalid choice. Please enter 1-4."),
        }
    }

    println!("\nGoodbye!");
    Ok(())
}

fn print_menu() {
    println!("Options:");
    println!("  1. Search by game name");
    println!("  2. Find from game executable");
    println!("  3. List common games");
    println!("  4. Exit");
}

fn search_by_name() -> Result<()> {
    print!("\nEnter game name: ");
    io::stdout().flush()?;

    let mut game_name = String::new();
    io::stdin().read_line(&mut game_name)?;
    let game_name = game_name.trim();

    if game_name.is_empty() {
        println!("Game name cannot be empty.");
        return Ok(());
    }

    println!("\nSearching for '{}'...", game_name);
    println!("\n⚠ Note: This requires internet connection to Steam's database.");
    println!("For offline use, try option 3 for common games.\n");

    // In a real implementation, this would query SteamDB API or scrape Steam store
    // For now, we'll show manual search instructions
    println!("Manual search instructions:");
    println!("1. Visit: https://steamdb.info/");
    println!("2. Search for: {}", game_name);
    println!("3. Look for the 'App ID' in the search results");
    println!("\nAlternatively, visit the game's Steam store page.");
    println!("The App ID is in the URL: store.steampowered.com/app/[APP_ID]/");

    Ok(())
}

fn find_from_executable() -> Result<()> {
    print!("\nEnter path to game executable: ");
    io::stdout().flush()?;

    let mut exe_path = String::new();
    io::stdin().read_line(&mut exe_path)?;
    let exe_path = exe_path.trim();

    if exe_path.is_empty() {
        println!("Path cannot be empty.");
        return Ok(());
    }

    let path = std::path::Path::new(exe_path);

    if !path.exists() {
        println!("Error: File does not exist: {}", exe_path);
        return Ok(());
    }

    let dir = path.parent().unwrap_or(path);

    // Check for steam_appid.txt
    let steam_appid_path = dir.join("steam_appid.txt");
    if steam_appid_path.exists() {
        match std::fs::read_to_string(&steam_appid_path) {
            Ok(content) => {
                let app_id = content.trim();
                println!("\n✓ Found steam_appid.txt");
                println!("  App ID: {}", app_id);
                return Ok(());
            }
            Err(e) => {
                println!("Error reading steam_appid.txt: {}", e);
            }
        }
    }

    println!("\n⚠ No steam_appid.txt found in game directory.");
    println!("Try searching by game name (option 1).");

    Ok(())
}

fn list_common_games() {
    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║              Common Steam Game App IDs                  ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    let common_games = vec![
        ("Counter-Strike 2", 730),
        ("Dota 2", 570),
        ("Team Fortress 2", 440),
        ("Left 4 Dead 2", 550),
        ("Portal 2", 620),
        ("Half-Life 2", 220),
        ("Garry's Mod", 4000),
        ("Rust", 252490),
        ("ARK: Survival Evolved", 346110),
        ("Terraria", 105600),
        ("Stardew Valley", 413150),
        ("Don't Starve Together", 322330),
        ("Dead by Daylight", 381210),
        ("PAYDAY 2", 218620),
        ("GTA V", 271590),
        ("Elden Ring", 1245620),
        ("Dark Souls III", 374320),
        ("Sekiro", 814380),
        ("Cyberpunk 2077", 1091500),
        ("The Witcher 3", 292030),
        ("Skyrim Special Edition", 489830),
        ("Fallout 4", 377160),
        ("Red Dead Redemption 2", 1174180),
        ("Monster Hunter: World", 582010),
        ("Resident Evil Village", 1196590),
        ("Valheim", 892970),
        ("Among Us", 945360),
        ("Fall Guys", 1097150),
        ("Phasmophobia", 739630),
        ("Lethal Company", 1966720),
        ("Satisfactory", 526870),
        ("Factorio", 427520),
        ("Rimworld", 294100),
        ("Project Zomboid", 108600),
        ("7 Days to Die", 251570),
        ("Spacewar (Test)", 480),
    ];

    for (name, app_id) in common_games {
        println!("  {:<40} {}", name, app_id);
    }

    println!("\n💡 Tip: Use these App IDs with:");
    println!("   oracle install \"C:/Games/YourGame\" <APP_ID>");
}
