//#![deny(warnings)]
#![feature(proc_macro_hygiene, decl_macro)]

pub mod dto;
pub mod responder_type;
pub mod routes;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use rocket::{catchers, config::Environment, routes, Config};
use std::env;
use std::sync::Mutex;

use cloud_vault::{
    credentials::Credentials, error_routes, guard_rate_limiter,
};
use firestore_db_and_auth::{
    credentials::Credentials as DBCredentials, sessions::service_account::Session as SASession,
};
use routes::*;


pub fn create_rocket(rate_limit: u32) -> Result<rocket::Rocket, failure::Error> {
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

    let firebase_credentials = DBCredentials::new(
        include_str!("../secrets/firebase-account.json"),
        &[
            include_str!("../secrets/openhabx-device@openhabx.iam.gserviceaccount.com.json"),
            include_str!("../secrets/securetoken@system.gserviceaccount.com.json"),
        ],
    )?;

    let firebase = Mutex::new(SASession::new(firebase_credentials.clone())?);

    let bt = braintreepayment_graphql::Braintree::new(serde_json::from_str(include_str!("../secrets/braintree.json"))?);

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
                _g_scopes.get_scopes(),
                &_g_access_token
            );
            info!(
                "OHX 1h access code for scopes: {:?}\n\t{}",
                _ohx_scopes.get_scopes(),
                &_ohx_access_token
            );
        }

    Ok(rocket::custom(config)
        .manage(credentials_list)
        .manage(lim)
        .manage(bt)
        .manage(firebase)
        .manage(firebase_credentials)
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
                client_token,
                client_token_fire_auth,
                client_token_unauthorized,
                confirm,
                confirm_unauthorized,
                check_payments,
                check_payments_unauthorized,
            ],
        ))
}
