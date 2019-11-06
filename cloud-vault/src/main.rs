#![deny(warnings)]
#![feature(proc_macro_hygiene, decl_macro)]

mod responder_type;
mod access_scopes;
mod routes;
mod travis;

use routes::*;

use responder_type::MyResponder;
use cloud_auth::guard_ip_addr::ClientRealAddr;
use guard_rate_limiter::RateLimiter;

use credentials::Credentials;
use access_scopes::AccessScopes;

use std::env;
use rocket::http::RawStr;
use signal_hook::{iterator::Signals, SIGINT, SIGKILL, SIGTERM, SIGHUP, SIGQUIT};
use std::ops::Deref;
use rocket::http::Status;
use stackdriver_logger;
#[allow(unused_imports)]
use log::{error, info, trace, debug, warn};

use include_dir::Dir;
use cloud_auth_lib::{guard_rate_limiter, Credentials, fairing_cors, catch_all};

const SECRETS_DIR: Dir = include_dir!("../secrets");

// Embed the allowed oauth clients
const ACCESS_SCOPES: &'static str = include_str!("../../data/access_scopes.json");

// Include all kind of keys from the secret directory
const KEY_GOOGLE_TRAVIS: &'static str = include_str!("../../secrets/travisci-deployer@openhabx.iam.gserviceaccount.com.key");
const GOOGLE_SERVICE_ACCOUNT_ST: &'static str = include_str!("../../secrets/securetoken@system.gserviceaccount.com.json");
const GOOGLE_SERVICE_ACCOUNT_TRAVIS: &'static str = include_str!("../../secrets/travisci-deployer@openhabx.iam.gserviceaccount.com.json");
const OHX_ADMIN_ACCOUNT: &'static str = include_str!("../../secrets/ohx_oauth_key.key");
const OHX_AUTH_JWKS: &'static str = include_str!("../../secrets/ohx_oauth_key.json");

/// Start rocket. A few states need to be initialized first.
fn main() -> Result<(), failure::Error> {
    stackdriver_logger::init_with_cargo!();

    use rocket::config::{Config, Environment};

    let signals = match std::env::args().count() {
        1 => Signals::new(&[SIGINT, SIGTERM, SIGHUP, SIGQUIT]),
        _ => Signals::new(&[SIGINT, SIGKILL, SIGTERM, SIGHUP, SIGQUIT])
    }?;

    std::thread::spawn(move || {
        for sig in signals.forever() {
            warn!("Received signal {:?}", sig);
            std::process::exit(1);
        }
    });

    let access_scopes = AccessScopes::new(ACCESS_SCOPES)?;

    // Rate limit: Allow 5 units per second
    let lim = guard_rate_limiter::RateLimiterMutex::new(5u32);

    let (google_credentials, _g_access_token, _g_scopes) =
        Credentials::load_and_check(KEY_GOOGLE_TRAVIS, &[GOOGLE_SERVICE_ACCOUNT_ST, GOOGLE_SERVICE_ACCOUNT_TRAVIS], None::<&[&str]>)?;

    let (openhabx_credentials, _ohx_access_token, _ohx_scopes) =
        Credentials::load_and_check(OHX_ADMIN_ACCOUNT, &[OHX_AUTH_JWKS], None::<&[&str]>)?;

    let credentials_list = vec![google_credentials, openhabx_credentials];

    let config = Config::build(Environment::Development)
        .port(env::var("PORT").unwrap_or("8080".to_owned()).parse::<u16>()?)
        .address("0.0.0.0")
        .workers(2)
        .finalize()?;

    #[cfg(debug_assertions)]
        {
            info!("Listening on http://localhost:{}", config.port);
            info!("Access scopes {:?}", &access_scopes.0);
            info!("Google 1h access code for scopes: {:?}\n\t{}", _g_scopes.get_scopes(), &_g_access_token);
            info!("OHX 1h access code for scopes: {:?}\n\t{}", _ohx_scopes.get_scopes(), &_ohx_access_token);
        }

    rocket::custom(config)
        .attach(fairing_cors::CorsFairing)
        .manage(credentials_list)
        .manage(lim)
        .manage(access_scopes)
        .register(catchers![error_routes::not_found, error_routes::access_denied, error_routes::not_authorized, error_routes::error_rate_limit])
        .mount("/", routes![index, retrieve_oauth, retrieve_not_authorized, renew, renew_unauthorized, list, list_not_authorized])
        .mount("/", catch_all::catch_rest())
        .launch();
    Ok(())
}

#[test]
fn check_credentials() -> Result<(), failure::Error> {
    Credentials::load_and_check(KEY_GOOGLE_TRAVIS, &[GOOGLE_SERVICE_ACCOUNT_ST, GOOGLE_SERVICE_ACCOUNT_TRAVIS], None::<&[&str]>)?;
    Credentials::load_and_check(OHX_ADMIN_ACCOUNT, &[OHX_AUTH_JWKS], None::<&[&str]>)?;
    Ok(())
}