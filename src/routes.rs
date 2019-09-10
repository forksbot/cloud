// own
use crate::dto::db;
use crate::dto::oauth::*;
use crate::responder_type::MyResponder;
use crate::token::{decrypt_unsigned_jwt_token, encrypt_unsigned_jwt_token, hash_of_token};
use crate::oauth_clients::OAuthClients;
use crate::tools::JoinableIterator;

// External, controlled libraries
use cloud_vault::{
    credentials::Credentials, guard_oauth_jwt_access, guard_rate_limiter::RateLimiter, jwt,
};
use firestore_db_and_auth::{
    rocket::FirestoreAuthSessionGuard, sessions::service_account::Session as SASession, documents,
};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

// External libraries
use redis::Commands;
use rocket::http::RawStr;
use rocket::response::content;
use rocket::{get, post};

// std
use biscuit::jwa::SignatureAlgorithm;
use chrono::Duration;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::Mutex;
use rocket_contrib::json::Json;

const CREDENTIALS_GOOGLE_SERVICE_ACCOUNT_INDEX: usize = 0;
const CREDENTIALS_OHX_SERVICE_ACCOUNT_INDEX: usize = 1;

/// Empty default route
#[get("/")]
pub fn index() -> &'static str {
    ""
}

#[get("/.well-known/jwks.json")]
pub fn pubkey_jwk() -> &'static str {
    include_str!("../secrets/ohx_oauth_key.json")
}

#[get("/.well-known/openid-configuration")]
pub fn openid_configuration() -> &'static str {
    include_str!("../openid-configuration.json")
}

/// Refresh access token environment variable on travis-ci
/// for all the different repositories (listed in repositories.json).
///
/// A token is valid for 6 hours. A cron job must call this endpoint periodically.
#[get("/check_for_users")]
pub fn check_for_users(
    oauth_user: guard_oauth_jwt_access::OAuthIdentity,
    firebase: rocket::State<Mutex<SASession>>,
) -> Result<String, MyResponder> {
    // Only the google account is allowed to call this endpoint
    if oauth_user.credentials_index != CREDENTIALS_GOOGLE_SERVICE_ACCOUNT_INDEX {
        return Err(MyResponder::AccessScopeInsufficient(
            "Only the google CI account is allowed to call this endpoint".to_owned(),
        ));
    }

    use firestore_db_and_auth::dto;
    let timestamp: i64 = chrono::Utc::now().timestamp_millis() - 1000 * 60 * 60;

    let session_mutex = firebase.lock()?;
    let session: &SASession = session_mutex.deref();

    let result = documents::query(session, "users", timestamp.into(), dto::FieldOperator::LESS_THAN_OR_EQUAL, "queued_remove")?;
    for metadata in result {
        let name = documents::abs_to_rel(metadata.name.as_ref().unwrap());
        let user_id = &name[name.rfind("/").unwrap() + 1..];
        info!("FOUND DOCUMENT {} - {}", name, user_id);
        let user_session = firestore_db_and_auth::UserSession::by_user_id(&session.credentials, user_id, false);
        match user_session {
            Ok(user_session) => {
                match firestore_db_and_auth::users::user_remove(&user_session) {
                    Ok(_) => {
                        documents::delete(session, name, false)?;
                    }
                    Err(e) => {
                        error!("Could not delete user {}. {:?}", user_id, e);
                    }
                }
            }
            Err(e) => {
                error!("Could not delete user {}. {:?}", user_id, e);
            }
        };
    }
    Ok(String::new())
}

#[get("/check_for_users", rank = 2)]
pub fn check_for_users_unauthorized() -> MyResponder {
    MyResponder::AccessScopeInsufficient("Requires authorization".to_owned())
}

/// Deserialize type T and return a "content::Json" rocket response
fn con_json<T>(t: &T) -> Result<content::Json<String>, MyResponder>
    where
        T: serde::Serialize,
{
    Ok(content::Json(serde_json::to_string(t)?))
}

