use super::credentials::Credentials;

use serde::{Deserialize, Serialize};

use std::ops::Deref;
use chrono::Utc;
use std::collections::BTreeSet;

use biscuit::jwa::SignatureAlgorithm;
use biscuit::{ValidationOptions, ClaimPresenceOptions, StringOrUri};

use crate::tools::{scope_serialize, scope_deserialize};

type Error = failure::Error;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct JwtOAuthPrivateClaims {
    /// Scopes, separated by whitespace that this token allows access to.
    #[serde(skip_serializing_if = "BTreeSet::is_empty", deserialize_with = "scope_deserialize", serialize_with = "scope_serialize")]
    pub scope: BTreeSet<String>,
    /// The client (Addon CLI, etc) that requested this token, if set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    /// Probably the firebase User ID if set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
    /// An encryption key that can be used to allow end-to-end encryption between peers that have access
    /// tokens issued for the same "uid".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enc_key: Option<([u8; 32])>,
}

impl JwtOAuthPrivateClaims {}

pub type AuthClaimsJWT = biscuit::JWT<JwtOAuthPrivateClaims, biscuit::Empty>;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct JWSEntry {
    #[serde(flatten)]
    pub(crate) headers: biscuit::jws::RegisteredHeader,
    #[serde(flatten)]
    pub(crate) ne: biscuit::jwk::RSAKeyParameters,
}

#[derive(Serialize, Deserialize)]
pub struct JWKSetDTO {
    pub keys: Vec<JWSEntry>,
}

/// Download the Google JWK Set for a given service account.
/// The resulting set of JWKs need to be added to a credentials object
/// for jwk verifications.
pub fn download_google_jwks(account_mail: &str) -> Result<JWKSetDTO, Error> {
    let r = ureq::get(&format!(
        "https://www.googleapis.com/service_accounts/v1/jwk/{}",
        account_mail
    )).call();
    if r.ok() {
        let jwk_set: JWKSetDTO = serde_json::from_value(r.into_json()?)?;
        Ok(jwk_set)
    } else {
        Err(failure::err_msg(format!("Failed to get google jwks: {}", r.status_text())))
    }
}

pub fn create_jwt_encoded<L, T>(credentials: &Credentials, scope: Option<L>, duration: chrono::Duration,
                                client_id: Option<String>) -> Result<String, Error>
    where L: IntoIterator<Item=T>, T: AsRef<str> {
    let jwt = create_jwt(credentials, scope, duration, client_id, None, &credentials.client_email)?;
    let secret = credentials.keys.secret.as_ref().ok_or(failure::err_msg("No private key added via add_keypair_key!"))?;
    Ok(jwt.encode(&secret.deref())?.encoded()?.encode())
}

pub fn create_jwt_encoded_for_user<L, T>(credentials: &Credentials, scope: Option<L>, duration: chrono::Duration,
                                         client_id: Option<String>, user_id: String, user_email: String) -> Result<String, Error>
    where L: IntoIterator<Item=T>, T: AsRef<str> {
    let jwt = create_jwt(credentials, scope, duration, client_id, Some(user_id), &user_email)?;
    let secret = credentials.keys.secret.as_ref().ok_or(failure::err_msg("No private key added via add_keypair_key!"))?;
    Ok(jwt.encode(&secret.deref())?.encoded()?.encode())
}

pub fn create_jwt<L, T>(credentials: &Credentials, scope: Option<L>, duration: chrono::Duration,
                        client_id: Option<String>, user_id: Option<String>, user_email: &str) -> Result<AuthClaimsJWT, Error>
    where L: IntoIterator<Item=T>, T: AsRef<str> {
    use std::str::FromStr;
    use std::ops::Add;
    use uuid::Uuid;

    // Each access token requires a unique ID, so that we can revoke access tokens by that ID
    let jit = Uuid::new_v4().to_string();

    use biscuit::{
        Empty,
        jws::{Header, RegisteredHeader},
        ClaimsSet, RegisteredClaims, SingleOrMultiple, JWT,
    };

    let header: Header<Empty> = Header::from(RegisteredHeader {
        algorithm: SignatureAlgorithm::RS256,
        key_id: Some(credentials.private_key_id.to_owned()),
        ..Default::default()
    });
    let expected_claims = ClaimsSet::<JwtOAuthPrivateClaims> {
        registered: RegisteredClaims {
            issuer: Some(FromStr::from_str(&credentials.client_email)?),
            audience: Some(SingleOrMultiple::Single(StringOrUri::from_str("OHX")?)),
            subject: Some(StringOrUri::from_str(user_email)?),
            expiry: Some(biscuit::Timestamp::from(Utc::now().add(duration))),
            not_before: Some(biscuit::Timestamp::from(Utc::now())),
            issued_at: Some(biscuit::Timestamp::from(Utc::now())),
            id: Some(jit),
        },
        private: JwtOAuthPrivateClaims {
            scope: scope.and_then(|f| Some(f.into_iter().map(|f| f.as_ref().to_owned()).collect())).unwrap_or_default(),
            client_id,
            uid: user_id,
            enc_key: None,
        },
    };
    Ok(JWT::new_decoded(header, expected_claims))
}

pub struct TokenValidationResult {
    pub claims: JwtOAuthPrivateClaims,
    pub subject: String,
}

pub fn verify_access_token(
    credentials: &Credentials,
    access_token: &str,
) -> Result<Option<TokenValidationResult>, Error> {
    let token = AuthClaimsJWT::new_encoded(&access_token);

    let header = token.unverified_header()?;
    let kid = header
        .registered
        .key_id
        .as_ref();
    if kid.is_none() {
        return Ok(None);
    }
    let kid = kid.unwrap();

    let secret = credentials
        .decode_secret(kid);
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

