//! # Alexa Interface https://developer.amazon.com/de/docs/device-apis/alexa-interface.html
//! The Alexa interface contains directives and events related to state and error reporting. Although endpoints implement this interface implicitly, you should explicitly include this interface and the supported version in your discovery response.
//!

pub mod authorization;
pub mod discovery;
pub mod change_report;
pub mod properties;

use serde::{Deserialize, Serialize};

use super::{
    Header,
    PAYLOAD_VERSION,
    ErrorType,
    Endpoint,
};

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize)]
pub struct EmptyPayload {}

/// If a directive is successfully handled, you should respond with an Response event.
/// A Response can be sent from a Lambda function to Alexa or from your device cloud to the Alexa event gateway.
/// When you send a response event you must report the value of affected properties in the context of the message,
/// but you can include any and all reportable properties in the context.
///
/// In most cases Alexa waits 8 seconds before timing out whether the response is synchronous or asynchronous.
/// Exceptions are noted in individual interfaces.
/// For example, Alexa will wait longer for an asynchronous response from an endpoint that implements the LockController interface.
///
/// If you are responding asynchronously to the event gateway, the Response must include a scope that authenticates the customer to Alexa.
/// When you respond synchronously or asynchronously, the message should include a correlation token.
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize)]
pub struct Response<PROPERTIES, EVENT> where PROPERTIES: Serialize, EVENT: Serialize {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) context: Option<PROPERTIES>,
    pub(crate) event: EVENT,
}

impl<PROPERTIES, EVENT> Response<PROPERTIES, EVENT> where PROPERTIES: Serialize, EVENT: Serialize {
    pub fn new(event: EVENT) -> Self {
        Self {
            context: None,
            event,
        }
    }
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize)]
pub struct EventWithoutEndpoint<PAYLOAD> where PAYLOAD: Serialize {
    pub header: Header,
    pub(crate) payload: PAYLOAD,
}

impl<PAYLOAD> EventWithoutEndpoint<PAYLOAD> where PAYLOAD: Serialize {
    pub fn new(header: Header, payload: PAYLOAD) -> Self {
        Self { header, payload }
    }
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize)]
pub struct Event<PAYLOAD> where PAYLOAD: Serialize {
    pub header: Header,
    pub(crate) endpoint: Endpoint,
    pub(crate) payload: PAYLOAD,
}

impl<PAYLOAD> Event<PAYLOAD> where PAYLOAD: Serialize {
    pub fn new(header: Header, endpoint: Endpoint, payload: PAYLOAD) -> Self {
        Self { header, endpoint, payload }
    }
}

/// When you receive a directive, you can send a synchronous response or a DeferredResponse indicating you received the directive,
/// and later follow up with an asynchronous event to the Alexa endpoint.
/// In either case, there is a hard limit of 8 seconds before Alexa times out.
///
/// Known exceptions to the 8 second limit are: LockController
pub type DeferredResponse = Response<EmptyPayload, Event<DeferredResponsePayload>>;

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct DeferredResponsePayload {
    /// An integer that indicates an approximate time, in seconds, before the asynchronous response is sent.
    #[serde(rename = "estimatedDeferralInSeconds")]
    estimated_deferral_in_seconds: Option<u32>,
}

impl DeferredResponse {
    pub fn new(message_id: &str, correlation_token: String, endpoint: Endpoint, estimated_deferral_in_seconds: Option<u32>) -> Self {
        DeferredResponse {
            context: None,
            event: Event {
                header: Header::with_correlation_token("Alexa", "DeferredResponse", correlation_token),
                endpoint,
                payload: DeferredResponsePayload { estimated_deferral_in_seconds },
            },
        }
    }
}