pub fn return_token_response(
    redis: &redis::Client,
    session: &SASession,
    credentials: &Credentials,
    client_id: String,
    code: Option<String>,
) -> Result<content::Json<String>, MyResponder> {
    if code.is_none() {
        return Err(MyResponder::bad_request("Code invalid"));
    }
    let code = code.unwrap();
    let mut c = redis.get_connection()?;

    let jwt_token: Option<String> = c.get(&code)?;
    if jwt_token.is_none() {
        return Err(MyResponder::bad_request("Code invalid"));
    }
    let jwt_token = jwt_token.unwrap();

    if let Some(token_result) = jwt::verify_access_token(&credentials, &jwt_token)? {
        if token_result.claims.uid.is_none() {
            return Err(MyResponder::bad_request("Access token has no user_id!"));
        }
        let uid = token_result.claims.uid.as_ref().unwrap().clone();

        let scopes = token_result.get_scopes();
        return if scopes.contains(SCOPE_OFFLINE_ACCESS) {
            // Create access token (same as refresh token but without the SCOPE_OFFLINE_ACCESS scope
            // and with only 1h expiry time
            let access_token = jwt::create_jwt_encoded_for_user(
                &credentials,
                Some(scopes.iter().filter(|f| f.as_str() != SCOPE_OFFLINE_ACCESS)),
                Duration::hours(1),
                Some(client_id.clone()),
                uid.clone(),
                token_result.subject.clone(),
            )?;

            // Write refresh token to database. Can be revoked by the user (== deleted) and is used
            // by the token endpoint to create new access_tokens.
            documents::write(
                session,
                "access_tokens",
                Some(&code),
                &db::AccessTokenInDB {
                    uid,
                    client_id,
                    token: jwt_token.clone(),
                    scopes: token_result.get_scopes().into_iter().collect(),
                    issued_at: chrono::Utc::now().timestamp(),
                },
                documents::WriteOptions::default(),
            )?;

            c.del(&code)?;
            con_json(&OAuthTokenResponse::new(
                access_token,
                Some(jwt_token),
                scopes,
            ))
        } else {
            c.del(&code)?;
            con_json(&OAuthTokenResponse::new(jwt_token, None, scopes))
        };
    }
    return Err(MyResponder::bad_request("Unexpected"));
}

#[post("/grant_scopes", format = "application/json", data = "<request>")]
pub fn grant_scopes(
    request: Json<GrantRequest>,
    redis: rocket::State<redis::Client>,
    firestore_auth: FirestoreAuthSessionGuard,
    credentials_list: rocket::State<Vec<Credentials>>,
    _rate_limiter: RateLimiter,
) -> Result<String, MyResponder> {
    let credentials = credentials_list
        .get(CREDENTIALS_OHX_SERVICE_ACCOUNT_INDEX)
        .unwrap();
    let secret = credentials.encode_secret().ok_or(MyResponder::InternalError("No private key found!".into()))?;

    let mut jwt = decrypt_unsigned_jwt_token(&request.unsigned.as_bytes())?;

    // Fix signature
    jwt.header_mut()?.registered.algorithm = SignatureAlgorithm::RS256;

    let mut payload = jwt.payload_mut()?;

    // Fix user_id
    payload.private.uid = Some(firestore_auth.0.user_id);

    // Fix scopes
    let requested_scopes: HashSet<String> = payload
        .private
        .scope
        .clone()
        .unwrap_or(String::new())
        .split(" ")
        .filter(|f| !f.is_empty())
        .map(|f| f.to_owned())
        .collect();

    payload.private.scope = Some(requested_scopes.intersection(&request.scopes).join(" "));

    // Sign
    let jwt = jwt.encode(&secret.deref())?.encoded()?.encode();

    let mut c = redis.get_connection()?;
    c.set_nx(&request.code, &jwt)?;
    c.expire(&request.code, 360)?;
    Ok(request.code.clone())
}

#[post("/grant_scopes", rank = 2)]
pub fn grant_scopes_unauthorized() -> MyResponder {
    MyResponder::AccessScopeInsufficient("Not authorized!".into())
}

