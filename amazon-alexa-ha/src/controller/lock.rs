//! The Alexa.LockController capability interface describes the messages used to control lockable endpoints.

use serde::{Serialize, Deserialize};

use crate::{
    directive::{Command, Directive},
    event::{
        discovery::{DeviceEndpoint, Capability, CapabilityAdditional, CapabilityProperties},
        properties::{Property, PropertyName},
    },
    utils_serde::ArrayOfStaticStrings,
    property_types::LockStatePropertyValue
};

const INTERFACE_NAME: &'static str = "Alexa.LockController";
const SUPPORTED_PROPERTIES: &'static [&'static str] = &["lockState"];

/// Add Alexa.LockController property to response or report
pub fn add_to_response_context(properties: &mut Vec<Property>, value: LockStatePropertyValue) {
    properties.push(Property::new(INTERFACE_NAME, PropertyName::LockState { value }));
}

/// Add Alexa.LockController capability to a device endpoint for discovery responses
pub fn add_to_endpoint(endpoint: &mut DeviceEndpoint, proactively_reported: bool) {
    endpoint.capabilities.push(Capability::new(CapabilityAdditional::LockController {
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
pub struct DirectiveLock {}

impl Directive for DirectiveLock { const NAME: &'static str = "Lock"; }

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveUnlock {}

impl Directive for DirectiveUnlock { const NAME: &'static str = "Unlock"; }
