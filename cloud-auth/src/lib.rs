//#![deny(warnings)]
#![feature(proc_macro_hygiene, decl_macro)]

pub(crate) mod responder_type;
pub(crate) mod routes;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use cloud_auth_lib::{oauth_clients, guard_rate_limiter, fairing_cors, catch_all, Credentials, error_routes};
use std::sync::Mutex;
use rocket::{Config,catchers,routes};
use rocket::config::Environment;
use std::env;
use firestore_db_and_auth::{credentials::Credentials as DBCredentials, sessions::service_account::Session as SASession};

use routes::*;

// Embed the allowed oauth clients
const OAUTH_CLIENTS: &'static str = include_str!("../../data/oauth_clients.json");

// Include all kind of keys from the secret directory
const KEY_GOOGLE_TRAVIS: &'static str = include_str!("../../secrets/travisci-deployer@openhabx.iam.gserviceaccount.com.key");
const GOOGLE_SERVICE_ACCOUNT_ST: &'static str = include_str!("../../secrets/securetoken@system.gserviceaccount.com.json");
const GOOGLE_SERVICE_ACCOUNT_TRAVIS: &'static str = include_str!("../../secrets/travisci-deployer@openhabx.iam.gserviceaccount.com.json");
const GOOGLE_SERVICE_ACCOUNT_OHX: &'static str = include_str!("../../secrets/openhabx-device@openhabx.iam.gserviceaccount.com.json");
const OHX_ADMIN_ACCOUNT: &'static str = include_str!("../../secrets/ohx_oauth_key.key");
const OHX_AUTH_JWKS: &'static str = include_str!("../../secrets/ohx_oauth_key.json");
const FIREBASE_CREDENTIALS: &'static str = include_str!("../../secrets/openhabx-device@openhabx.iam.gserviceaccount.com.key");
const REDIS_CREDENTIALS: &'static str = include_str!("../../secrets/redis.txt");

pub fn create_rocket(rate_limit: u32) -> Result<rocket::Rocket, failure::Error> {
    let oauth_clients = oauth_clients::new(OAUTH_CLIENTS)?;

    // Rate limit
    let lim = guard_rate_limiter::RateLimiterMutex::new(rate_limit);

    let (google_credentials, _g_access_token, _g_scopes) =
        Credentials::load_and_check(KEY_GOOGLE_TRAVIS, &[GOOGLE_SERVICE_ACCOUNT_ST, GOOGLE_SERVICE_ACCOUNT_TRAVIS], None::<&[&str]>)?;

    let (openhabx_credentials, _ohx_access_token, _ohx_scopes) =
        Credentials::load_and_check(OHX_ADMIN_ACCOUNT, &[OHX_AUTH_JWKS], None::<&[&str]>)?;

    let credentials_list = vec![google_credentials, openhabx_credentials];

    let redis = redis::Client::open(REDIS_CREDENTIALS)?;

    let firebase_credentials = DBCredentials::new(FIREBASE_CREDENTIALS, &[GOOGLE_SERVICE_ACCOUNT_OHX, GOOGLE_SERVICE_ACCOUNT_ST, ])?;

    let firebase = Mutex::new(SASession::new(firebase_credentials.clone())?);

    let config = Config::build(Environment::Development)
        .port(
            env::var("PORT")
                .unwrap_or("8080".to_owned())
                .parse::<u16>()?,
        )
        .address("0.0.0.0")
        .workers(2)
        .finalize()?;

    #[cfg(debug_assertions)]
        {
            info!("Listening on http://localhost:{}", config.port);
            info!(
                "Google 1h access code for scopes: {:?}\n\t{}",
                _g_scopes.claims.scope,
                &_g_access_token
            );
            info!(
                "OHX 1h access code for scopes: {:?}\n\t{}",
                _ohx_scopes.claims.scope,
                &_ohx_access_token
            );
        }

    Ok(rocket::custom(config)
        .manage(credentials_list)
        .manage(lim)
        .manage(firebase)
        .manage(firebase_credentials)
        .manage(redis)
        .manage(oauth_clients)
        .attach(fairing_cors::CorsFairing)
        .register(catchers![
            error_routes::not_found,
            error_routes::access_denied,
            error_routes::not_authorized,
            error_routes::error_rate_limit
        ])
        .mount(
            "/",
            routes![
                index,
                check_for_users,
                check_for_users_unauthorized,
                authorize,
                list_intermediate_tokens,
                user_info,
                grant_scopes,
                grant_scopes_unauthorized,
                token,
                revoke_by_oauth,
                pubkey_jwk,
                openid_configuration
            ],
        )
        .mount("/", catch_all::catch_rest())
    )
}