/// Exchange
#[post("/token", data = "<token_request>")]
pub fn token(
    token_request: TokenRequest,
    redis: rocket::State<redis::Client>,
    firebase: rocket::State<Mutex<SASession>>,
    credentials_list: rocket::State<Vec<Credentials>>,
    _rate_limiter: RateLimiter,
) -> Result<content::Json<String>, MyResponder> {
    let credentials = credentials_list
        .get(CREDENTIALS_OHX_SERVICE_ACCOUNT_INDEX)
        .unwrap();

    let session_mutex = firebase.lock()?;
    let session: &SASession = session_mutex.deref();

    match &token_request.grant_type[..] {
        "urn:ietf:params:oauth:grant-type:device_code" => {
            return_token_response(&redis, &session, &credentials, token_request.0.client_id, token_request.0.device_code)
        }
        "authorization_code" => {
            return_token_response(&redis, &session, &credentials, token_request.0.client_id, token_request.0.code)
        }
        "refresh_token" => {
            if token_request.refresh_token.is_none() {
                return Err(MyResponder::bad_request("You must provide a refresh_token"));
            }
            let code = hash_of_token(token_request.refresh_token.as_ref().unwrap().as_bytes());
            let db_entry: db::AccessTokenInDB = firestore_db_and_auth::documents::read(session, "access_tokens", code).map_err(|_e| MyResponder::bad_request("Access Token not valid. It may have been revoked!"))?;
            // Filter out offline scope and create access token
            let access_token = jwt::create_jwt_encoded_for_user(&credentials, Some(db_entry.scopes.iter().filter(|f| f.as_str() != SCOPE_OFFLINE_ACCESS)),
                                                                Duration::hours(1),
                                                                Some(db_entry.client_id.clone()), db_entry.uid.clone(), credentials.client_email.clone())?;

            con_json(&OAuthTokenResponse::new(access_token, Some(db_entry.token), db_entry.scopes.clone().into_iter().collect()))
        }
        _ => Err(MyResponder::bad_request("grant_type must be authorization_code, refresh_token or urn:ietf:params:oauth:grant-type:device_code"))
    }
}

/// Create a code for an authorized firestore user that can be changed to access tokens
#[post("/authorize", data = "<request>")]
pub fn authorize(
    request: GenerateTokenRequest,
    credentials_list: rocket::State<Vec<Credentials>>,
    client_data: rocket::State<OAuthClients>,
    _rate_limiter: RateLimiter,
) -> Result<RedirectOrResponseAuthorize, MyResponder> {
    use rocket::http::uri::{Query, UriDisplay};
    use rocket::response::Redirect;

    // Check if client_id is valid
    let client_data = match client_data.get(&request.client_id) {
        Some(c) => c,
        None => return Err(MyResponder::bad_request("client_id unknown"))
    };

    let credentials = credentials_list
        .get(CREDENTIALS_OHX_SERVICE_ACCOUNT_INDEX)
        .unwrap();

    let scopes: HashSet<String> = match request.0.scope {
        Some(ref v) => v.split(" ").filter(|f| !f.is_empty()).map(|f| f.to_owned()).collect(),
        None => HashSet::new(),
    };

    // Check scopes: Only those defined in oauth_clients.json are allowed
    if !scopes.is_subset(&client_data.scopes) {
        return Err(MyResponder::bad_request("Requested scopes are invalid"));
    }

    let duration = match scopes.contains(SCOPE_OFFLINE_ACCESS) {
        true => chrono::Duration::weeks(52 * 10),
        false => chrono::Duration::hours(1),
    };

    // Create a token without signature
    let jwt = jwt::create_jwt(
        &credentials,
        Some(scopes),
        duration,
        Some(request.client_id.clone()),
        None, &credentials.client_email,
    )?;

    let unsigned = encrypt_unsigned_jwt_token(jwt)?;

    let message = AuthPageRedirectUri {
        client_id: request.client_id.clone(),
        client_secret: request.client_secret.clone(),
        client_name: request.client_name.clone(),
        redirect_uri: request.redirect_uri.clone(),
        response_type: request.response_type.clone(),
        scope: request.scope.as_ref().and_then(|f| Some(f.trim().to_owned())),
        state: request.state.clone(),
        code: hash_of_token(&unsigned.as_bytes()),
        unsigned,
    };
    let uri = format!(
        "https://openhabx.com/auth?{}",
        &message as &dyn UriDisplay<Query>
    );

    match &request.response_type[..] {
        "code" => {
            return Ok(RedirectOrResponseAuthorize::ToOhxLoginPage(Redirect::to(uri)));
        }
        "device" => {
            Ok(RedirectOrResponseAuthorize::Json(content::Json(
                serde_json::to_string(&DeviceFlowResponse {
                    user_code: hash_of_token(&message.unsigned.as_bytes()),
                    verification_uri: uri,
                    interval: 2,
                    device_code: message.unsigned,
                    expires_in: 3600,
                })?,
            )))
        }
        _ => Err(MyResponder::bad_request("invalid response_type")),
    }
}

