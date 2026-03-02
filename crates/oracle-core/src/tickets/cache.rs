use super::EncryptedAppTicket;
use anyhow::{Context, Result};
use parking_lot::RwLock;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Arc;

pub struct TicketCache {
    db: Arc<RwLock<Connection>>,
}

impl TicketCache {
    pub fn new() -> Result<Arc<Self>> {
        let db_path = Self::get_cache_path()?;

        // Ensure directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path).context("Failed to open ticket cache database")?;

        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS encrypted_tickets (
                steam_id INTEGER NOT NULL,
                app_id INTEGER NOT NULL,
                ticket_data BLOB NOT NULL,
                user_data BLOB,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (steam_id, app_id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS auth_tickets (
                handle INTEGER PRIMARY KEY,
                steam_id INTEGER NOT NULL,
                app_id INTEGER NOT NULL,
                ticket_data BLOB NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_auth_tickets_steam_id 
             ON auth_tickets(steam_id)",
            [],
        )?;

        Ok(Arc::new(Self {
            db: Arc::new(RwLock::new(conn)),
        }))
    }

    pub fn store_encrypted_ticket(
        &self,
        steam_id: u64,
        app_id: u32,
        ticket: &EncryptedAppTicket,
    ) -> Result<()> {
        let db = self.db.write();

        let created_at = ticket
            .created_at
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        db.execute(
            "INSERT OR REPLACE INTO encrypted_tickets 
             (steam_id, app_id, ticket_data, user_data, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                steam_id as i64,
                app_id as i64,
                &ticket.ticket_data,
                &ticket.user_data,
                created_at,
            ],
        )?;

        Ok(())
    }

    pub fn get_encrypted_ticket(&self, steam_id: u64, app_id: u32) -> Option<EncryptedAppTicket> {
        let db = self.db.read();

        let mut stmt = db
            .prepare(
                "SELECT ticket_data, user_data, created_at 
             FROM encrypted_tickets 
             WHERE steam_id = ?1 AND app_id = ?2",
            )
            .ok()?;

        stmt.query_row(params![steam_id as i64, app_id as i64], |row| {
            let ticket_data: Vec<u8> = row.get(0)?;
            let user_data: Vec<u8> = row.get(1)?;
            let created_at: i64 = row.get(2)?;

            Ok(EncryptedAppTicket {
                ticket_data,
                steam_id,
                app_id,
                user_data,
                created_at: std::time::UNIX_EPOCH
                    + std::time::Duration::from_secs(created_at as u64),
            })
        })
        .ok()
    }

    pub fn cleanup_expired(&self) -> Result<usize> {
        let db = self.db.write();

        let cutoff = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs()
            - 86400) as i64; // 24 hours

        let count = db.execute(
            "DELETE FROM encrypted_tickets WHERE created_at < ?1",
            params![cutoff],
        )?;

        db.execute(
            "DELETE FROM auth_tickets WHERE created_at < ?1",
            params![cutoff],
        )?;

        Ok(count)
    }

    fn get_cache_path() -> Result<PathBuf> {
        let mut path = oracle_core::config::get_data_dir()?;
        path.push("tickets.db");
        Ok(path)
    }
}

impl Clone for TicketCache {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
        }
    }
}
