use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RatingsInDB {
    pub uid: String,
    pub token: String,
    pub client_name: String,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub issued_at: i64,
}

#[derive(Serialize, Deserialize)]
pub struct UserEntry {
    pub braintree_customer_id: Option<String>,
}
