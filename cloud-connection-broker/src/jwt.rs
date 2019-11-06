use serde::{Deserialize, Serialize};
use super::Error;

use std::ops::{Deref, Add};
use std::collections::BTreeMap;

use biscuit::jwa::SignatureAlgorithm;
use biscuit::{ValidationOptions, ClaimPresenceOptions, StringOrUri};
use std::sync::Arc;
use chrono::FixedOffset;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct JwtOAuthPrivateClaims {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>, // Probably the firebase User ID if set
}

pub type AuthClaimsJWE = biscuit::JWE<JwtOAuthPrivateClaims, biscuit::Empty, biscuit::Empty>;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct JWSEntry {
    #[serde(flatten)]
    pub(crate) headers: biscuit::jws::RegisteredHeader,
    #[serde(flatten)]
    pub(crate) ne: biscuit::jwk::RSAKeyParameters,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct JWKSet {
    pub keys: Vec<JWSEntry>,
}

pub struct JWKVerificator {
    pub keys: BTreeMap<String, Arc<biscuit::jws::Secret>>,
    pub expires_timestamp: i64,
    pub decrypt_key: Vec<u8>
}

impl JWKVerificator {
    /// Find the secret in the jwt set that matches the given key id, if any.
    /// Used for jws validation
    pub fn decode_secret(&self, kid: &str) -> Option<Arc<biscuit::jws::Secret>> {
        self.keys.get(kid).and_then(|f| Some(f.clone()))
    }

    /// Download a JWK Set from a given address and on success return a JWKSet
    pub async fn new(url: &str) -> Result<JWKVerificator, Error> {
        let mut resp = reqwest::get(url).await?;
        let jwk_set: JWKSet = resp.json().await?;
        let mut keys = BTreeMap::new();
        for entry in jwk_set.keys.iter() {
            if entry.headers.key_id.is_none() {
                continue;
            }

            let key_id = entry.headers.key_id.as_ref().unwrap().to_owned();
            keys.insert(key_id, Arc::new(entry.ne.jws_public_key_secret()));
        }
        Ok(JWKVerificator { keys, expires_timestamp: chrono::Utc::now().timestamp() + 60 * 60 * 24 * 7 })
    }
}

pub struct TokenValidationResult {
    pub claims: JwtOAuthPrivateClaims,
    pub subject: String,
}

impl TokenValidationResult {
    pub fn has_scope(&self, scope: &str) -> bool {
        match self.claims.scope {
            Some(ref v) => {
                v.split(" ").find(|&v| v == scope).is_some()
            }
            None => false
        }
    }
}

pub fn verify_access_token(
    verificator: &JWKVerificator,
    access_token: &str,
) -> Result<Option<TokenValidationResult>, Error> {
    let token = AuthClaimsJWE::new_encrypted(&access_token);

    let key = biscuit::jwk::JWK::new_octect_key(&verificator.decrypt_key, biscuit::Empty {});
    let mg = biscuit::jwa::KeyManagementAlgorithm::A256GCMKW;
    let ce = biscuit::jwa::ContentEncryptionAlgorithm::A256GCM;
    let (_, token) = token.into_decrypted(&key, mg, ce)?.unwrap_decrypted();

    let header = token.header()?;
    let kid = header
        .registered
        .key_id
        .as_ref();
    if kid.is_none() {
        return Ok(None);
    }
    let kid = kid.unwrap();

    let secret = verificator.decode_secret(kid);
    if secret.is_none() {
        return Ok(None);
    }
    let secret = secret.unwrap();

    let token = token.into_decoded(&secret.deref(), SignatureAlgorithm::RS256);
    if token.is_err() {
        return Ok(None);
    }

    let token = token.unwrap();

    use biscuit::Validation;
    use std::str::FromStr;

    let o = ValidationOptions {
        claim_presence_options: ClaimPresenceOptions::strict(),
        audience: Validation::Validate(StringOrUri::from_str("OHX")?),
        ..Default::default()
    };

    let claims = token.payload()?;
    claims.registered.validate(o)?;

    let subject = match claims.registered.subject.as_ref() {
        None => String::new(),
        Some(uri) => match uri {
            StringOrUri::String(s) => s.clone(),
            _ => String::new()
        }
    };

    Ok(Some(TokenValidationResult { claims: claims.private.clone(), subject }))
}

