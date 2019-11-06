use cloud_auth_lib::guard_oauth_jwt_access;
use crate::responder_type::MyResponder;
use crate::{SECRETS_DIR, travis};
use cloud_auth_lib::guard_ip_addr::ClientRealAddr;
use cloud_auth_lib::guard_rate_limiter::RateLimiter;
use crate::access_scopes::AccessScopes;
use std::ops::Deref;

const CREDENTIALS_GOOGLE_SERVICE_ACCOUNT_INDEX: usize = 0;
const CREDENTIALS_OHX_SERVICE_ACCOUNT_INDEX: usize = 1;

/// Empty default route
#[get("/")]
pub fn index() -> &'static str {
    ""
}

/// Refresh access token environment variable on travis-ci
/// for all the different repositories (listed in repositories.json).
///
/// A token is valid for 6 hours. A cron job must call this endpoint periodically.
#[get("/renew")]
pub fn renew(oauth_user: guard_oauth_jwt_access::OAuthIdentity, credentials_list: rocket::State<Vec<credentials::Credentials>>) -> Result<String, MyResponder> {
    // Only the google account is allowed to call this endpoint
    if oauth_user.credentials_index != CREDENTIALS_GOOGLE_SERVICE_ACCOUNT_INDEX {
        return Err(MyResponder::AccessScopeInsufficient("Only the google CI account is allowed to call this endpoint".to_owned()));
    }

    let repositories: Vec<String> = serde_json::from_str(include_str!("../repositories.json"))?;
    let travis_token = SECRETS_DIR.get_file("travis-token.txt").unwrap().contents_utf8().unwrap();
    let credentials = credentials_list.get(CREDENTIALS_OHX_SERVICE_ACCOUNT_INDEX).unwrap();
    let response = travis::set_env_var(travis_token, repositories, credentials)?;

    Ok(response)
}

/// This is a rate limited, auth-only endpoint to get a secret
#[allow(unused_variables)]
#[get("/get/<id>?<auth>", rank = 2)]
pub fn retrieve_oauth(id: &RawStr, auth: Option<&RawStr>,
                  oauth: guard_oauth_jwt_access::OAuthIdentity,
                  client_addr: ClientRealAddr,
                  access_scopes: rocket::State<AccessScopes>,
                  rate_limiter: RateLimiter) -> Result<&'static str, MyResponder> {
    let id = id.as_str();
    match access_scopes.deref().0.get(id) {
        Some(v) => {
            for scope in &oauth.scopes {
                if v.contains(scope) {
                    // Access the requested file or return a file not found
                    let filename = SECRETS_DIR.get_file(id).ok_or(MyResponder::NotFound(format!("File not found {}", id)))?;
                    return Ok(filename.contents_utf8().unwrap());
                };
            }
        }
        _ => {}
    };
    Err(MyResponder::AccessScopeInsufficient(format!("Your access token does not allow access to {}. You need one of {:?}", id, &oauth.scopes)))
}

#[allow(unused_variables)]
#[get("/list?<auth>")]
pub fn list(auth: Option<&RawStr>, _oauth_user: guard_oauth_jwt_access::OAuthIdentity) -> Result<String, failure::Error> {
    let mut response = String::new();
    for entry in SECRETS_DIR.files() {
        let path = entry.path();
        response += path.to_str().unwrap_or("");
        response += "\n";
    }
    Ok(response)
}

#[get("/renew", rank = 2)]
pub fn renew_unauthorized() -> Status {
    Status::Unauthorized
}

#[get("/list", rank = 2)]
pub fn list_not_authorized() -> Status {
    Status::Unauthorized
}

#[get("/get/<_id>", rank = 3)]
pub fn retrieve_not_authorized(_id: &RawStr) -> Status {
    Status::Unauthorized
}
