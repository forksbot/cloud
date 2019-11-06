//! The Alexa.ColorTemperatureController capability interface describes the messages used to control the color temperature of an endpoint that supports tunable white light.
//!
//! The following table lists some color temperature values. If an endpoint does not support a value that a customer requests, we recommend that you set the endpoint to the nearest possible value.
//!
//!Shade of White	| Color Temperature in Kelvin
//! * warm, warm white          | 2200
//! * incandescent, soft white	| 2700
//! * white	                    | 4000
//! * daylight, daylight white  | 5500
//! * cool, cool white          | 7000

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::{
    directive::{Command, Directive},
    event::{
        discovery::{DeviceEndpoint, Capability, CapabilityAdditional, CapabilityProperties},
        properties::{Property, PropertyName},
    },
    property_types::ColorPropertyValue,
    utils_serde::ArrayOfStaticStrings
};

const INTERFACE_NAME: &'static str = "Alexa.ColorTemperatureController";
const SUPPORTED_PROPERTIES: &'static [&'static str] = &["colorTemperatureInKelvin"];

/// Add Alexa.ColorTemperatureController capability to a device endpoint for discovery responses
pub fn add_to_endpoint(endpoint: &mut DeviceEndpoint, proactively_reported: bool) {
    endpoint.capabilities.push(Capability::new(CapabilityAdditional::ColorTemperatureController {
        properties: CapabilityProperties {
            supported: ArrayOfStaticStrings(SUPPORTED_PROPERTIES),
            proactively_reported,
            retrievable: true,
            non_controllable: false
        }
    }));
}

/// Add Alexa.ColorTemperatureController property to response or report
#[inline]
pub fn add_to_response_context(properties: &mut Vec<Property>, color: u64) {
    properties.push(Property::new(INTERFACE_NAME, PropertyName::ColorTemperatureInKelvin { value: color }));
}

/// Extract the colorTemperatureInKelvin of a SetColorTemperature directive
pub fn color_temperature_in_kelvin(command: &Command) -> Option<u64> {
    command.get("colorTemperatureInKelvin").and_then(|v| v.as_u64())
}

#[derive(Deserialize)]
pub struct DirectiveIncreaseColorTemperature {}

impl Directive for DirectiveIncreaseColorTemperature { const NAME: &'static str = "IncreaseColorTemperature"; }

#[derive(Deserialize)]
pub struct DirectiveDecreaseColorTemperature {}

impl Directive for DirectiveDecreaseColorTemperature { const NAME: &'static str = "DecreaseColorTemperature"; }


#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveSetColorTemperature {
    #[serde(rename = "colorTemperatureInKelvin")]
    pub color_temperature_in_kelvin: u64,
}

impl Directive for DirectiveSetColorTemperature { const NAME: &'static str = "SetColorTemperature"; }
