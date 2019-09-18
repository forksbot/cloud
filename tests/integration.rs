#![feature(proc_macro_hygiene, decl_macro)]

use cloud_auth::*;
use dto::oauth;

use failure::bail;
use cloud_vault::credentials::Credentials;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use rocket::http::uri::{Query, UriDisplay};
use serde::{Serialize, Deserialize};

use rocket::http::{ContentType, Header, Status};

use firestore_db_and_auth::{credentials::Credentials as DBCredentials, sessions::service_account::Session as SASession, errors::FirebaseError, documents, UserSession, FirebaseAuthBearer};

const CI_DEMO_USER: &'static str = "ci@openhabx.com";

#[derive(Deserialize)]
pub struct ErrorResult {
    pub error: String,
}

impl From<String> for ErrorResult {
    fn from(str: String) -> Self {
        serde_json::from_str(&str).unwrap()
    }
}

fn create_user(firebase: &SASession) -> Result<UserSession, failure::Error> {

    //// Create demo user
    let user_session = match firestore_db_and_auth::users::sign_up(&firebase, CI_DEMO_USER, "password1") {
        Ok(session) => session,
        Err(err) => {
            match err {
                FirebaseError::APIError(code, message, _context) => {
                    match code == 400 && message == "EMAIL_EXISTS" {
                        true => firestore_db_and_auth::users::sign_in(&firebase, CI_DEMO_USER, "password1")?,
                        false => bail!("Expected EMAIL_EXISTS: {} {} {}", code, message, _context)
                    }
                }
                _ => bail!("Unknown error: {:?}", err)
            }
        }
    };

    #[derive(Serialize)]
    struct UserData {
        queued_remove: i64
    }

    //// Add removal flag

    // Let's pretend the user has queued its profile to be deleted. That must have happened at least an hour ago for
    // the /check_for_users endpoint to delete the profile.
    let user_data = UserData { queued_remove: chrono::Utc::now().timestamp_millis() - 1000 * 60 * 60 };
    documents::write(&user_session, "users", Some(&user_session.user_id), &user_data, documents::WriteOptions::default())?;

    Ok(user_session)
}


fn check_for_users(client: &rocket::local::Client, g_access_token: &str, firebase: &SASession) -> Result<(), failure::Error> {

    ///////////////// check_for_users FAIL /////////////////
    let request = client.get("/check_for_users");
    let response = request.dispatch();
    assert_eq!(response.status(), Status::Unauthorized);

    ///////////////// check_for_users OK /////////////////

    //// Remove

    let mut request = client.get("/check_for_users");
    request.add_header(Header::new(
        "Authorization",
        format!("Bearer {}", g_access_token),
    ));

    let response = request.dispatch();
    assert_eq!(response.status(), Status::Ok);

    //// Check that user is gone

    assert!(firestore_db_and_auth::users::sign_in(&firebase, "ci@openhabx.com", "password1").is_err());

    Ok(())
}

