//#![deny(warnings)]
#![feature(proc_macro_hygiene, decl_macro)]

pub mod dto;
pub mod oauth_clients;
pub mod responder_type;
pub mod routes;
pub mod token;
pub mod jwt;
pub mod credentials;
pub mod rocket_helper;
mod tools;

pub use rocket_helper::*;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use rocket::{catchers, config::Environment, routes, Config};
use std::env;
use std::sync::Mutex;

use credentials::Credentials;
use rocket_helper::{error_routes, guard_rate_limiter, fairing_cors, catch_all};
use firestore_db_and_auth::{
    credentials::Credentials as DBCredentials, sessions::service_account::Session as SASession,
};
use routes::*;

pub fn create_rocket(rate_limit: u32) -> Result<rocket::Rocket, failure::Error> {
    let oauth_clients = oauth_clients::new(include_str!("../oauth_clients.json"))?;

    // Rate limit
    let lim = guard_rate_limiter::RateLimiterMutex::new(rate_limit);

    let (google_credentials, _g_access_token, _g_scopes) = Credentials::load_and_check(
        include_str!("../secrets/google-ci-key.json"),
        &[
            include_str!("../secrets/securetoken@system.gserviceaccount.com.json"),
            include_str!("../secrets/travisci-deployer@openhabx.iam.gserviceaccount.com.json"),
        ],
        None::<&[&str]>,
    )?;

    let (openhabx_credentials, _ohx_access_token, _ohx_scopes) = Credentials::load_and_check(
        include_str!("../secrets/ohx_admin_account.json"),
        &[include_str!("../secrets/ohx_oauth_key.json")],
        None::<&[&str]>,
    )?;

    let credentials_list = vec![google_credentials, openhabx_credentials];

    let redis = redis::Client::open(include_str!("../secrets/redis.txt"))?;

    let firebase_credentials = DBCredentials::new(
        include_str!("../secrets/firebase-account.json"),
        &[
            include_str!("../secrets/openhabx-device@openhabx.iam.gserviceaccount.com.json"),
            include_str!("../secrets/securetoken@system.gserviceaccount.com.json"),
        ],
    )?;

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
