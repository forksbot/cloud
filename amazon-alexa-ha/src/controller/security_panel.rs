//! The Alexa.SecurityPanelController interface describes messages that you can use to develop Alexa skills for security systems.
//! Customers can use your skill to arm and disarm their security system, and you can report alarm conditions to your customers.
//!
//! If your security system uses contact and motion sensors, your skill can also implement
//! the Alexa.ContactSensor and Alexa.MotionSensor interfaces for a unified customer experience.
//!
//! https://developer.amazon.com/de/docs/device-apis/alexa-securitypanelcontroller.html


use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::{
    utils_serde::ArrayOfStaticStrings,
    directive::{Command, Directive},
    event::{
        discovery::{DeviceEndpoint, Capability, CapabilityAdditional, CapabilityProperties},
        properties::{Property, PropertyName},
        self,
    },
    {Endpoint, Header},
    property_types::AlarmPropertyValueTagged
};

const INTERFACE_NAME: &'static str = "Alexa.SecurityPanelController";
const SUPPORTED_PROPERTIES: &'static [&'static str] = &["armState", "burglaryAlarm", "fireAlarm", "waterAlarm"];

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct SecurityPanelConfiguration {
    /// The states that the security system supports. By default, Alexa assumes that the security system supports all states.
    #[serde(rename = "supportedAuthorizationTypes")]
    pub supported_authorization_types: Vec<SupportedAuthorizationType>,
    /// Use this field only when your skill controls a security system that supports four digit PIN codes and your backend systems can validate those PIN codes.
    /// The only valid type is FOUR_DIGIT_PIN.
    #[serde(rename = "supportedArmStates")]
    pub supported_arm_states: Vec<SupportedArmStateTagged>,
}

