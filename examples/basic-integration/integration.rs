use oracle_core::{SteamClient, SteamId};

fn main() {
    println!("Oracle Steam - Basic Integration Example");

    // Create a client
    let client = SteamClient::new(76561198012345678, "TestUser".to_string());

    println!("Steam ID: {}", client.get_steam_id());
    println!("Username: {}", client.get_username());

    // Unlock an achievement
    let result = client.unlock_achievement(480, "ACH_FIRST_GAME");
    match result {
        Ok(true) => println!("✓ Achievement unlocked!"),
        Ok(false) => println!("Achievement already unlocked"),
        Err(e) => println!("Error: {}", e),
    }

    // Check achievement status
    let is_unlocked = client.is_achievement_unlocked(480, "ACH_FIRST_GAME");
    println!(
        "Achievement status: {}",
        if is_unlocked { "Unlocked" } else { "Locked" }
    );

    // Set stats
    client.set_stat_int(480, "games_played", 42);
    client.set_stat_float(480, "win_rate", 0.625);

    // Get stats
    if let Some(games) = client.get_stat_int(480, "games_played") {
        println!("Games played: {}", games);
    }

    // Create a lobby
    let lobby_id = client.create_lobby(480, 4);
    println!("Created lobby: {}", lobby_id);
}
