//! The Alexa.ColorController capability interface describes the messages used to change the color of an endpoint such as a color-changing light bulb.

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::{
    property_types::ColorPropertyValue,
    directive::{Command, Directive},
    event::{
        discovery::{DeviceEndpoint, Capability, CapabilityAdditional, CapabilityProperties},
        properties::{Property, PropertyName},
    },
    utils_serde::ArrayOfStaticStrings
};

const INTERFACE_NAME: &'static str = "Alexa.ColorController";
const SUPPORTED_PROPERTIES: &'static [&'static str] = &["color"];

/// Add Alexa.ColorController capability to a device endpoint for discovery responses
pub fn add_to_endpoint(endpoint: &mut DeviceEndpoint, proactively_reported: bool) {
    endpoint.capabilities.push(Capability::new(CapabilityAdditional::ColorController {
        properties: CapabilityProperties {
            supported: ArrayOfStaticStrings(SUPPORTED_PROPERTIES),
            proactively_reported,
            retrievable: true,
            non_controllable: false
        }
    }));
}

/// Add Alexa.ColorController property to response or report
#[inline]
pub fn add_to_response_context(properties: &mut Vec<Property>, color: ColorPropertyValue) {
    properties.push(Property::new(INTERFACE_NAME, PropertyName::Color { value: color }));
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveSetColor {
    pub color: ColorPropertyValue,
}

impl Directive for DirectiveSetColor { const NAME: &'static str = "SetColor"; }
