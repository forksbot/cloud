use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub type OAuthClients = HashMap<String, OAuthClient>;

pub fn new(json: &str) -> Result<OAuthClients, failure::Error> {
    Ok(serde_json::from_str(json)?)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OAuthClient {
    pub id: String,
    pub title: String,
    pub author: String,
    pub logo_url: String,
    pub redirect_uri: Vec<String>,
    pub scopes: HashSet<String>,
}
