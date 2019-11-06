use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RatingsInDB {
    pub rate: i64,
    pub last_rating: i64,
    pub addon_id: String
}

#[derive(Serialize, Deserialize)]
pub struct DownloadsInDB {
    pub installed: i64, // +1: Installed, -1: Removed, 0: Installed then removed
    pub addon_id: String
}
