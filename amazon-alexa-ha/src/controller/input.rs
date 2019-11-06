//! The Alexa.InputController interface describes messages for changing the input of an entertainment device.

use serde::{Serialize, Deserialize};
use crate::{
    property_types::ChannelPropertyValue,
    directive::{Command, Directive},
    event::{
        discovery::{DeviceEndpoint, Capability, CapabilityAdditional},
        properties::{Property, PropertyName},
    },
    utils_serde::ArrayOfStaticStrings,
};

const INTERFACE_NAME: &'static str = "Alexa.InputController";

/// Add Alexa.InputController property to response or report
/// # Arguments
/// * input: String identifying the new input device. For example, HDMI 1.
pub fn add_to_response_context(properties: &mut Vec<Property>, input: String) {
    properties.push(Property::new(INTERFACE_NAME, PropertyName::Input { value: input }));
}

/// Add Alexa.InputController capability to a device endpoint for discovery responses
#[inline]
pub fn add_to_endpoint(endpoint: &mut DeviceEndpoint, inputs: ArrayOfStaticStrings) {
    endpoint.capabilities.push(Capability::new(CapabilityAdditional::InputController { inputs }));
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveSelectInput {
    /// The name of the input. The name must be unique across inputs for this endpoint.
    pub input: String
}

impl Directive for DirectiveSelectInput { const NAME: &'static str = "SelectInput"; }