fn user_info(client: &rocket::local::Client, g_access_token: &str, ohx_access_token: &str) -> Result<(), failure::Error> {

    ///////////////// userinfo (Io2cPph06rUWM3ABcIHguR3CIw6v1) FAIL (wrong scope. Need "profile") /////////////////

    println!("/userinfo");
    let mut request = client.get(format!(
        "/userinfo?user_id={}",
        "Io2cPph06rUWM3ABcIHguR3CIw6v1"
    ));
    request.add_header(ContentType::JSON);
    request.add_header(Header::new(
        "Authorization",
        format!("Bearer {}", g_access_token),
    ));

    let response = request.dispatch();
    assert_eq!(response.status(), Status::Unauthorized);

    ///////////////// create service account session with correct scopes /////////////////

    let (_, g_access_token, _) = Credentials::load_and_check(
        include_str!("../secrets/google-ci-key.json"),
        &[
            include_str!("../secrets/securetoken@system.gserviceaccount.com.json"),
            include_str!("../secrets/travisci-deployer@openhabx.iam.gserviceaccount.com.json"),
        ],
        Some(&["profile"]),
    )?;

    ///////////////// userinfo (Io2cPph06rUWM3ABcIHguR3CIw6v1) OK /////////////////

    println!("/userinfo by service account");
    let mut request = client.get(format!(
        "/userinfo?user_id={}",
        "Io2cPph06rUWM3ABcIHguR3CIw6v1"
    ));
    request.add_header(ContentType::JSON);
    request.add_header(Header::new(
        "Authorization",
        format!("Bearer {}", g_access_token),
    ));

    let mut response = request.dispatch();
    let body = response
        .body_string()
        .unwrap();
    println!("response: {}", body);
    assert_eq!(response.status(), Status::Ok);
    assert!(body.contains("Io2cPph06rUWM3ABcIHguR3CIw6v1"));

    ///////////////// userinfo (Io2cPph06rUWM3ABcIHguR3CIw6v1) OK /////////////////
    println!("/userinfo by user account");
    let mut request = client.get("/userinfo");
    request.add_header(ContentType::JSON);
    request.add_header(Header::new(
        "Authorization",
        format!("Bearer {}", &ohx_access_token),
    ));

    let mut response = request.dispatch();
    let body = response
        .body_string()
        .unwrap();
    println!("response: {}", body);
    assert!(body.contains("ci@openhabx.com"));
    assert_eq!(response.status(), Status::Ok);


    Ok(())
}


fn auth_and_token_code_grant_flow(client: &rocket::local::Client, _g_access_token: &str, _firebase: &SASession, user_session: &UserSession) -> Result<(), failure::Error> {

    ///////////////// code grant + device flow - authorize fail client unknown /////////////////

    let mut message = oauth::GenerateCodeDTO {
        client_id: "demo_client".to_string(),
        client_secret: Some("demo_secret".to_string()),
        client_name: Some("demo_name".to_string()),
        redirect_uri: Some("demo_redirect".to_string()),
        response_type: "code".to_string(),
        scope: None,
        state: Some("test".to_string()),
    };

    info!("/authorize fail client unknown");
    let mut request = client.post("/authorize");
    request.add_header(ContentType::Form);
    request.set_body(format!("{}", &message as &dyn UriDisplay<Query>));

    let mut response = request.dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    let response = ErrorResult::from(response.body_string().unwrap());
    assert_eq!(response.error, "client_id unknown");

    ///////////////// code grant + device flow - invalid requested scopes /////////////////
    message.scope = Some("admin".into());
    message.client_id = "ohx".to_owned();

    info!("/authorize invalid requested scopes");
    let mut request = client.post("/authorize");
    request.add_header(ContentType::Form);
    request.set_body(format!("{}", &message as &dyn UriDisplay<Query>));

    let mut response = request.dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    let response = ErrorResult::from(response.body_string().unwrap());
    assert_eq!(response.error, "Requested scopes are invalid");

    ///////////////// code grant flow - authorize OK /////////////////
    message.scope = Some("device".into());

    info!("/authorize code grant");
    let mut request = client.post("/authorize");
    request.add_header(ContentType::Form);
    request.set_body(format!("{}", &message as &dyn UriDisplay<Query>));

    let response = request.dispatch();
    assert_eq!(response.status(), Status::SeeOther);
    let location_redirect = response.headers().get("Location").next().unwrap();
    debug!("Location {}", location_redirect);

    let redirect: oauth::AuthPageRedirectUri = serde_urlencoded::from_str(&location_redirect[location_redirect.find("?").unwrap() + 1..])?;
    assert_eq!(redirect.client_id, message.client_id.clone());
    assert_eq!(redirect.client_secret, message.client_secret.clone());
    assert_eq!(redirect.client_name, message.client_name.clone());
    assert_eq!(redirect.redirect_uri, message.redirect_uri.clone());
    assert_eq!(redirect.response_type, "code");
    assert_eq!(redirect.state, message.state);

    ///////////////// code grant flow - Simulated UI grants scopes OK /////////////////

    let mut r = oauth::GrantRequest {
        unsigned: redirect.unsigned,
        code: redirect.code,
        scopes: Default::default(),
    };
    r.scopes.insert("device".to_owned());

    info!("/grant_scopes code grant");
    let mut request = client.post("/grant_scopes");
    request.add_header(ContentType::JSON);
    request.add_header(Header::new(
        "Authorization",
        format!("Bearer {}", user_session.access_token()),
    ));
    println!("SEND {}", serde_json::to_string(&r)?);
    request.set_body(serde_json::to_string(&r)?);

    let mut response = request.dispatch();
    let code = response.body_string().unwrap();
    println!("RECEIVE {}", &code);
    assert_eq!(response.status(), Status::Ok);

    ///////////////// code grant flow - Tokenize OK/////////////////

    let message = oauth::TokenDTO {
        code: Some(code),
        client_id: message.client_id,
        client_secret: None,
        grant_type: "authorization_code".to_string(),
        ..Default::default()
    };

    info!("/token code grant");
    let mut request = client.post("/token");
    request.add_header(ContentType::Form);
    request.set_body(format!("{}", &message as &dyn UriDisplay<Query>));

    let mut response = request.dispatch();
    let code = response.body_string().unwrap();
    println!("{}", &code);
    assert_eq!(response.status(), Status::Ok);

    let token_response: oauth::OAuthTokenResponse = serde_json::from_str(&code)?;
    assert_eq!(token_response.scope, "device");
    assert_eq!(token_response.token_type, "bearer");
    assert!(!token_response.access_token.is_empty());
    assert!(token_response.refresh_token.is_none());

    Ok(())
}

