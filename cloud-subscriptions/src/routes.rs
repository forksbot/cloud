// own
use crate::dto::{db};
use crate::responder_type::MyResponder;

// External, controlled libraries
use cloud_vault::{
    guard_oauth_jwt_access, guard_rate_limiter::RateLimiter,
};
use firestore_db_and_auth::{
    rocket::FirestoreAuthSessionGuard, sessions::service_account::Session as SASession, documents,
};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

// External libraries
use rocket::{get};

// std
use std::ops::Deref;
use std::sync::Mutex;

use braintreepayment_graphql::Braintree;

const CREDENTIALS_GOOGLE_SERVICE_ACCOUNT_INDEX: usize = 0;
const CREDENTIALS_OHX_SERVICE_ACCOUNT_INDEX: usize = 1;

/// Empty default route
#[get("/")]
pub fn index() -> &'static str {
    ""
}

fn client_token_internal(session: &SASession, braintree: &Braintree, doc: db::UserEntry, user_id: &str,
                         user_email: impl Fn() -> Result<String, MyResponder>) -> Result<String, MyResponder> {
    use braintreepayment_graphql::queries::customer::{customer_client_token, create_customer};

    let cust_id = match doc.braintree_customer_id {
        Some(id) => id,
        None => {
            let r = braintree.perform(create_customer::CreateCustomer {
                customer: create_customer::CustomerInput {
                    company: None,
                    custom_fields: None,
                    email: Some(user_email()?),
                    first_name: None,
                    last_name: None,
                    phone_number: None,
                }
            })?;
            if let Some(id) = r.create_customer.and_then(|f| f.customer).and_then(|f| Some(f.id)) {
                let doc = db::UserEntry {
                    braintree_customer_id: Some(id.clone())
                };
                documents::write(session, "users", Some(&user_id), &doc, documents::WriteOptions { merge: true })?;
                id
            } else {
                return Err(MyResponder::InternalError("Could not create a customer".to_owned()));
            }
        }
    };

    let r = braintree.perform(customer_client_token::CustomerClientToken {
        cust_id
    })?;
    if let Some(token) = r.create_client_token {
        if let Some(token) = token.client_token {
            return Ok(token);
        }
    }

    Err(MyResponder::InternalError("Got no token from Braintree".to_owned()))
}

fn get_email_for_firebase_auth_user(session: &firestore_db_and_auth::UserSession) -> Result<String, MyResponder> {
    let response = firestore_db_and_auth::users::user_info(session)?.users;
    for entry in response {
        if let Some(email) = entry.email {
            return Ok(email);
        }
    }
    Err(MyResponder::bad_request("User email address not found!"))
}

/// Get a braintree customer client token
#[get("/client_token")]
pub fn client_token(
    oauth_user: guard_oauth_jwt_access::OAuthIdentity,
    firebase: rocket::State<Mutex<SASession>>,
    braintree: rocket::State<Braintree>,
    _rate_limiter: RateLimiter,
) -> Result<String, MyResponder> {
    // Only the google account is allowed to call this endpoint
    if oauth_user.credentials_index != CREDENTIALS_OHX_SERVICE_ACCOUNT_INDEX || oauth_user.user_id.is_none() {
        return Err(MyResponder::AccessScopeInsufficient(
            "Only an OHX account is allowed to call this endpoint".to_owned(),
        ));
    }

    let session_mutex = firebase.lock()?;
    let session: &SASession = session_mutex.deref();

    let user_id = oauth_user.user_id.unwrap();
    let doc: db::UserEntry = documents::read(session, "users", &user_id)?;

    client_token_internal(session, &braintree, doc, &user_id, || {
        let user_session = firestore_db_and_auth::UserSession::by_user_id(&session.credentials, &user_id, false)?;
        get_email_for_firebase_auth_user(&user_session)
    })
}

/// Get a braintree customer client token
#[get("/client_token", rank = 2)]
pub fn client_token_fire_auth(
    firestore_auth: FirestoreAuthSessionGuard,
    firebase: rocket::State<Mutex<SASession>>,
    braintree: rocket::State<Braintree>,
    _rate_limiter: RateLimiter,
) -> Result<String, MyResponder> {
    let session_mutex = firebase.lock()?;
    let session: &SASession = session_mutex.deref();

    let doc: db::UserEntry = documents::read(session, "users", &firestore_auth.0.user_id)?;

    client_token_internal(session, &braintree, doc, &firestore_auth.0.user_id, || get_email_for_firebase_auth_user(&firestore_auth.0))
}

#[get("/client_token", rank = 3)]
pub fn client_token_unauthorized() -> MyResponder {
    MyResponder::AccessScopeInsufficient("Requires authorization".to_owned())
}

/// Get a braintree customer client token
#[get("/confirm")]
pub fn confirm(
    _oauth_user: guard_oauth_jwt_access::OAuthIdentity,
    _firebase: rocket::State<Mutex<SASession>>,
    _braintree: rocket::State<Braintree>,
) -> Result<String, MyResponder> {
    Ok(String::new())
}


#[get("/confirm", rank = 2)]
pub fn confirm_unauthorized() -> MyResponder {
    MyResponder::AccessScopeInsufficient("Requires authorization".to_owned())
}


/// Get a braintree customer client token
#[get("/check_payments")]
pub fn check_payments(
    _oauth_user: guard_oauth_jwt_access::OAuthIdentity,
    _firebase: rocket::State<Mutex<SASession>>,
    _braintree: rocket::State<Braintree>,
) -> Result<String, MyResponder> {
    Ok(String::new())
}

#[get("/check_payments", rank = 2)]
pub fn check_payments_unauthorized() -> MyResponder {
    MyResponder::AccessScopeInsufficient("Requires authorization".to_owned())
}
