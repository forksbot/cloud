//! The Alexa.BrightnessController capability interface describes messages used to control the brightness of an endpoint such as a light bulb.
//! Use this interface for devices that you know support brightness control instead of the more general Alexa.PercentageController interface.
//! If you want to control the power level of an endpoint such as a dimmer switch, use the Alexa.PowerLevelController interface instead.

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

const INTERFACE_NAME: &'static str = "Alexa.BrightnessController";
const SUPPORTED_PROPERTIES: &'static [&'static str] = &["brightness"];

/// Add Alexa.BrightnessController property to response or report
pub fn add_to_response_context(properties: &mut Vec<Property>, brightness: i64) {
    properties.push(Property::new(INTERFACE_NAME, PropertyName::Brightness { value: brightness }));
}

/// Add Alexa.BrightnessController capability to a device endpoint for discovery responses
pub fn add_to_endpoint(endpoint: &mut DeviceEndpoint, proactively_reported: bool) {
    endpoint.capabilities.push(Capability::new(CapabilityAdditional::BrightnessController {
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
pub struct DirectiveSetBrightness {
    pub brightness: u64
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveAdjustBrightness {
    #[serde(rename = "brightnessDelta")]
    pub brightness_delta: i64
}

impl Directive for DirectiveSetBrightness { const NAME: &'static str = "SetBrightness"; }
impl Directive for DirectiveAdjustBrightness { const NAME: &'static str = "AdjustBrightness"; }
