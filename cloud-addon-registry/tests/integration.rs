#![feature(proc_macro_hygiene, decl_macro)]

use cloud_addon_registry::create_rocket;
use cloud_addon_lib::{dto::{db,addons}, github};
use cloud_auth_lib::Credentials;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use serde::Deserialize;

use rocket::http::{ContentType, Header, Status};
use cloud_vault::jwt::create_jwt_encoded_for_user;

use chrono::Duration;

use firestore_db_and_auth::{credentials::Credentials as DBCredentials, ServiceSession, documents};

impl From<String> for ErrorResult {
    fn from(str: String) -> Self {
        serde_json::from_str(&str).unwrap()
    }
}

#[test]
fn parse_extensions_github_file() {
    let client = github::create_client().unwrap();
    let url =
        "https://raw.githubusercontent.com/openhab-nodes/addons-registry/master/extensions.json";
    let r = client.0.get(url).send().unwrap().text().unwrap();
    let _r: addons::AddonEntryMap = serde_json::from_str(&r).unwrap();
}

#[derive(Deserialize)]
pub struct ErrorResult {
    pub error: String,
    pub message: String,
}

fn delete_tests(client: &rocket::local::Client, access_token: &str) {
    let mut r = client.delete("/addon/fantasy_name");
    r.add_header(Header::new(
        "Authorization",
        format!("Bearer {}", access_token),
    ));
    let mut r = r.dispatch();
    let response = ErrorResult::from(r.body_string().unwrap());
    assert_eq!(response.error, "NOT_FOUND");
    assert_eq!(r.status(), Status::BadRequest);

    // Delete demo addon
    let mut r = client.delete("/addon/ohx-ci-test-addon?force=true");
    r.add_header(Header::new(
        "Authorization",
        format!("Bearer {}", access_token),
    ));
    r.dispatch();
}

fn add_addon_tests(client: &rocket::local::Client, addons_file: &mut addons::AddonFileEntryPlusStats, access_token: &str) {

    // Test add addon - Fail not preprocessed

    let mut request = client.post("/addon");
    request.add_header(ContentType::JSON);
    request.add_header(Header::new(
        "Authorization",
        format!("Bearer {}", access_token),
    ));
    request.set_body(serde_json::to_string(&addons_file).unwrap());

    let mut response = request.dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    let error_response = ErrorResult::from(response.body_string().unwrap());
    assert_eq!(error_response.error, "NOT_PREPROCESSED");

    // Test add addon - OK
    let mut service_entry = addons_file.services.get_mut("ohx-addon-name").unwrap();
    service_entry.image = Some("my-image".to_owned());
    service_entry.build = None;

    let mut request = client.post("/addon");
    request.add_header(ContentType::JSON);
    request.add_header(Header::new(
        "Authorization",
        format!("Bearer {}", access_token),
    ));
    request.set_body(serde_json::to_string(&addons_file).unwrap());

    let mut response = request.dispatch();
    println!("{}", response.body_string().unwrap_or_default());
    assert_eq!(response.status(), Status::Ok);
}

fn update_addon_tests(client: &rocket::local::Client, addons_file: &mut addons::AddonFileEntryPlusStats, access_token: &str, access_token_other_user: &str) {

    // Test update addon - fail version
    addons_file.x_ohx_registry.version = "1.0.0".to_owned();
    let mut request = client.post("/addon");
    request.add_header(ContentType::JSON);
    request.add_header(Header::new(
        "Authorization",
        format!("Bearer {}", access_token),
    ));
    request.set_body(serde_json::to_string(&addons_file).unwrap());

    let mut response = request.dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    let error_response = ErrorResult::from(response.body_string().unwrap());
    assert_eq!(error_response.error, "VERSION_MUST_BE_NEWER");

    // Test update addon - fail owner
    addons_file.x_ohx_registry.version = "3.0.0".to_owned();
    let mut request = client.post("/addon");
    request.add_header(ContentType::JSON);
    request.add_header(Header::new(
        "Authorization",
        format!("Bearer {}", access_token_other_user),
    ));
    request.set_body(serde_json::to_string(&addons_file).unwrap());

    let mut response = request.dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    let error_response = ErrorResult::from(response.body_string().unwrap());
    assert_eq!(error_response.error, "WRONG_OWNER");

    // Test update ok
    let mut request = client.post("/addon");
    request.add_header(ContentType::JSON);
    request.add_header(Header::new(
        "Authorization",
        format!("Bearer {}", access_token),
    ));
    request.set_body(serde_json::to_string(&addons_file).unwrap());

    let mut response = request.dispatch();
    println!("Update Ok: {}", response.body_string().unwrap_or_default());
    assert_eq!(response.status(), Status::Ok);
}

