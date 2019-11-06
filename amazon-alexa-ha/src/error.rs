use super::property_types::TemperaturePropertyValue;
use super::{Header, PAYLOAD_VERSION, Endpoint};
use serde::{Deserialize, Serialize};

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ErrorType {
    /// * You were unable to call Login with Amazon to exchange the authorization code for access and refresh tokens
    /// * You were trying to store the access and refresh tokens for the customer, but were unable to complete the operation for some reason
    /// * Any other errors that occurred while trying to retrieve and store the access and refresh tokens
    ACCEPT_GRANT_FAILED { message: String },
    /// The operation can't be performed because the endpoint is already in operation.
    ALREADY_IN_OPERATION { message: String },
    /// The bridge is unreachable or offline. For example, the bridge might be turned off, disconnected from the customer's local area network, or connectivity between the bridge and the device cloud might have been lost. When you respond to a ReportState directive, there may be times when you should return a StateReport instead of this error. For more information, see Alexa.EndpointHealth.
    BRIDGE_UNREACHABLE { message: String },
    /// The endpoint can't handle the directive because it is performing another action, which may or may not have originated from a request to Alexa.
    ENDPOINT_BUSY { message: String },
    /// The endpoint can't handle the directive because the battery power is too low.
    ENDPOINT_LOW_POWER { message: String },
    /// The endpoint is unreachable or offline. For example, the endpoint might be turned off, disconnected from the customer's local area network, or connectivity between the endpoint and bridge or the endpoint and the device cloud might have been lost. When you respond to a ReportState directive, there may be times when you should return a StateReport instead of this error. For more information, see Alexa.EndpointHealth.
    ENDPOINT_UNREACHABLE { message: String },
    /// The authorization credential provided by Alexa has expired. For example, the OAuth2 access token for the customer has expired.
    EXPIRED_AUTHORIZATION_CREDENTIAL { message: String },
    /// The endpoint can't handle the directive because it's firmware is out of date.
    FIRMWARE_OUT_OF_DATE { message: String },
    /// The endpoint can't handle the directive because it has experienced a hardware malfunction.
    HARDWARE_MALFUNCTION { message: String },
    /// Alexa does not have permissions to perform the specified action on the endpoint.
    INSUFFICIENT_PERMISSIONS { message: String },
    /// An error occurred that can't be described by one of the other error types. For example, a runtime exception occurred. We recommend that you always send a more specific error type.
    INTERNAL_ERROR { message: String },
    /// The authorization credential provided by Alexa is invalid. For example, the OAuth2 access token is not valid for the customer's device cloud account.
    INVALID_AUTHORIZATION_CREDENTIAL { message: String },
    /// The directive is not supported by the skill, or is malformed.
    INVALID_DIRECTIVE { message: String },
    /// The directive contains a value that is not valid for the target endpoint. For example, an invalid heating mode, channel, or program value.
    INVALID_VALUE { message: String },
    /// The endpoint does not exist, or no longer exists.
    NO_SUCH_ENDPOINT { message: String },
    /// The endpoint can't handle the directive because it is in a calibration phase, such as warming up.
    NOT_CALIBRATED { message: String },
    /// The endpoint can't be set to the specified value because of its current mode of operation. When you send this error response, include a field in the payload named current_device_mode that indicates why the device cannot be set to the new value. Valid values are COLOR, ASLEEP, NotProvisioned, OTHER.
    NOT_SUPPORTED_IN_CURRENT_MODE {
        message: String,
        #[serde(rename = "currentDeviceMode")]
        current_device_mode: NotSupportedInCurrentModeEnum,
    },
    /// The endpoint is not in operation. For example, a smart home skill can return a NOT_IN_OPERATION error when it receives a RESUME directive but the endpoint is the OFF mode.
    NOT_IN_OPERATION { message: String },
    /// The endpoint can't handle the directive because it doesn't support the requested power level.
    POWER_LEVEL_NOT_SUPPORTED { message: String },
    /// The maximum rate at which an endpoint or bridge can process directives has been exceeded.
    RATE_LIMIT_EXCEEDED { message: String },
    /// The endpoint can't be set to the specified value because it's outside the acceptable range. For example, you can use this error when a customer requests a percentage value over 100. For temperature values, use TEMPERATURE_VALUE_OUT_OF_RANGE instead. When you send this error response, optionally include a validRange field in the payload that indicates the acceptable range. For more information, see examples.
    VALUE_OUT_OF_RANGE { message: String },
    // Specific errors for Temperature
    ///	Indicates temperature value is outside the allowable range. This error is in the Alexa namespace. A validRange object that contains two Temperature objects named minimumValue and maximumValue, that indicate the minimum and maximum temperature settings for the thermostat.
    TEMPERATURE_VALUE_OUT_OF_RANGE { message: String },
    ///	Setpoints are too close together. Includes a Temperature object named minimum_temperature_delta that indicates the minimum allowable difference between setpoints.
    REQUESTED_SETPOINTS_TOO_CLOSE {
        message: String,
        #[serde(rename = "minimumTemperatureDelta")]
        minimum_temperature_delta: TemperaturePropertyValue,
    },
    ///	Thermostat is off and cannot be turned on.	None
    THERMOSTAT_IS_OFF { message: String },
    ///	The thermostat doesn't support the specified mode.	None
    UNSUPPORTED_THERMOSTAT_MODE { message: String },
    ///	The thermostat doesn't support dual setpoints in the current mode.	None
    DUAL_SETPOINTS_UNSUPPORTED { message: String },
    ///	The thermostat doesn't support triple setpoints in the current mode.	None
    TRIPLE_SETPOINTS_UNSUPPORTED { message: String },
    ///	You cannot set the requested schedule.	None
    UNWILLING_TO_SET_SCHEDULE { message: String },
    ///	You cannot set the value because it may cause damage to the device or appliance.	None
    UNWILLING_TO_SET_VALUE { message: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum NotSupportedInCurrentModeEnum {
    #[serde(rename = "COLOR")]
    Color,
    #[serde(rename = "ASLEEP")]
    Asleep,
    #[serde(rename = "NotProvisioned")]
    NotProvisioned,
    #[serde(rename = "OTHER")]
    Other,
}

/// Send an ErrorResponse if you cannot process a directive as expected.
pub type ErrorResponse = super::event::Response<super::event::EmptyPayload, super::event::Event<ErrorType>>;

impl ErrorResponse {
    pub fn new(
        message_id: String,
        correlation_token: Option<String>,
        endpoint: Endpoint,
        error: ErrorType,
    ) -> Self {
        ErrorResponse {
            context: None,
            event: super::event::Event {
                header: Header {
                    namespace: "Alexa".into(),
                    name: "ErrorResponse".to_owned(),
                    payload_version: PAYLOAD_VERSION.to_string(),
                    message_id,
                    correlation_token,
                },
                endpoint,
                payload: error,
            },
        }
    }
}
