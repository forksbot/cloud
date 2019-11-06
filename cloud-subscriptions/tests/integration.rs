#![feature(proc_macro_hygiene, decl_macro)]

use cloud_subscription::*;

use cloud_vault::credentials::Credentials;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use serde::{ Deserialize};

use firestore_db_and_auth::{credentials::Credentials as DBCredentials, sessions::service_account::Session as SASession};

#[derive(Deserialize)]
pub struct ErrorResult {
    pub error: String,
}

impl From<String> for ErrorResult {
    fn from(str: String) -> Self {
        serde_json::from_str(&str).unwrap()
    }
}

#[test]
fn integration() -> Result<(), failure::Error> {
    let rocket = create_rocket(100)?;

    let firebase_credentials = DBCredentials::new(
        include_str!("../secrets/firebase-account.json"),
        &[
            include_str!("../secrets/openhabx-device@openhabx.iam.gserviceaccount.com.json"),
            include_str!("../secrets/securetoken@system.gserviceaccount.com.json"),
        ],
    )?;

    let _firebase = SASession::new(firebase_credentials)?;


    let (_, _g_access_token, _) = Credentials::load_and_check(
        include_str!("../secrets/google-ci-key.json"),
        &[
            include_str!("../secrets/securetoken@system.gserviceaccount.com.json"),
            include_str!("../secrets/travisci-deployer@openhabx.iam.gserviceaccount.com.json"),
        ],
        None::<&[&str]>,
    )?;

    let _client = rocket::local::Client::new(rocket).expect("valid rocket instance");

    Ok(())
}