fn stats_tests(client: &rocket::local::Client, firebase: &ServiceSession, google_access_token: &str) {
    let github_client = github::create_client().unwrap();

    // Get rating from before
    let (rating, _sha) = github::get_metadata_content(&github_client).unwrap();

    // Test update statistics
    let stat_update = db::RatingsInDB {
        rate: 4,
        last_rating: 0,
        addon_id: "ohx-ci-test-addon".to_string(),
    };

    let dl_update = db::DownloadsInDB {
        installed: 1,
        addon_id: "ohx-ci-test-addon".to_string(),
    };

    // Write ratings and downloads document
    documents::write(firebase, "ratings", Some("ci_demo"), &stat_update, documents::WriteOptions::default()).unwrap();
    documents::write(firebase, "downloads", Some("ci_demo"), &dl_update, documents::WriteOptions::default()).unwrap();

    let mut request = client.get("/update_stats");
    request.add_header(Header::new(
        "Authorization",
        format!("Bearer {}", google_access_token),
    ));
    let mut response = request.dispatch();
    println!("Update Ok: {}", response.body_string().unwrap_or_default());
    assert_eq!(response.status(), Status::Ok);

    // Check that ratings and downloads document have been used
    let r: Result<db::RatingsInDB, _> = documents::read(firebase, "ratings", "ci_demo");
    assert!(r.is_err());
    let r: Result<db::DownloadsInDB, _> = documents::read(firebase, "downloads", "ci_demo");
    assert!(r.is_err());

    // Check that rating is updated
    if let Some(rating) = rating.get("ohx-ci-test-addon") {
        let (new_rating, _sha) = github::get_metadata_content(&github_client).unwrap();
        let new_rating = new_rating.get("ohx-ci-test-addon").expect("Entry expected!");
        assert_eq!(rating.d + 1, new_rating.d);
        assert_eq!(rating.v + 1, new_rating.v);
        assert_eq!(rating.p + 4, new_rating.p);
    } else {}
}

#[test]
fn integration() -> Result<(), failure::Error> {
    let rocket = create_rocket(100)?;

    let (_, google_access_token, _) = Credentials::load_and_check(
        include_str!("../secrets/google-ci-key.json"),
        &[
            include_str!("../secrets/securetoken@system.gserviceaccount.com.json"),
            include_str!("../secrets/travisci-deployer@openhabx.iam.gserviceaccount.com.json"),
        ],
        None::<&[&str]>,
    )?;

    let (credentials, _, _) = Credentials::load_and_check(
        include_str!("../secrets/ohx_admin_account.json"),
        &[
            include_str!("../secrets/ohx_oauth_key.json"),
        ],
        None::<&[&str]>,
    )?;

    let client = rocket::local::Client::new(rocket).expect("valid rocket instance");

    // Test remove of non existing addon
    let r = client.delete("/addon/fantasy_name").dispatch();
    assert_eq!(r.status(), Status::Unauthorized);

    let access_token = create_jwt_encoded_for_user(&credentials, None::<&[&str]>, Duration::hours(1), Some(credentials.client_id.clone()),
                                                   "demo_user".to_owned(), "email".to_owned())?;

    let access_token_other_user = create_jwt_encoded_for_user(&credentials, None::<&[&str]>, Duration::hours(1), Some(credentials.client_id.clone()),
                                                              "demo_user_2".to_owned(), "email".to_owned())?;

    let firebase_credentials = DBCredentials::new(
        include_str!("../secrets/firebase-account.json"),
        &[
            include_str!("../secrets/openhabx-device@openhabx.iam.gserviceaccount.com.json"),
            include_str!("../secrets/securetoken@system.gserviceaccount.com.json"),
        ],
    )?;

    let firebase = ServiceSession::new(firebase_credentials)?;

    let addons_file = addons::open_validate_addons_file("tests/addon.yml").unwrap();
    let mut addons_file = addons::AddonFileEntryPlusStats {
        services: addons_file.services,
        x_ohx_registry: addons_file.x_ohx_registry,
        x_runtime: addons_file.x_runtime,
        archs: vec!["x86".to_owned()],
        size: 112,
    };

    delete_tests(&client, &access_token);
    add_addon_tests(&client, &mut addons_file, &access_token);
    stats_tests(&client, &firebase, &google_access_token);
    update_addon_tests(&client, &mut addons_file, &access_token, &access_token_other_user);

    // Remove test addon
    let mut request = client.delete("/addon/ohx-ci-test-addon?force=true");
    request.add_header(Header::new(
        "Authorization",
        format!("Bearer {}", access_token),
    ));
    let mut response = request.dispatch();
    println!("Update Ok: {}", response.body_string().unwrap_or_default());
    assert_eq!(response.status(), Status::Ok);

    Ok(())
}