/// This is a rate limited endpoint to revoke an auth token
#[get("/revoke?<token>", rank = 2)]
pub fn revoke_by_oauth(
    token: &RawStr,
    firebase: rocket::State<Mutex<SASession>>,
    oauth_user: guard_oauth_jwt_access::OAuthIdentity,
    _rate_limiter: RateLimiter,
) -> Result<(), MyResponder> {
    // Only the google account is allowed to call this endpoint
    if oauth_user.credentials_index != CREDENTIALS_GOOGLE_SERVICE_ACCOUNT_INDEX {
        return Err(MyResponder::AccessScopeInsufficient(
            "Only the google CI account is allowed to call this endpoint".to_owned(),
        ));
    }

    let session_mutex = firebase.lock()?;
    let code = hash_of_token(token.as_str().as_bytes());
    firestore_db_and_auth::documents::delete(
        session_mutex.deref(),
        &format!("access_tokens/{}", code),
        false,
    )?;

    Ok(())
}

/// This is a rate limited endpoint to poll for an auth token
#[get("/userinfo?<user_id>")]
pub fn user_info(
    user_id: &RawStr,
    oauth_user: guard_oauth_jwt_access::OAuthIdentity,
    firebase: rocket::State<Mutex<SASession>>,
    _rate_limiter: RateLimiter,
) -> Result<content::Json<String>, MyResponder> {
    if !oauth_user.scopes.contains("profile") {
        return Err(MyResponder::AccessScopeInsufficient(
            "profile scope required!".into(),
        ));
    }

    let session_mutex = firebase.lock()?;
    let session: &SASession = session_mutex.deref();

    let user_session = firestore_db_and_auth::UserSession::by_user_id(
        &session.credentials,
        user_id.as_str(),
        false,
    )?;

    let info = firestore_db_and_auth::users::user_info(&user_session)?;
    if !info.users.len() == 1 {
        return Err(MyResponder::NotFound("User info not found".to_owned()));
    }
    if let Some(user_info) = info.users.iter().next() {
        return Ok(content::Json(serde_json::to_string(&user_info)?));
    }
    Err(MyResponder::NotFound("User not found!".into()))
}

/// This is a rate limited endpoint to poll for an auth token
#[get("/list_intermediate_tokens")]
pub fn list_intermediate_tokens(
    redis: rocket::State<redis::Client>,
    oauth_user: guard_oauth_jwt_access::OAuthIdentity,
    _rate_limiter: RateLimiter,
) -> Result<content::Json<String>, MyResponder> {
    // Only the google account is allowed to call this endpoint
    if oauth_user.credentials_index != CREDENTIALS_GOOGLE_SERVICE_ACCOUNT_INDEX {
        return Err(MyResponder::AccessScopeInsufficient(
            "Only the google CI account is allowed to call this endpoint".to_owned(),
        ));
    }

    let mut c = redis.get_connection()?;
    let k: Vec<String> = c.keys("*").unwrap();
    let mut map: HashMap<String, String> = HashMap::new();
    for key in k {
        let v: Option<String> = c.get(&key).unwrap();
        if !v.is_some() {
            continue;
        }
        map.insert(key.to_owned(), v.unwrap());
    }
    Ok(content::Json(serde_json::to_string(&map)?))
}
