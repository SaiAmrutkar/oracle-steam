use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

const AVATAR_SMALL: i32 = 1;
const AVATAR_MEDIUM: i32 = 2;
const AVATAR_LARGE: i32 = 3;

pub struct AvatarManager {
    avatars: Arc<RwLock<HashMap<u64, AvatarSet>>>,
    next_handle: Arc<RwLock<i32>>,
}

#[derive(Debug, Clone)]
struct AvatarSet {
    small: i32,
    medium: i32,
    large: i32,
}

impl AvatarManager {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            avatars: Arc::new(RwLock::new(HashMap::new())),
            next_handle: Arc::new(RwLock::new(1000)),
        })
    }

    pub fn get_small_avatar(&self, steamid: u64) -> i32 {
        let avatars = self.avatars.read();
        avatars
            .get(&steamid)
            .map(|a| a.small)
            .unwrap_or_else(|| self.create_default_avatar(steamid, AVATAR_SMALL))
    }

    pub fn get_medium_avatar(&self, steamid: u64) -> i32 {
        let avatars = self.avatars.read();
        avatars
            .get(&steamid)
            .map(|a| a.medium)
            .unwrap_or_else(|| self.create_default_avatar(steamid, AVATAR_MEDIUM))
    }

    pub fn get_large_avatar(&self, steamid: u64) -> i32 {
        let avatars = self.avatars.read();
        avatars
            .get(&steamid)
            .map(|a| a.large)
            .unwrap_or_else(|| self.create_default_avatar(steamid, AVATAR_LARGE))
    }

    pub fn load_avatar(&self, steamid: u64, size: i32, image_data: Vec<u8>) -> i32 {
        let handle = self.allocate_handle();

        let mut avatars = self.avatars.write();
        let avatar_set = avatars.entry(steamid).or_insert_with(|| AvatarSet {
            small: 0,
            medium: 0,
            large: 0,
        });

        match size {
            AVATAR_SMALL => avatar_set.small = handle,
            AVATAR_MEDIUM => avatar_set.medium = handle,
            AVATAR_LARGE => avatar_set.large = handle,
            _ => {}
        }

        // Store image data in oracle-storage
        let _ = self.store_avatar_data(handle, image_data);

        handle
    }

    fn create_default_avatar(&self, steamid: u64, size: i32) -> i32 {
        let handle = self.allocate_handle();

        let mut avatars = self.avatars.write();
        let avatar_set = avatars.entry(steamid).or_insert_with(|| AvatarSet {
            small: 0,
            medium: 0,
            large: 0,
        });

        match size {
            AVATAR_SMALL => avatar_set.small = handle,
            AVATAR_MEDIUM => avatar_set.medium = handle,
            AVATAR_LARGE => avatar_set.large = handle,
            _ => {}
        }

        handle
    }

    fn allocate_handle(&self) -> i32 {
        let mut handle = self.next_handle.write();
        let current = *handle;
        *handle += 1;
        current
    }

    fn store_avatar_data(&self, handle: i32, data: Vec<u8>) -> anyhow::Result<()> {
        // Store in oracle-storage for later retrieval
        oracle_storage::store_avatar(handle, data)?;
        Ok(())
    }
}
