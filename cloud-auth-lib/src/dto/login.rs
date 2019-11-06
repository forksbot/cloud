use serde::{Deserialize, Serialize};
use chrono::{Utc, DateTime};
use std::collections::BTreeSet;

#[allow(unused_imports)]
use crate::tools::{scope_deserialize,scope_serialize};

#[derive(Deserialize, Serialize)]
pub struct UserSession {
    pub refresh_token: Option<String>,
    pub access_token: String,
    pub access_token_expires: DateTime<Utc>,
    pub client_id: String,
    pub user_id: String,
    pub user_email: String,
    pub user_display_name: String,
}

#[derive(Serialize)]
pub struct TokenRequestForRefreshToken {
    refresh_token: String,
    client_id: String,
    grant_type: String,
}

#[derive(Serialize)]
pub struct TokenRequestForDevice {
    device_code: String,
    client_id: String,
    grant_type: String,
}

#[derive(Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    // "bearer"
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    /// The scope field is usually a space separated list of scopes. A special serializer/deserializer
    /// converts this into a set.
    #[serde(skip_serializing_if = "BTreeSet::is_empty", deserialize_with = "scope_deserialize", serialize_with = "scope_serialize")]
    pub scope: BTreeSet<String>,
}

#[derive(Serialize)]
pub(crate) struct AuthRequest {
    pub client_id: String,
    pub client_name: String,
    pub response_type: String,
    pub scope: String,
}

#[derive(Deserialize)]
pub struct DeviceFlowResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub interval: u32,
    pub expires_in: i64,
}

#[derive(Deserialize)]
pub struct ErrorResult {
    pub error: String,
}

impl From<String> for ErrorResult {
    fn from(message: String) -> Self {
        serde_json::from_str(&message).expect("extracting json from a 400 error")
    }
}
