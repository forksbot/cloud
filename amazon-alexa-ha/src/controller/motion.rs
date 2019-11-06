//! The Alexa.MotionSensor interface describes the properties and events used to report the state of an endpoint that senses physical movement in an area.
//! For example, the MotionSensor capability can be implemented by a motion sensor in a front yard.
//!
//! If your motion sensors are components of a larger security system,
//! your skill can also implement the Alexa.SecurityPanelController interface for a unified customer experience.

use serde::{Serialize, Serializer};
use chrono::{DateTime, Utc};

use crate::{
    event::{
        discovery::{DeviceEndpoint, Capability, CapabilityAdditional, CapabilityProperties},
        properties::{Property, PropertyName},
    },
    directive::Command,
    property_types::DetectionStatePropertyValue,
    utils_serde::ArrayOfStaticStrings
};

const INTERFACE_NAME: &'static str = "Alexa.MotionSensor";
const SUPPORTED_PROPERTIES: &'static [&'static str] = &["detectionState"];

/// Add Alexa.MotionSensor capability to a device endpoint for discovery responses
pub fn add_to_endpoint(endpoint: &mut DeviceEndpoint, proactively_reported: bool) {
    endpoint.capabilities.push(Capability::new(CapabilityAdditional::ContactSensor {
        properties: CapabilityProperties {
            supported: ArrayOfStaticStrings(SUPPORTED_PROPERTIES),
            proactively_reported,
            retrievable: true,
            non_controllable: false
        }
    }));
}
/// Add Alexa.MotionSensor property to response or report
#[inline]
pub fn add_to_response_context(properties: &mut Vec<Property>, value: DetectionStatePropertyValue) {
    properties.push(Property::new(INTERFACE_NAME, PropertyName::DetectionState { value }));
}
