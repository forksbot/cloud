//! The Alexa.InputController interface describes messages for changing the input of an entertainment device.

use serde::{Serialize, Deserialize};

use crate::{
    property_types::ChannelPropertyValue,
    directive::{Command, Directive},
    event::{
        discovery::{DeviceEndpoint, Capability, CapabilityAdditional, CapabilityProperties},
        properties::{Property, PropertyName},
    },
    utils_serde::ArrayOfStaticStrings,
    property_types::CapabilityResource
};

#[derive(Serialize, Deserialize)]
pub struct ModeConfiguration {
    /// True if the mode values have an order; otherwise, false.
    /// For example, a wash temperature mode could have values ordered from cold, to warm, to hot.
    /// Only modes that are ordered support the adjustMode directive.
    pub ordered: bool,
    /// The values that are accepted for the mode. Ordered mode values should be listed in increasing order.
    #[serde(rename = "supportedModes")]
    pub supported_modes: Vec<SupportedMode>,
}

#[derive(Serialize, Deserialize)]
pub struct SupportedMode {
    pub value: String,
    #[serde(rename = "modeResources")]
    pub mode_resources: CapabilityResource,
}


const INTERFACE_NAME: &'static str = "Alexa.ModeController";

/// Add Alexa.ModeController property to response or report
/// # Arguments
/// * mode: String identifying the new input device. For example, HDMI 1.
pub fn add_to_response_context(properties: &mut Vec<Property>, mode: String) {
    properties.push(Property::new(INTERFACE_NAME, PropertyName::Mode { value: mode }));
}

/// Add Alexa.ModeController capability to a device endpoint for discovery responses
#[inline]
pub fn add_to_endpoint(endpoint: &mut DeviceEndpoint, properties: CapabilityProperties,
                       capability_resources: CapabilityResource,
                       configuration: ModeConfiguration) {
    endpoint.capabilities.push(Capability::new(CapabilityAdditional::ModeController { properties, capability_resources, configuration }));
}

/// Support the SetMode directive so that customers can set the mode of a device.
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveSetMode {
    /// The mode to set for the device.
    pub mode: String
}

impl Directive for DirectiveSetMode { const NAME: &'static str = "SetMode"; }

/// Support the AdjustMode directive so that customers can adjust the mode of a device.
///
/// For modes that are ordered, customers can increase or decrease the mode by a specified delta.
/// This directive does not restrict requests based on the current mode.
/// That means you can support wrapped modes by appropriately handling requests to increase and decrease the mode.
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveAdjustMode {
    /// The amount by which to change the mode. The default is 1.
    #[serde(rename = "modeDelta")]
    pub mode_delta: u64
}

impl Directive for DirectiveAdjustMode { const NAME: &'static str = "AdjustMode"; }
