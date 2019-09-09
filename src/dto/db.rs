use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AccessTokenInDB {
    pub uid: String,
    pub token: String,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub issued_at: i64,
}
