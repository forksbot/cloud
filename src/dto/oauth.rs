use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use rocket::request::{FromForm, LenientForm};
use rocket::response::{content, Redirect};
use rocket::Responder;

pub const SCOPE_OFFLINE_ACCESS: &str = "offline_access";

#[derive(UriDisplayQuery, FromForm)]
pub struct GenerateCodeDTO {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub response_type: String, // "code" or "device"
    pub scope: Option<String>, // offline_access -> return a refresh_token
    pub state: Option<String>,
}

pub type GenerateTokenRequest = LenientForm<GenerateCodeDTO>;

use rocket::UriDisplayQuery;

#[derive(Deserialize, UriDisplayQuery)]
pub struct AuthPageRedirectUri {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub response_type: String, // "code" or "device"
    pub scope: Option<String>, // offline_access -> return a refresh_token
    pub state: Option<String>,
    // Add
    pub code: String,
    pub unsigned: String,
}

#[derive(Default, Deserialize, Serialize, FromForm, UriDisplayQuery)]
pub struct TokenDTO {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,          // for grant_type "authorization_code"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>, // for grant_type "refresh_token"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,      // for grant_type "password"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,      // for grant_type "password"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_code: Option<String>, // for grant_type "urn:ietf:params:oauth:grant-type:device_code"

    pub client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    pub grant_type: String,
}

pub type TokenRequest = LenientForm<TokenDTO>;

#[derive(Deserialize, Serialize)]
pub struct GrantRequest {
    pub unsigned: String,
    pub code: String,
    pub scopes: HashSet<String>,
}

#[derive(Serialize, Deserialize)]
struct RevokeDTO {
    pub client_id: String,
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DeviceFlowResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub interval: u32,
    pub expires_in: u32,
}

#[derive(Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String, // "bearer"
    pub expires_in: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    pub scope: String, // Space delimiter
}

impl OAuthTokenResponse {
    pub fn new(
        access_token: String,
        refresh_token: Option<String>,
        scopes: HashSet<String>,
    ) -> Self {
        use crate::tools::JoinableIterator;
        OAuthTokenResponse {
            access_token,
            refresh_token,
            expires_in: 3600,
            token_type: "bearer".to_string(),
            scope: scopes.iter().join(" "),
        }
    }
}

#[derive(Responder)]
pub enum RedirectOrResponseAuthorize {
    Json(content::Json<String>),
    ToOhxLoginPage(Redirect),
}
