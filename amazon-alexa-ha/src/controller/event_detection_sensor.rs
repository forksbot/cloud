//! Support the Alexa.EventDetectionSensor interface so that your camera devices can notify Alexa when they detect the presence of a person.
//! When your camera detects a person, you report that information to Alexa in a change report, and Alexa notifies your user.
//! Users can set up notifications and routines for person detection in the Alexa app.
//! By setting up person detection, users can reduce the number of notifications they receive when tracking all motion detection.
//!
//! You can also support the Alexa.MediaMetadata Interface to enable Alexa to search and display the media clip of the person that is detected.
//! Users can ask Alexa questions like "Alexa, show the last person detected at my front door."

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::{
    event::{
        discovery::{DeviceEndpoint, Capability, CapabilityAdditional, CapabilityProperties},
        properties::{Property, PropertyName},
    },
    directive::Command,
    property_types::{ChannelPropertyValue, DetectionStatePropertyValue},
    utils_serde::ArrayOfStaticStrings
};

const INTERFACE_NAME: &'static str = "Alexa.EventDetectionSensor";
const SUPPORTED_PROPERTIES: &'static [&'static str] = &["value", "detectionMethods", "media"];

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct EventDetectionSensor {
    #[serde(rename = "detectionMethods")]
    pub detection_methods: Vec<String>,
    #[serde(rename = "detectionModes")]
    pub detection_modes: DetectionModes,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetectionModes {
    #[serde(rename = "humanPresence")]
    pub human_presence: HumanPresence,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HumanPresence {
    #[serde(rename = "featureAvailability")]
    pub feature_availability: String,
    #[serde(rename = "supportsNotDetected")]
    pub supports_not_detected: bool,
}


#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub enum DetectionStateMedia {
    AUDIO,
    VIDEO,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct HumanPresenceDetectionStateValue {
    /// Whether presence is detected or not;
    pub value: DetectionStatePropertyValue,
    /// The methods that the endpoint uses to detect events; AUDIO, VIDEO. The default value is ["AUDIO", "VIDEO"].
    #[serde(rename = "detectionMethods")]
    pub detection_methods: Option<Vec<DetectionStateMedia>>,
    /// The identifier of the audio or video recorded for the detection event.
    pub media: Option<HumanPresenceDetectionStateMediaValue>,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct HumanPresenceDetectionStateMediaValue {
    /// Represents adjustment value for the specified equalizer band.
    #[serde(rename = "type")]
    pub media_type: String,
    /// Specifies how the band should be adjusted.
    pub id: String,
}


/// Add Alexa.EventDetectionSensor capability to a device endpoint for discovery responses
pub fn add_to_endpoint(endpoint: &mut DeviceEndpoint, configuration: EventDetectionSensor, proactively_reported: bool) {
    endpoint.capabilities.push(Capability::new(CapabilityAdditional::EventDetectionSensor {
        configuration,
        properties: CapabilityProperties {
            supported: ArrayOfStaticStrings(SUPPORTED_PROPERTIES),
            proactively_reported,
            retrievable: true,
            non_controllable: false
        },
    }));
}

/// Add Alexa.EventDetectionSensor property to response or report
#[inline]
pub fn add_to_response_context(properties: &mut Vec<Property>, channel: ChannelPropertyValue) {
    properties.push(Property::new(INTERFACE_NAME, PropertyName::Channel { value: channel }));
}
