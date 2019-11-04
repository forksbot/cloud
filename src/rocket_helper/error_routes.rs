use rocket::{catch,response::Responder};

#[catch(404)]
pub fn not_found(_req: &rocket::Request) -> &'static str {
    "404: Resource not found"
}

#[catch(403)]
pub fn access_denied(_req: &rocket::Request) -> &'static str {
    "403: Forbidden. Your request does not contain a valid authentication token"
}


#[derive(Responder)]
pub struct MyOtherResponder {
    inner: String,
    content_type: rocket::http::ContentType,
    www_auth: rocket::http::Header<'static>,
}

#[catch(401)]
pub fn not_authorized(_req: &rocket::Request) -> MyOtherResponder {
    MyOtherResponder {
        inner: "401: Unauthorized. The request requires user authentication." .to_owned(),
        content_type: rocket::http::ContentType::Plain,
        www_auth: rocket::http::Header::new(
            "WWW-Authenticate",
            "Bearer error='invalid_token' error_description='The access token expired'",
        ),
    }
}

#[catch(429)]
pub fn error_rate_limit(_req: &rocket::Request) -> &'static str {
    "429: Rate limited. This service is rated limited to prevent brute force attacks!"
}

