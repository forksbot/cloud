//! The Alexa.RangeController capability interface describes messages used to control the settings
//! of an endpoint that are represented by numbers within a minimum and maximum range.
//! You can use the RangeController interface to model properties of an endpoint that can be set to one of a range of values,
//! such as the speed settings on a blender or a fan.
//!
//! The RangeController interface is highly configurable and enables you to model many different kinds of settings for many different kinds of devices.
//! Use one of the following more specific interfaces if it's appropriate for your device:
//!
//! * Alexa.PowerLevelController
//! * Alexa.PercentageController
//! * Alexa.BrightnessController
//! * Alexa.EqualizerController
//! * Alexa.StepSpeaker
//! * Alexa.Speaker
//! You can also use the RangeController interface to model properties of an endpoint that customers can't change as described in the discovery section.

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::{
    utils_serde::ArrayOfStaticStrings,
    directive::{Command, Directive},
    event::{
        discovery::{DeviceEndpoint, Capability, CapabilityAdditional, CapabilityProperties},
        properties::{Property, PropertyName},
    },
    property_types::{FriendlyName, CapabilityResource}
};

const INTERFACE_NAME: &'static str = "Alexa.RangeController";
const SUPPORTED_PROPERTIES: &'static [&'static str] = &["rangeValue"];

#[derive(Serialize, Deserialize)]
pub struct RangeConfiguration {
    #[serde(rename = "supportedRange")]
    pub supported_range: SupportedRange,
    pub presets: Vec<Preset>,
}

#[derive(Serialize, Deserialize)]
pub struct Preset {
    #[serde(rename = "rangeValue")]
    pub range_value: i64,
    #[serde(rename = "presetResources")]
    pub preset_resources: PresetResources,
}

#[derive(Serialize, Deserialize)]
pub struct PresetResources {
    #[serde(rename = "friendlyNames")]
    pub friendly_names: Vec<FriendlyName>,
}

#[derive(Serialize, Deserialize)]
pub struct SupportedRange {
    #[serde(rename = "minimumValue")]
    pub minimum_value: i64,
    #[serde(rename = "maximumValue")]
    pub maximum_value: i64,
    pub precision: i64,
}


/// Add Alexa.RangeController property to response or report
pub fn add_to_response_context(properties: &mut Vec<Property>, level: u64) {
    properties.push(Property::new(INTERFACE_NAME, PropertyName::RangeValue { value: level }));
}

/// Add Alexa.RangeController capability to a device endpoint for discovery responses
pub fn add_to_endpoint(endpoint: &mut DeviceEndpoint, properties: CapabilityProperties,
                       capability_resources: CapabilityResource,
                       configuration: RangeConfiguration) {
    endpoint.capabilities.push(Capability::new(CapabilityAdditional::RangeController { properties, capability_resources, configuration }));
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveSetRangeValue {
    #[serde(rename = "rangeValue")]
    pub range_value: u64
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveAdjustRangeValue {
    /// The amount by which to change the feature of the device.
    /// If the user does not specify the amount, precision is used.
    #[serde(rename = "rangeValueDelta")]
    pub range_value_delta: i64,
    /// False if the user specified the amount of the change.
    #[serde(rename = "rangeValueDeltaDefault")]
    range_value_delta_default: bool
}

impl Directive for DirectiveSetRangeValue { const NAME: &'static str = "SetRangeValue"; }
impl Directive for DirectiveAdjustRangeValue { const NAME: &'static str = "AdjustRangeValue"; }
