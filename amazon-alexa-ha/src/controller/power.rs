//! The Alexa.PowerController capability interface describes the messages used to control and report on the power state of a device.

use serde::{Serialize, Deserialize};

use crate::{
    directive::{Command, Directive},
    event::{
        discovery::{DeviceEndpoint, Capability, CapabilityAdditional, CapabilityProperties},
        properties::{Property, PropertyName},
    },
    utils_serde::ArrayOfStaticStrings,
    property_types::PowerStatePropertyValue
};

const INTERFACE_NAME: &'static str = "Alexa.PowerController";
const SUPPORTED_PROPERTIES: &'static [&'static str] = &["powerState"];

/// Add Alexa.PowerController property to response or report
pub fn add_to_response_context(properties: &mut Vec<Property>, value: PowerStatePropertyValue) {
    properties.push(Property::new(INTERFACE_NAME, PropertyName::PowerState { value }));
}

/// Add Alexa.PowerController capability to a device endpoint for discovery responses
pub fn add_to_endpoint(endpoint: &mut DeviceEndpoint, proactively_reported: bool) {
    endpoint.capabilities.push(Capability::new(CapabilityAdditional::PowerController {
        properties: CapabilityProperties {
            supported: ArrayOfStaticStrings(SUPPORTED_PROPERTIES),
            proactively_reported,
            retrievable: true,
            non_controllable: false
        }
    }));
}

/// Support the TurnOn directive so that customers can turn on devices.
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveTurnOn {}

impl Directive for DirectiveTurnOn { const NAME: &'static str = "TurnOn"; }

/// Support the TurnOff directive so that customers can turn off devices.
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveTurnOff {}

impl Directive for DirectiveTurnOff { const NAME: &'static str = "TurnOff"; }
