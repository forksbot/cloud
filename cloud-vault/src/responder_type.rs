use rocket::response::Responder;

#[derive(Responder, Debug)]
pub enum MyResponder {
    #[response(status = 401)]
    AccessScopeInsufficient(String),
    #[response(status = 404)]
    NotFound(String),
    #[response(status = 204)]
    NoContent(String),
    #[response(status = 429)]
    RateLimited(String),
    #[response(status = 500)]
    InternalError(String),
}

impl From<failure::Error> for MyResponder {
    fn from(err: failure::Error) -> MyResponder {
        #[allow(unused_imports)]
        use failure::{AsFail, Fail};
        MyResponder::InternalError(format!("{}, {}", err.as_fail(), err.backtrace()))
    }
}

impl From<serde_json::Error> for MyResponder {
    fn from(err: serde_json::Error) -> MyResponder {
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
