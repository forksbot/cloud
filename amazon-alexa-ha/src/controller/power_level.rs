//! The Alexa.PowerLevelController capability interface describes the messages used to control the power level of a device.
//! Use this interface for devices that support power-level control, such as a dimmer switch.
//! For more general properties that can be expressed as a percentage, implement the Alexa.PercentageController interface instead.

use serde::Deserialize;
use chrono::{DateTime, Utc};

use crate::{
    event::{
        discovery::{DeviceEndpoint, Capability, CapabilityAdditional, CapabilityProperties},
        properties::{Property, PropertyName},
    },
    directive::{Command, Directive},
    utils_serde::ArrayOfStaticStrings
};

const INTERFACE_NAME: &'static str = "Alexa.PowerLevelController";
const SUPPORTED_PROPERTIES: &'static [&'static str] = &["powerLevel"];

/// Add Alexa.PowerLevelController property to response or report
pub fn add_to_response_context(properties: &mut Vec<Property>, level: u64) {
    properties.push(Property::new(INTERFACE_NAME, PropertyName::PowerLevel { value: level }));
}

/// Add Alexa.PowerLevelController capability to a device endpoint for discovery responses
pub fn add_to_endpoint(endpoint: &mut DeviceEndpoint, proactively_reported: bool) {
    endpoint.capabilities.push(Capability::new(CapabilityAdditional::PowerLevelController {
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
pub struct DirectiveSetPowerLevel {
    #[serde(rename = "powerLevel")]
    pub power_level: u64
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveAdjustPowerLevel {
    #[serde(rename = "powerLevelDelta")]
    pub power_level_delta: i64
}

impl Directive for DirectiveSetPowerLevel { const NAME: &'static str = "SetPowerLevel"; }
impl Directive for DirectiveAdjustPowerLevel { const NAME: &'static str = "AdjustPowerLevel"; }
