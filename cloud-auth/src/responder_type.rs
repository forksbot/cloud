//! # Custom Rocket.rs responder

use rocket::response::Responder;
use serde_json::json;

#[derive(Responder, Debug)]
pub enum MyResponder {
    #[response(status = 400, content_type = "json")]
    BadRequest(String),
    #[response(status = 401)]
    AccessScopeInsufficient(String),
    #[response(status = 404)]
    NotFound(String),
    //#[response(status = 204)]
    //NoContent(String),
    #[response(status = 429)]
    RateLimited(String),
    #[response(status = 500)]
    InternalError(String),
    #[response(status = 500, content_type = "json")]
    InternalErrorJson(String),
}

impl MyResponder {
    pub fn bad_request(message: &str) -> MyResponder {
        MyResponder::BadRequest(json!({"error":message.to_owned()}).to_string())
    }
    pub fn internal_error(message: &str) -> MyResponder {
        MyResponder::InternalErrorJson(json!({"error":message.to_owned()}).to_string())
    }
}

impl From<failure::Error> for MyResponder {
    fn from(err: failure::Error) -> MyResponder {
        #[allow(unused_imports)]
        use failure::{AsFail, Fail};
        MyResponder::InternalError(format!("{}, {}", err.as_fail(), err.backtrace()))
    }
}

impl From<redis::RedisError> for MyResponder {
    fn from(err: redis::RedisError) -> MyResponder {
        MyResponder::InternalError(err.to_string())
    }
}

impl<'a, T> From<std::sync::PoisonError<std::sync::MutexGuard<'a, T>>> for MyResponder {
    fn from(err: std::sync::PoisonError<std::sync::MutexGuard<'a, T>>) -> MyResponder {
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
        MyResponder::BadRequest(json!({"error":err.to_string()}).to_string())
    }
}

use cloud_auth_lib::CloudAuthError;

impl From<cloud_auth_lib::CloudAuthError> for MyResponder {
    fn from(err: cloud_auth_lib::CloudAuthError) -> MyResponder {
        match err {
            CloudAuthError::TooManyRequests => MyResponder::RateLimited(String::new()),
            err => MyResponder::BadRequest(json!({"error":err.to_string()}).to_string())
        }
    }
}

impl From<firestore_db_and_auth::errors::FirebaseError> for MyResponder {
    fn from(err: firestore_db_and_auth::errors::FirebaseError) -> MyResponder {
        MyResponder::InternalError(err.to_string())
    }
}

impl From<biscuit::errors::Error> for MyResponder {
    fn from(err: biscuit::errors::Error) -> MyResponder {
        MyResponder::BadRequest(json!({"error":err.to_string()}).to_string())
    }
}

//
//impl From<ring::error::Unspecified> for MyResponder {
//    fn from(err: ring::error::Unspecified) -> MyResponder {
//        MyResponder::BadRequest(json!({ "error": format!("{:?}", err) }).to_string())
//    }
//}
//
//impl From<base64::DecodeError> for MyResponder {
//    fn from(err: base64::DecodeError) -> MyResponder {
//        MyResponder::BadRequest(json!({"error":err.to_string()}).to_string())
//    }
//}
//
//impl From<miniz_oxide::inflate::TINFLStatus> for MyResponder {
//    fn from(err: miniz_oxide::inflate::TINFLStatus) -> MyResponder {
//        MyResponder::BadRequest(json!({ "error": format!("{:?}", err) }).to_string())
//    }
//}
//
