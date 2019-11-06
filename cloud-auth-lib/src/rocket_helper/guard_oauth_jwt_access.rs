use crate::credentials::Credentials;
use crate::jwt::verify_access_token;
use rocket::{request, Outcome, State};
use std::collections::{BTreeSet};
use crate::CloudAuthError;

pub struct OAuthIdentity {
    pub credentials_index: usize,
    pub credentials_email: String,
    pub user_id: Option<String>,
    pub client_id: Option<String>,
    pub scopes: BTreeSet<String>,
}

fn get_token(request: &request::Request) -> Option<OAuthIdentity> {
    let token = request
        .headers()
        .get_one("Authorization")
        .and_then(|bearer| {
            if !bearer.starts_with("Bearer ") {
                return None;
            }
            Some((&bearer[7..]).to_owned())
        });
    let token: Option<String> = match token {
        Some(token) => Some(token.to_owned()),
        None => request.get_query_value("auth").and_then(|r| r.ok())
    };
    if token.is_none() {
        return None;
    }
    let token = token.unwrap();
    type CredentialsList = Vec<Credentials>;

    let credentials_list = match request
        .guard::<State<CredentialsList>>() {
        Outcome::Success(s) => s,
        _ => return None
    };

    let mut counter: usize = 0;
    for credentials in credentials_list.iter() {
        match verify_access_token(&credentials, &token) {
            Ok(token_validation_result) => {
                if let Some(validation_result) = token_validation_result {
                    return Some(OAuthIdentity {
                        credentials_index: counter,
                        credentials_email: credentials.client_email.clone(),
                        scopes: validation_result.claims.scope,
                        user_id: validation_result.claims.uid,
                        client_id: validation_result.claims.client_id,
                    });
                }
            }
            // An error means that the credentials matching but the token is invalid
            Err(_e) => {
                return None;
            }
        }
        counter += 1;
    }
    None
}

impl<'a, 'r> request::FromRequest<'a, 'r> for OAuthIdentity {
    type Error = CloudAuthError;

    fn from_request(request: &'a request::Request<'r>) -> request::Outcome<Self, Self::Error> {
        match get_token(request) {
            Some(data) => Outcome::Success(data),
            None => Outcome::Forward(())
        }
    }
}

impl<'a, 'r> request::FromRequest<'a, 'r> for &'a OAuthIdentity {
    type Error = CloudAuthError;

    fn from_request(request: &'a request::Request<'r>) -> request::Outcome<Self, Self::Error> {
        let cache: &Option<OAuthIdentity> = request.local_cache(|| get_token(request));

        match &cache {
            Some(data) => Outcome::Success(data),
            None => Outcome::Forward(())
        }
    }
}