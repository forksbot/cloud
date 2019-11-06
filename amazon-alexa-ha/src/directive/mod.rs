use serde::{Deserialize, Serialize};

use super::{
    Header,
    PAYLOAD_VERSION,
    ErrorType,
    Endpoint,
};
use std::collections::BTreeMap;

/// A command consists of zero, one or more properties like "brightness=50".
pub type Command = serde_json::value::Map<String, serde_json::Value>;

pub trait Directive {
    const NAME: &'static str;
    fn new(data: Command) -> Result<Self, serde_json::Error> where Self: std::marker::Sized, for<'de> Self: Deserialize<'de> {
        serde_json::from_value(serde_json::Value::Object(data))
    }
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct EmptyPayload {}

#[allow(non_camel_case_types)]
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum GrantType {
    /// The only supported child type for a grant is the OAuth2.AuthorizationCode type.
    /// When you receive an OAuth2.AuthorizationCode, grant also contains a code attribute that contains an OAuth2 authorization code.
    /// Exchange the authorization code for an access token and refresh token using Login With Amazon (LWA).
    #[serde(rename = "OAuth2.AuthorizationCode")]
    AuthorizationCode { code: String },
}

#[allow(non_camel_case_types)]
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum GranteeType {
    /// Currently, the only supported child type for grantee is the BearerToken type.
    /// When you receive a BearerToken type, grantee also contains a token attribute,
    /// which contains the customer access token received by Alexa from the account linking process.
    /// Use the token to identify the customer in your system.
    BearerToken { code: String },
}

/// The Alexa interface directives are all supported by this enumeration. Specific controller
/// directives are found in the [`super::controller`] module.
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum DirectivePayload {
    /// The purpose of the AcceptGrant is to enable you to obtain credentials that identify and authenticate a customer to Alexa.
    AcceptGrant { grant: GrantType, grantee: GranteeType },

    /// Support the Discover directive so that customers find the devices associated with their account.
    /// Users can ask Alexa to discover their devices, or they can open the Alexa app and choose discover devices.
    ///
    /// If you can't handle a Discover directive successfully, respond with an Alexa.ErrorResponse event.
    /// Use one of the following error types as appropriate:
    /// BRIDGE_UNREACHABLE, EXPIRED_AUTHORIZATION_CREDENTIAL, INSUFFICIENT_PERMISSIONS, INTERNAL_ERROR, INVALID_AUTHORIZATION_CREDENTIAL.
    Discover { scope: super::Scope },

    /// A ReportState directive is sent to request the current values of state properties for an endpoint.
    /// You respond to a ReportState with a StateReport and include all of the properties you have defined for that endpoint.
    ReportState,

    /// Virtual directive of this crate: Encapsulates properties like "brightness" of a command like "SetBrightness".
    Command(Command),
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
#[serde(tag = "namespace")]
pub enum DirectiveNamespace {
    /// The purpose of the AcceptGrant is to enable you to obtain credentials that identify and authenticate a customer to Alexa.
    Alexa(DirectivePayload),

}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectivePreliminary {
    directive: DirectivePreliminaryInner,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
struct DirectivePreliminaryInner {
    pub header: Header,
    pub endpoint: Option<Endpoint>,
    pub payload: Option<serde_json::value::Map<String, serde_json::Value>>,
}

pub struct DirectiveDecodedResult {
    pub header: Header,
    pub endpoint: Option<Endpoint>,
    pub payload: DirectivePayload,
}

/// TODO
/// Returns the correlation token, if any.
pub fn decode(input: &str) -> Result<DirectiveDecodedResult, ErrorType> {
    let input: DirectivePreliminary = serde_json::from_str(input).map_err(|e| ErrorType::INTERNAL_ERROR { message: e.to_string() })?;

    let mut payload = match input.directive.payload {
        Some(payload) => payload,
        None => Command::new()
    };

    payload.insert("type".to_owned(), input.directive.header.name.clone().into());

    match &input.directive.header.namespace[..] {
        "Alexa" | "Alexa.Discovery" => {
            let payload: DirectivePayload = serde_json::from_value(serde_json::Value::Object(payload))
                .map_err(|e| ErrorType::INTERNAL_ERROR { message: e.to_string() })?;
            Ok(DirectiveDecodedResult {
                header: input.directive.header,
                endpoint: input.directive.endpoint,
                payload,
            })
        }
        _ => {
            Ok(DirectiveDecodedResult {
                header: input.directive.header,
                endpoint: input.directive.endpoint,
                payload: DirectivePayload::Command(payload),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::*;

    #[test]
    fn test_decode_report_state() {
        let input = r#"{
  "directive": {
    "header": {
      "messageId": "abc-123-def-456",
      "correlationToken": "abcdef-123456",
      "namespace": "Alexa",
      "name": "ReportState",
      "payloadVersion": "3"
    },
    "endpoint": {
      "endpointId": "appliance-001",
      "cookie": {},
      "scope":{
            "type":"BearerToken",
            "token":"access-token-from-skill"
      }
    },
    "payload": {
    }
  }
}"#;
        let decoded = decode(input).unwrap();
        assert_eq!(decoded.header.message_id, "abc-123-def-456");
        assert_eq!(decoded.endpoint.unwrap().endpoint_id, "appliance-001");
        match decoded.payload {
            DirectivePayload::ReportState => {}
            _ => panic!("Unexpected payload")
        }
    }

    #[test]
    fn test_decode_set_brightness() {
        let input = r#"{
  "directive": {
    "header": {
      "namespace": "Alexa.BrightnessController",
      "name": "SetBrightness",
      "messageId": "abc-123-def-789",
      "correlationToken": "<an opaque correlation token>",
      "payloadVersion": "3"
    },
    "endpoint": {
      "scope": {
        "type": "BearerToken",
        "token": "<an OAuth2 bearer token>"
      },
      "endpointId": "appliance-002",
      "cookie": {}
    },
    "payload": {
      "brightness": 50
    }
  }
}"#;
        let decoded = decode(input).unwrap();
        assert_eq!(decoded.header.message_id, "abc-123-def-789");
        assert_eq!(decoded.endpoint.unwrap().endpoint_id, "appliance-002");

        match decoded.payload {
            DirectivePayload::Command(v) => {
                let v = brightness::DirectiveSetBrightness::new(v).unwrap();
                assert_eq!(v.brightness, 50);
            }
            _ => panic!("Unexpected payload")
        }
    }
}
