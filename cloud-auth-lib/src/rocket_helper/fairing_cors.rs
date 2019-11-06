use rocket::fairing::{Fairing, Info, Kind};
use rocket::{http::Method, http::Status, Request, Response};

pub struct CorsFairing;

impl Fairing for CorsFairing {
    fn info(&self) -> Info {
        Info {
            name: "CORS Fairing",
            kind: Kind::Response,
        }
    }

    fn on_response(&self, request: &Request, response: &mut Response) {
        // Add CORS headers to allow all origins to all outgoing requests
        response.set_header(rocket::http::Header::new(
            "Access-Control-Allow-Origin",
            request.headers().get_one("Origin").unwrap_or("*").to_owned(),
        ));
        response.set_header(rocket::http::Header::new(
            "Access-Control-Allow-Methods",
            "POST",
        ));
        response.set_header(rocket::http::Header::new(
            "Access-Control-Allow-Headers",
            "origin, authorization, content-type",
        ));
        response.set_header(rocket::http::Header::new(
            "Access-Control-Allow-Credentials",
            "true",
        ));
        response.set_header(rocket::http::Header::new(
            "Vary",
            "Origin",
        ));

        // Respond to all `OPTIONS` requests with a `204` (no content) status
        if response.status() == Status::NotFound && request.method() == Method::Options {
            response.set_status(Status::NoContent);
        }
    }
}