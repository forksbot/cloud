//! The Alexa.BrightnessController capability interface describes messages used to control the brightness of an endpoint such as a light bulb.
//! Use this interface for devices that you know support brightness control instead of the more general Alexa.PercentageController interface.
//! If you want to control the power level of an endpoint such as a dimmer switch, use the Alexa.PowerLevelController interface instead.

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::{
    utils_serde::ArrayOfStaticStrings,
    directive::{Command, Directive},
    event::{
        discovery::{DeviceEndpoint, Capability, CapabilityAdditional, CapabilityProperties},
        properties::{Property, PropertyName},
        self
    },
    {Endpoint, Header}
};

const INTERFACE_NAME: &'static str = "Alexa.RTCSessionController";

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct RTCConfiguration {
    #[serde(rename = "isFullDuplexAudioSupported")]
    is_full_duplex_audio_supported: bool
}

/// used by the Connected and Disconnected responses
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct RTCResponsePayload {
    /// The identifier for the session from the original InitiateSessionWithOffer directive.
    #[serde(rename = "sessionId")]
    pub session_id: uuid::Uuid
}

/// Used by the AnswerGeneratedForSession responses
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct RTCAnswerPayload {
    /// An SDP answer.
    #[serde(rename = "sessionId")]
    pub answer: Answer
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "format")]
pub enum Offer {
    /// The RTCSessionController interface uses the Session Description Protocol (SDP).
    SDP {
        /// an SDP offer value.
        /// https://developer.amazon.com/de/docs/device-apis/alexa-rtcsessioncontroller.html#SDP
        value: String
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "format")]
pub enum Answer {
    /// The RTCSessionController interface uses the Session Description Protocol (SDP).
    SDP {
        /// an SDP answer value.
        /// https://developer.amazon.com/de/docs/device-apis/alexa-rtcsessioncontroller.html#SDP
        value: String
    }
}

/// Add Alexa.RTCSessionController capability to a device endpoint for discovery responses
pub fn add_to_endpoint(endpoint: &mut DeviceEndpoint, is_full_duplex_audio_supported: bool) {
    endpoint.capabilities.push(Capability::new(CapabilityAdditional::RTCSessionController {
        configuration: RTCConfiguration { is_full_duplex_audio_supported }
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