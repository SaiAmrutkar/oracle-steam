// tools/ticket-generator/src/main.rs
// CLI tool for generating and managing Steam encrypted tickets

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, Select};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "Oracle Steam Ticket Generator")]
#[command(about = "Generate and manage Steam encrypted app tickets", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract ticket from running Steam client
    Extract {
        /// App ID to generate ticket for
        #[arg(short, long)]
        app_id: u32,
        
        /// Output file (defaults to configs.user.ini)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Generate ticket via Oracle Steam backend
    Generate {
        /// App ID
        #[arg(short, long)]
        app_id: u32,
        
        /// Steam username
        #[arg(short, long)]
        username: String,
        
        /// Steam password
        #[arg(short, long)]
        password: String,
        
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Show ticket cache statistics
    Stats {
        /// Data directory
        #[arg(short, long, default_value = "oracle_data")]
        data_dir: PathBuf,
    },
    
    /// Interactive mode
    Interactive,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Extract { app_id, output } => {
            extract_from_steam(app_id, output).await?;
        }
        Commands::Generate { app_id, username, password, output } => {
            generate_via_oracle(app_id, &username, &password, output).await?;
        }
        Commands::Stats { data_dir } => {
            show_stats(data_dir).await?;
        }
        Commands::Interactive => {
            interactive_mode().await?;
        }
    }

    Ok(())
}

async fn extract_from_steam(app_id: u32, output: Option<PathBuf>) -> Result<()> {
    println!("╔════════════════════════════════════════╗");
    println!("║  Oracle Steam - Ticket Extractor       ║");
    println!("╚════════════════════════════════════════╝\n");

    println!("🎮 App ID: {}", app_id);
    println!("📋 Extracting ticket from Steam client...\n");

    // Check if Steam is running
    print!("⏳ Waiting for Steam client... ");
    
    let (steam_id, ticket) = oracle_core::ticket_manager::extract_ticket_from_steam(app_id)
        .await
        .context("Failed to extract ticket. Is Steam running and do you own this game?")?;

    println!("✅\n");
    println!("Steam ID: {}", steam_id);
    println!("Ticket length: {} bytes", 
             base64::engine::general_purpose::STANDARD.decode(&ticket)?.len());
    println!("\n{}", "═".repeat(50));
    println!("Encrypted Ticket:");
    println!("{}", "═".repeat(50));
    println!("{}", ticket);
    println!("{}\n", "═".repeat(50));

    // Save to file
    let output_path = output.unwrap_or_else(|| PathBuf::from("configs.user.ini"));
    
    let save = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Save to {}?", output_path.display()))
        .default(true)
        .interact()?;

    if save {
        save_config(&output_path, steam_id, &ticket)?;
        println!("✅ Saved to {}", output_path.display());
    }

    println!("\n💡 Ticket valid for: ~15 minutes (Denuvo games)");
    println!("📊 Daily limit: 5 tickets per game\n");

    Ok(())
}

async fn generate_via_oracle(
    app_id: u32,
    username: &str,
    password: &str,
    output: Option<PathBuf>,
) -> Result<()> {
    println!("╔════════════════════════════════════════╗");
    println!("║  Oracle Steam - Ticket Generator       ║");
    println!("╚════════════════════════════════════════╝\n");

    println!("🎮 App ID: {}", app_id);
    println!("👤 Username: {}", username);
    println!("🔒 Connecting to Steam servers...\n");

    // Initialize Oracle Steam CM client
    let mut cm_client = oracle_steamvent::SteamCMClient::new();
    
    print!("📡 Connecting... ");
    cm_client.connect().await
        .context("Failed to connect to Steam CM servers")?;
    println!("✅");

    print!("🔐 Authenticating... ");
    let steam_id = cm_client.login(username, password).await
        .context("Login failed. Check credentials.")?;
    println!("✅");
    
    println!("   Steam ID: {}", steam_id);

    print!("🎫 Requesting ticket... ");
    let ticket_data = cm_client.request_encrypted_app_ticket(app_id).await
        .context("Failed to get ticket. Does account own this game?")?;
    println!("✅\n");

    // Encode ticket
    use base64::Engine as _;
    let ticket = base64::engine::general_purpose::STANDARD.encode(&ticket_data);

    println!("{}", "═".repeat(50));
    println!("Encrypted Ticket:");
    println!("{}", "═".repeat(50));
    println!("{}", ticket);
    println!("{}\n", "═".repeat(50));

    // Save
    let output_path = output.unwrap_or_else(|| PathBuf::from("configs.user.ini"));
    save_config(&output_path, steam_id, &ticket)?;
    
    println!("✅ Saved to {}", output_path.display());
    println!("\n💡 Ticket valid for: ~15 minutes (Denuvo games)");
    println!("📊 Daily limit: 5 tickets per game\n");

    Ok(())
}

async fn show_stats(data_dir: PathBuf) -> Result<()> {
    println!("╔════════════════════════════════════════╗");
    println!("║  Oracle Steam - Ticket Statistics      ║");
    println!("╚════════════════════════════════════════╝\n");

    let manager = oracle_core::TicketManager::new(data_dir);
    manager.load_from_disk().await?;

    let tickets = manager.tickets.read().await;

    if tickets.is_empty() {
        println!("No cached tickets found.\n");
        return Ok(());
    }

    println!("App ID  │ Today │ Remaining │ Active │ History");
    println!("{}", "─".repeat(50));

    for (app_id, cache) in tickets.iter() {
        let stats = manager.get_stats(*app_id).await.unwrap();
        
        println!("{:<7} │ {:<5} │ {:<9} │ {:<6} │ {}",
                 app_id,
                 stats.generated_today,
                 stats.remaining_today,
                 if stats.has_active_ticket { "Yes" } else { "No" },
                 stats.history_count);
    }

    println!();

    Ok(())
}

async fn interactive_mode() -> Result<()> {
    println!("╔════════════════════════════════════════╗");
    println!("║  Oracle Steam - Interactive Mode       ║");
    println!("╚════════════════════════════════════════╝\n");

    let choices = &[
        "Extract from Steam client",
        "Generate via Oracle backend",
        "View cached tickets",
        "Exit",
    ];

    loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .items(choices)
            .default(0)
            .interact()?;

        match selection {
            0 => {
                // Extract
                let app_id: u32 = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter App ID")
                    .interact()?;

                if let Err(e) = extract_from_steam(app_id, None).await {
                    eprintln!("\n❌ Error: {}\n", e);
                }
            }
            1 => {
                // Generate
                let app_id: u32 = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter App ID")
                    .interact()?;

                let username: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Steam Username")
                    .interact()?;

                let password: String = dialoguer::Password::with_theme(&ColorfulTheme::default())
                    .with_prompt("Steam Password")
                    .interact()?;

                if let Err(e) = generate_via_oracle(app_id, &username, &password, None).await {
                    eprintln!("\n❌ Error: {}\n", e);
                }
            }
            2 => {
                // Stats
                if let Err(e) = show_stats(PathBuf::from("oracle_data")).await {
                    eprintln!("\n❌ Error: {}\n", e);
                }
            }
            3 => {
                // Exit
                println!("\nGoodbye! 👋\n");
                break;
            }
            _ => unreachable!(),
        }

        println!();
    }

    Ok(())
}

fn save_config(path: &PathBuf, steam_id: u64, ticket: &str) -> Result<()> {
    use std::io::Write;
    
    let mut file = std::fs::File::create(path)?;
    
    writeln!(file, "[user::general]")?;
    writeln!(file, "account_steamid={}", steam_id)?;
    writeln!(file, "ticket={}", ticket)?;
    
    Ok(())
}