fn auth_and_token_device_flow(client: &rocket::local::Client, g_access_token: &str, _firebase: &SASession, user_session: &UserSession) -> Result<(), failure::Error> {
    ///////////////// device flow - authorize OK /////////////////

    let generate_token = oauth::GenerateCodeDTO {
        client_id: "addoncli".to_string(),
        client_secret: None,
        client_name: None,
        redirect_uri: None,
        response_type: "device".to_string(),
        scope: Some("addons offline_access".into()),
        state: Some("test".to_string()),
    };

    info!("/authorize device flow - authorize OK");
    let mut request = client.post("/authorize");
    request.add_header(ContentType::Form);
    request.set_body(format!("{}", &generate_token as &dyn UriDisplay<Query>));

    let mut response = request.dispatch();
    let code = response.body_string().unwrap();
    println!("RECEIVE {}", &code);
    assert_eq!(response.status(), Status::Ok);

    let token_response: oauth::DeviceFlowResponse = serde_json::from_str(&code)?;
    let redirect: oauth::AuthPageRedirectUri = serde_urlencoded::from_str(&token_response.verification_uri[token_response.verification_uri.find("?").unwrap() + 1..])?;
    assert_eq!(redirect.client_id, generate_token.client_id.clone());
    assert_eq!(redirect.response_type, "device");
    assert_eq!(redirect.state, generate_token.state);

    ///////////////// device flow - Check .. not authorized yet /////////////////

    use rocket::UriDisplayQuery;

    #[derive(UriDisplayQuery)]
    pub struct TokenDTO {
        pub device_code: String,
        pub client_id: String,
        pub grant_type: String,
    }

    let message = TokenDTO {
        device_code: code,
        client_id: generate_token.client_id.clone(),
        grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_string(),
    };

    info!("/token device flow authorization_pending");
    let mut request = client.post("/token");
    request.add_header(ContentType::Form);
    request.set_body(format!("{}", &message as &dyn UriDisplay<Query>));

    let mut response = request.dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    let response = ErrorResult::from(response.body_string().unwrap());
    assert_eq!(response.error, "authorization_pending");

    ///////////////// device flow - Simulated UI grants scopes OK /////////////////

    let mut r = oauth::GrantRequest {
        unsigned: redirect.unsigned,
        code: redirect.code,
        scopes: Default::default(),
    };
    r.scopes.insert("addons".to_owned());
    r.scopes.insert("offline_access".to_owned());

    info!("/grant_scopes");
    let mut request = client.post("/grant_scopes");
    request.add_header(ContentType::JSON);
    request.add_header(Header::new(
        "Authorization",
        format!("Bearer {}", user_session.access_token()),
    ));
    println!("SEND {}", serde_json::to_string(&r)?);
    request.set_body(serde_json::to_string(&r)?);

    let mut response = request.dispatch();
    let code = response.body_string().unwrap();
    println!("{}", &code);
    assert_eq!(response.status(), Status::Ok);

    ///////////////// device flow - Tokenize OK/////////////////

    let message = oauth::TokenDTO {
        code: Some(code),
        client_id: generate_token.client_id.clone(),
        grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_string(),
        ..Default::default()
    };

    info!("/token");
    let mut request = client.post("/token");
    request.add_header(ContentType::Form);
    request.set_body(format!("{}", &message as &dyn UriDisplay<Query>));

    let mut response = request.dispatch();
    let code = response.body_string().unwrap();
    println!("{}", &code);
    assert_eq!(response.status(), Status::Ok);

    let token_response: oauth::OAuthTokenResponse = serde_json::from_str(&code)?;
    let refresh_token = token_response.refresh_token.as_ref().unwrap();
    assert!(token_response.scope.contains("addons") && token_response.scope.contains("offline_access"));
    assert_eq!(token_response.token_type, "bearer");
    assert!(!token_response.access_token.is_empty());
    assert!(!refresh_token.is_empty());

    // Check if in database -- get new access token

    info!("/token refresh token");

    let message = oauth::TokenDTO {
        refresh_token: Some(refresh_token.to_owned()),
        client_id: generate_token.client_id,
        grant_type: "refresh_token".to_string(),
        ..Default::default()
    };

    let mut request = client.post("/token");
    request.add_header(ContentType::Form);
    request.set_body(format!("{}", &message as &dyn UriDisplay<Query>));

    let mut response = request.dispatch();
    let code = response.body_string().unwrap();
    println!("{}", &code);
    assert_eq!(response.status(), Status::Ok);

    // Remove token

    info!("/revoke");

    let mut request = client.get(format!("/revoke?token={}", refresh_token));
    request.add_header(Header::new(
        "Authorization",
        format!("Bearer {}", g_access_token),
    ));

    let response = request.dispatch();
    assert_eq!(response.status(), Status::Ok);

    Ok(())
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

    let firebase = SASession::new(firebase_credentials)?;


    let (_, g_access_token, _) = Credentials::load_and_check(
        include_str!("../secrets/google-ci-key.json"),
        &[
            include_str!("../secrets/securetoken@system.gserviceaccount.com.json"),
            include_str!("../secrets/travisci-deployer@openhabx.iam.gserviceaccount.com.json"),
        ],
        None::<&[&str]>,
    )?;

    let (_, ohx_access_token, _) = Credentials::load_and_check_for_user(
        include_str!("../secrets/ohx_admin_account.json"),
        &[
            include_str!("../secrets/ohx_oauth_key.json"),
        ],
        Some(&["profile"]),
        CI_DEMO_USER.to_owned(),
    )?;


    let client = rocket::local::Client::new(rocket).expect("valid rocket instance");

    let user_session = create_user(&firebase)?;

    user_info(&client, &g_access_token, &ohx_access_token)?;
    auth_and_token_code_grant_flow(&client, &g_access_token, &firebase, &user_session)?;
    auth_and_token_device_flow(&client, &g_access_token, &firebase, &user_session)?;
    check_for_users(&client, &g_access_token, &firebase)?;

    Ok(())
}
