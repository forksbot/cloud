use rocket::response::Responder;
use serde_json::json;

#[derive(Responder, Debug)]
pub enum MyResponder {
    #[response(status = 400, content_type = "json")]
    BadRequest(String),
    #[response(status = 401, content_type = "json")]
    AccessDenied(String),
    #[response(status = 404)]
    NotFound(String),
    #[response(status = 204)]
    NoContent(String),
    #[response(status = 429)]
    RateLimited(String),
    #[response(status = 500)]
    InternalError(String),
}

impl MyResponder {
    pub fn bad_request(code: &str, message: &str) -> MyResponder {
        MyResponder::BadRequest(json!({"error":code.to_owned(), "message": message.to_owned()}).to_string())
    }
    pub fn access_denied(code: &str, message: &str) -> MyResponder {
        MyResponder::AccessDenied(json!({"error":code.to_owned(), "message": message.to_owned()}).to_string())
    }
}

impl From<failure::Error> for MyResponder {
    fn from(err: failure::Error) -> MyResponder {
        #[allow(unused_imports)]
        use failure::{AsFail, Fail};
        MyResponder::InternalError(format!("{}, {}", err.as_fail(), err.backtrace()))
    }
}

impl<'a, T> From<std::sync::PoisonError<std::sync::MutexGuard<'a, T>>> for MyResponder {
    fn from(err: std::sync::PoisonError<std::sync::MutexGuard<'a, T>>) -> MyResponder {
        MyResponder::InternalError(err.to_string())
    }
}

impl From<semver::SemVerError> for MyResponder {
    fn from(err: semver::SemVerError) -> MyResponder {
        MyResponder::InternalError(err.to_string())
    }
}

impl From<serde_json::Error> for MyResponder {
    fn from(err: serde_json::Error) -> MyResponder {
        MyResponder::InternalError(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for MyResponder {
    fn from(err: std::string::FromUtf8Error) -> MyResponder {
        MyResponder::BadRequest(json!({"code":"INVALID_UTF8","error":err.to_string()}).to_string())
    }
}

impl From<firestore_db_and_auth::errors::FirebaseError> for MyResponder {
    fn from(err: firestore_db_and_auth::errors::FirebaseError) -> MyResponder {
        MyResponder::InternalError(err.to_string())
    }
}

impl From<reqwest::Error> for MyResponder {
    fn from(err: reqwest::Error) -> MyResponder {
        MyResponder::InternalError(format!("{:?}", err))
    }
}

impl From<reqwest::header::InvalidHeaderValue> for MyResponder {
    fn from(err: reqwest::header::InvalidHeaderValue) -> MyResponder {
        MyResponder::InternalError(format!("{:?}", err))
    }
}
