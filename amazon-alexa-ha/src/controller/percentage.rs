//! The Alexa.PercentageController capability interface describes the messages used to control properties of endpoints that can be expressed as a percentage.
//!
//! Use this interface only if there is not a more specific controller interface that applies to your device.
//! For example, if you want to handle requests specific to brightness values, implement the Alexa.BrightnessController interface instead.
//! If you want to handle requests specific to the power level of an endpoint such as a dimmer switch, implement the Alexa.PowerLevelController interface instead.

use serde::Deserialize;
use chrono::{DateTime, Utc};

use crate::{
    directive::{Command, Directive},
    event::{
        discovery::{DeviceEndpoint, Capability, CapabilityAdditional, CapabilityProperties},
        properties::{Property, PropertyName},
    },
    utils_serde::ArrayOfStaticStrings
};

const INTERFACE_NAME: &'static str = "Alexa.PercentageController";
const SUPPORTED_PROPERTIES: &'static [&'static str] = &["percentage"];

/// Add Alexa.PercentageController property to response or report
pub fn add_to_response_context(properties: &mut Vec<Property>, percentage: u64) {
    properties.push(Property::new(INTERFACE_NAME, PropertyName::Percentage { value: percentage }));
}

/// Add Alexa.PercentageController capability to a device endpoint for discovery responses
pub fn add_to_endpoint(endpoint: &mut DeviceEndpoint, proactively_reported: bool) {
    endpoint.capabilities.push(Capability::new(CapabilityAdditional::PercentageController {
        properties: CapabilityProperties {
            supported: ArrayOfStaticStrings(SUPPORTED_PROPERTIES),
            proactively_reported,
            retrievable: true,
            non_controllable: false
        }
    }));
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveSetPercentage {
    /// The percentage to set the device to. (0-100 inclusive)
    pub percentage: u64
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveAdjustPercentage {
    #[serde(rename = "percentageDelta")]
    /// The amount by which to change the percentage. (-100 to 100 inclusive)
    pub percentage_delta: i64
}

impl Directive for DirectiveSetPercentage { const NAME: &'static str = "SetPercentage"; }
impl Directive for DirectiveAdjustPercentage { const NAME: &'static str = "AdjustPercentage"; }
