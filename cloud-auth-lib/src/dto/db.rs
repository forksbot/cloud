use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Serialize, Deserialize)]
pub struct AccessTokenInDB {
    pub uid: String,
    pub token: String,
    pub client_id: String,
    pub scopes: BTreeSet<String>,
    pub issued_at: i64,
}
