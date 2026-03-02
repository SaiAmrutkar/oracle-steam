use anyhow::Result;
use parking_lot::RwLock;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub struct LicenseManager {
    db: Arc<RwLock<Connection>>,
    cache: Arc<RwLock<HashMap<(u64, u32), bool>>>,
}

impl LicenseManager {
    pub fn new() -> Result<Arc<Self>> {
        let db_path = Self::get_db_path()?;

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS licenses (
                steam_id INTEGER NOT NULL,
                app_id INTEGER NOT NULL,
                package_id INTEGER,
                granted_at INTEGER NOT NULL,
                PRIMARY KEY (steam_id, app_id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_licenses_steam_id ON licenses(steam_id)",
            [],
        )?;

        Ok(Arc::new(Self {
            db: Arc::new(RwLock::new(conn)),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }))
    }

    pub fn grant_license(&self, steam_id: u64, app_id: u32, package_id: Option<u32>) -> Result<()> {
        let db = self.db.write();

        let granted_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        db.execute(
            "INSERT OR REPLACE INTO licenses (steam_id, app_id, package_id, granted_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                steam_id as i64,
                app_id as i64,
                package_id.map(|p| p as i64),
                granted_at,
            ],
        )?;

        self.cache.write().insert((steam_id, app_id), true);

        Ok(())
    }

    pub fn has_license(&self, steam_id: u64, app_id: u32) -> bool {
        // Check cache first
        if let Some(&has_license) = self.cache.read().get(&(steam_id, app_id)) {
            return has_license;
        }

        // Check database
        let db = self.db.read();

        let result = db
            .query_row(
                "SELECT 1 FROM licenses WHERE steam_id = ?1 AND app_id = ?2",
                params![steam_id as i64, app_id as i64],
                |_| Ok(true),
            )
            .unwrap_or(false);

        self.cache.write().insert((steam_id, app_id), result);

        result
    }

    pub fn get_all_licenses(&self, steam_id: u64) -> Result<Vec<u32>> {
        let db = self.db.read();

        let mut stmt = db.prepare("SELECT app_id FROM licenses WHERE steam_id = ?1")?;

        let apps = stmt
            .query_map(params![steam_id as i64], |row| {
                let app_id: i64 = row.get(0)?;
                Ok(app_id as u32)
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(apps)
    }

    pub fn revoke_license(&self, steam_id: u64, app_id: u32) -> Result<()> {
        let db = self.db.write();

        db.execute(
            "DELETE FROM licenses WHERE steam_id = ?1 AND app_id = ?2",
            params![steam_id as i64, app_id as i64],
        )?;

        self.cache.write().remove(&(steam_id, app_id));

        Ok(())
    }

    fn get_db_path() -> Result<PathBuf> {
        let mut path = super::config::get_data_dir()?;
        path.push("licenses.db");
        Ok(path)
    }
}

static LICENSE_MANAGER: OnceLock<Arc<LicenseManager>> = OnceLock::new();

use std::sync::OnceLock;

pub fn init_licenses() -> Result<()> {
    let manager = LicenseManager::new()?;
    LICENSE_MANAGER
        .set(manager)
        .map_err(|_| anyhow::anyhow!("Licenses already initialized"))?;
    Ok(())
}

pub fn has_license(steam_id: u64, app_id: u32) -> bool {
    LICENSE_MANAGER
        .get()
        .map(|m| m.has_license(steam_id, app_id))
        .unwrap_or(false)
}

pub fn grant_license(steam_id: u64, app_id: u32) -> Result<()> {
    LICENSE_MANAGER
        .get()
        .ok_or_else(|| anyhow::anyhow!("License manager not initialized"))?
        .grant_license(steam_id, app_id, None)
}
