use rocket::{Request, Route, Data};
use rocket::http::{Status};
use rocket::handler::Outcome;
use rocket::http::Method::*;

fn forward<'r>(req: &'r Request, _data: Data) -> Outcome<'r> {
    Outcome::from(req, Status::NotFound)
}

/// Prevents log entries for not found routes. Bots will usually try a few GET addresses and we
/// do not want any logs for that.
pub fn catch_rest() -> Vec<Route> {
    vec![Route::ranked(10, Get, "/<id>", forward),
         Route::ranked(10, Get, "/<id>/<id2>", forward),
         Route::ranked(10, Get, "/<id>/<id2>/<id3>", forward),
         Route::ranked(10, Get, "/<id>/<id2>/<id3>/<id4>", forward)]
}