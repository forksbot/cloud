//! # Oauth Clients
//! OAuth clients are read in on application start. Only registered clients (client_id, [client_secret])
//! can use this oauth service.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub type OAuthClients = HashMap<String, OAuthClient>;

pub fn new(json: &str) -> Result<OAuthClients, failure::Error> {
    Ok(serde_json::from_str(json)?)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OAuthClient {
    pub id: String,
    pub secret: Option<String>,
    pub title: String,
    pub author: String,
    pub logo_url: String,
    #[serde(default)]
    pub redirect_uri: Vec<String>,
    #[serde(default)]
    pub scopes: HashSet<String>,
}
