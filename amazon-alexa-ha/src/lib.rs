pub mod directive;
pub mod display_categories;
pub mod event;
pub mod property_types;
pub mod uom;
pub mod controller;
mod error;
mod utils_serde;

pub use error::{ErrorType, ErrorResponse};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::BTreeMap;

pub const PAYLOAD_VERSION: &'static str = "3";

#[derive(Serialize, Deserialize)]
pub enum CapabilityVersion {
    #[serde(rename = "3")]
    THREE
}

/// The type of capability. Currently, the only available type is AlexaInterface.
#[derive(Serialize, Deserialize)]
pub enum CapabilityType {
    AlexaInterface
}

/// An endpoint object identifies the target for a directive and the origin of an event. An endpoint can represent one of the following:
///
/// * Physical device
/// * Virtual device
/// * Group or cluster of devices
/// * Software component
///
/// In addition, the endpoint contains an authentication token.
///
/// * For a directive, the token enables the communication with the device(s) or component represented by the endpoint.
/// * For an event, the token is only required for events sent to the Alexa event gateway.
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct Endpoint {
    ///	A polymorphic object that describes an aspect of the authentication granted to the message exchange.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<Scope>,
    /// A unique identifier. The identifier must be unique across all endpoints for a customer within the domain of the skill. This identifier is provided initially during device discovery, and should consistently identify this device associated with this user.
    #[serde(rename = "endpointId")]
    pub endpoint_id: String,
    /// A list of key/value pairs associated with the endpoint. These are provided during discovery and are sent in each message for an endpoint.
    pub cookie: BTreeMap<String, String>,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Scope {
    /// The token to validate a customer in the system they account-linked when they enabled the skill.
    BearerToken { token: String },
    /// Provides an OAuth bearer token for accessing a linked customer account and the physical location where the discovery request should be applied. Typically used for skills that target Alexa for Business.
    /// * partition: The location target for the request such as a room name or number.
    /// * userId: A unique identifier for the user who made the request. You should not rely on userId for identification of a customer. Use token instead.
    BearerTokenWithPartition {
        token: String,
        partition: String,
        #[serde(rename = "userId")]
        user_id: String,
    },
}

/// A header has a set of expected fields that are the same across message types. These provide different types of identifying information.
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct Header {
    /// A string that specifies the category for the message payload. This aligns to the capability interface that contains that directive.
    pub namespace: String,
    /// The name of the directive such as TurnOn or TurnOff
    pub name: String,
    /// The version of the capability interface applied to this message.
    #[serde(rename = "payloadVersion")]
    pub payload_version: String,
    /// A unique identifier for a single request or response. This is used for tracking purposes and your skill should log this information, although it should not be used to support business logic. Every message must have this field populated. Any string of alphanumeric characters and dashes less than 128 characters is valid, but a version 4 UUID, which is a UUID generated from random numbers, is recommended.
    #[serde(rename = "messageId")]
    pub message_id: String,
    /// A token that identifies a directive and one or more events associated with it. A correlation token is included in the following message types:
    /// * A directive from Alexa to the skill.
    /// * An event in response to a directive.
    /// A response event for a directive request must include the same correlation.
    /// If the event is not in response to a request from Alexa, a correlation token must not be provided.
    #[serde(rename = "correlationToken")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_token: Option<String>,
}

impl Header {
    pub fn new(namespace: &str, name: &str) -> Self {
        Header {
            namespace: namespace.to_string(),
            name: name.to_string(),
            payload_version: PAYLOAD_VERSION.to_string(),
            message_id: Uuid::new_v4().to_string(),
            correlation_token: None,
        }
    }
    pub fn with_correlation_token(namespace: &str, name: &str, correlation_token: String) -> Self {
        Header {
            namespace: namespace.to_string(),
            name: name.to_string(),
            payload_version: PAYLOAD_VERSION.to_string(),
            message_id: Uuid::new_v4().to_string(),
            correlation_token: Some(correlation_token),
        }
    }
}

#[test]
fn header_deserialize() {
    let str = serde_json::to_string(&Header {
        namespace: "Alexa.Discovery".to_owned(),
        name: "ErrorResponse".to_owned(),
        payload_version: PAYLOAD_VERSION.to_string(),
        message_id: "1234".to_string(),
        correlation_token: None,
    })
        .unwrap();
    assert_eq!(str, "{\"namespace\":\"Alexa.Discovery\",\"name\":\"ErrorResponse\",\"payloadVersion\":\"3\",\"messageId\":\"1234\"}");
}
