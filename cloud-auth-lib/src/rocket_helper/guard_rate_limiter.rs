use super::guard_ip_addr;
use std::sync::Mutex;
use rocket::{http::Status, request, Outcome, State};
use ratelimit_meter::KeyedRateLimiter;
use nonzero_ext::NonZero;
use crate::CloudAuthError;

pub struct RateLimiterMutex(Mutex<KeyedRateLimiter< std::net::IpAddr > >);

pub struct RateLimiter {}

impl RateLimiterMutex {
    pub fn new(rate: u32) -> RateLimiterMutex {
        let rate = NonZero::new(rate).unwrap();
        RateLimiterMutex(Mutex::new(KeyedRateLimiter::< std::net::IpAddr >::per_second(rate)))
    }
}

impl<'a, 'r> request::FromRequest<'a, 'r> for RateLimiter {
    type Error = CloudAuthError;

    fn from_request(request: &'a request::Request<'r>) -> request::Outcome<Self, Self::Error> {
        let rate_limiter_mutex = request
            .guard::<State<RateLimiterMutex>>()
            .success_or(CloudAuthError::TooManyRequests);
        if let Err(err) = rate_limiter_mutex {
            return Outcome::Failure((Status::InternalServerError, err));
        }
        let rate_limiter_mutex = &rate_limiter_mutex.as_ref().unwrap().0;
        let client_addr = guard_ip_addr::get_request_client_ip(&request);

        if let Some(client_addr) = client_addr {
            // The rate limiter state is mutex locked. Unwrap and check if the limit has been hit.
            if let Ok(mut rate_limiter) = rate_limiter_mutex.lock() {
                if rate_limiter.check(client_addr.ip).is_err() {
                    return Outcome::Failure((Status::TooManyRequests, CloudAuthError::TooManyRequests));
                }
            }
        }

        Outcome::Success(RateLimiter {})
    }
}

pub fn create() {}