/// Supported arm states. This is the tagged variant and produces serialized json code like so:
/// {"value": "ARMED_AWAY"}
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
#[serde(tag = "value")]
pub enum SupportedArmStateTagged {
    /// Indicates that the security system is active and the occupants are away.
    #[serde(rename = "ARMED_AWAY")]
    ArmedAway,
    /// Indicates that the security system is active and the occupants are present.
    #[serde(rename = "ARMED_STAY")]
    ArmedStay,
    /// Indicates that the security system is active and the occupants are sleeping.
    #[serde(rename = "ARMED_NIGHT")]
    ArmedNight,
    /// Indicates that the security system is not active.
    DISARMED,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub enum SupportedArmState {
    /// Indicates that the security system is active and the occupants are away.
    #[serde(rename = "ARMED_AWAY")]
    ArmedAway,
    /// Indicates that the security system is active and the occupants are present.
    #[serde(rename = "ARMED_STAY")]
    ArmedStay,
    /// Indicates that the security system is active and the occupants are sleeping.
    #[serde(rename = "ARMED_NIGHT")]
    ArmedNight,
    /// Indicates that the security system is not active.
    DISARMED,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
#[serde(tag = "value")]
pub enum SupportedAuthorizationType {
    #[serde(rename = "FOUR_DIGIT_PIN")]
    FourDigitPin,
}

/// Add Alexa.SecurityPanelController property to response or report
pub fn add_to_response_context_arm_state(properties: &mut Vec<Property>, value: SupportedArmState) {
    properties.push(Property::new(INTERFACE_NAME, PropertyName::ArmState { value }));
}

/// Indicates the current state of a burglary alarm
/// Add Alexa.SecurityPanelController property to response or report
pub fn add_to_response_context_burglary_alarm(properties: &mut Vec<Property>, value: AlarmPropertyValueTagged) {
    properties.push(Property::new(INTERFACE_NAME, PropertyName::BurglaryAlarm { value }));
}

/// Indicates the current state of a fire alarm
/// Add Alexa.SecurityPanelController property to response or report
pub fn add_to_response_context_fire_alarm(properties: &mut Vec<Property>, value: AlarmPropertyValueTagged) {
    properties.push(Property::new(INTERFACE_NAME, PropertyName::FireAlarm { value }));
}

/// Indicates the current state of a water alarm
/// Add Alexa.SecurityPanelController property to response or report
pub fn add_to_response_context_water_alarm(properties: &mut Vec<Property>, value: AlarmPropertyValueTagged) {
    properties.push(Property::new(INTERFACE_NAME, PropertyName::WaterAlarm { value }));
}


/// Add Alexa.SecurityPanelController capability to a device endpoint for discovery responses
pub fn add_to_endpoint(endpoint: &mut DeviceEndpoint, configuration: SecurityPanelConfiguration) {
    endpoint.capabilities.push(Capability::new(CapabilityAdditional::SecurityPanelController {
        configuration,
        properties: CapabilityProperties {
            supported: ArrayOfStaticStrings(SUPPORTED_PROPERTIES),
            proactively_reported,
            retrievable: true,
            non_controllable: false,
        },
    }));
}

/// Support the InitiateSessionWithOffer directive so that users can initiate a real-time communication session with a front door device.
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveInitiateSessionWithOffer {
    ///	The identifier of the session that wants to connect.
    #[serde(rename = "sessionId")]
    pub session_id: uuid::Uuid,
    ///	An SDP offer.
    pub offer: Offer,
}

/// The SessionConnected directive notifies you that your RTC session is connected.
///
///  Note: Alexa does not always send the SessionConnected directive.
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveSessionConnected {
    /// The identifier for the session from the original InitiateSessionWithOffer directive.
    #[serde(rename = "sessionId")]
    pub session_id: uuid::Uuid
}

/// The SessionDisconnected directive notifies you that your RTC session is disconnected.
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveSessionDisconnected {
    /// The identifier for the session from the original InitiateSessionWithOffer directive.
    #[serde(rename = "sessionId")]
    pub session_id: uuid::Uuid
}

impl Directive for DirectiveInitiateSessionWithOffer { const NAME: &'static str = "InitiateSessionWithOffer"; }

impl Directive for DirectiveSessionConnected { const NAME: &'static str = "SessionConnected"; }

impl Directive for DirectiveSessionDisconnected { const NAME: &'static str = "SessionDisconnected"; }

/// If you handle a InitiateSessionWithOffer directive successfully, respond with an SessionConnected event. You can respond synchronously or asynchronously.
pub type InitiateSessionWithOfferResponse = event::Response<event::EmptyPayload, event::Event<RTCAnswerPayload>>;

impl InitiateSessionWithOfferResponse {
    pub fn new(endpoint: Endpoint, answer: Answer) -> Self {
        let header = Header::new(INTERFACE_NAME, "AnswerGeneratedForSession");
        event::Response::new(event::Event::new(header, endpoint, RTCAnswerPayload { answer }))
    }
}

/// If you handle a SessionConnected directive successfully, respond with an SessionConnected event. You can respond synchronously or asynchronously.
pub type SessionConnectedResponse = event::Response<event::EmptyPayload, event::Event<RTCResponsePayload>>;

impl SessionConnectedResponse {
    pub fn new(endpoint: Endpoint, session_id: uuid::Uuid) -> Self {
        let header = Header::new(INTERFACE_NAME, "SessionConnected");
        event::Response::new(event::Event::new(header, endpoint, RTCResponsePayload { session_id }))
    }
}

/// If you handle a SessionDisconnected directive successfully, respond with an SessionDisconnected event. You can respond synchronously or asynchronously.
pub type SessionDisconnectedResponse = event::Response<event::EmptyPayload, event::Event<RTCResponsePayload>>;

impl SessionDisconnectedReport {
    pub fn new(endpoint: Endpoint, session_id: uuid::Uuid) -> Self {
        let header = Header::new(INTERFACE_NAME, "SessionDisconnected");
        event::Response::new(event::Event::new(header, endpoint, RTCResponsePayload { session_id }))
    }
}