use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    Header,
    ErrorType,
    EventWithoutEndpoint,
    EmptyPayload,
};

/// If the AcceptGrant directive was successfully handled, you must respond synchronously with an AcceptGrant.Response event.
pub type AcceptGrantResponse = super::Response<super::EmptyPayload, EventWithoutEndpoint<super::EmptyPayload>>;

/// If an error occurs while you are handling the AcceptGrant directive, you must respond synchronously with an ErrorResponse event.
pub type AcceptGrantErrorResponse = super::Response<super::EmptyPayload, EventWithoutEndpoint<ErrorType>>;

impl AcceptGrantResponse {
    pub fn new() -> Self {
        let header = Header::new("Alexa.Authorization", "AcceptGrant.Response");
        super::Response::new(EventWithoutEndpoint::new(header, EmptyPayload {}))
    }
}

impl AcceptGrantErrorResponse {
    pub fn error(message: String) -> Self {
        let header = Header::new("Alexa.Authorization", "ErrorResponse");
        super::Response::new(EventWithoutEndpoint::new(header, ErrorType::ACCEPT_GRANT_FAILED { message }))
    }
}
