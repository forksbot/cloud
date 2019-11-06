//! The Alexa.ChannelController interface exposes directives that are used to change or increment the channel for an entertainment device.

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::{
    property_types::ChannelPropertyValue,
    directive::{Command, Directive},
    event::{
        discovery::{DeviceEndpoint, Capability, CapabilityAdditional, CapabilityProperties},
        properties::{Property, PropertyName},
    },
    utils_serde::ArrayOfStaticStrings
};

const INTERFACE_NAME: &'static str = "Alexa.ChannelController";
const SUPPORTED_PROPERTIES: &'static [&'static str] = &["channel"];

/// Add Alexa.ChannelController capability to a device endpoint for discovery responses
pub fn add_to_endpoint(endpoint: &mut DeviceEndpoint, proactively_reported: bool) {
    endpoint.capabilities.push(Capability::new(CapabilityAdditional::ChannelController {
        properties: CapabilityProperties {
            supported: ArrayOfStaticStrings(SUPPORTED_PROPERTIES),
            proactively_reported,
            retrievable: true,
            non_controllable: false
        }
    }));
}

/// Add Alexa.ChannelController property to response or report
#[inline]
pub fn add_to_response_context(properties: &mut Vec<Property>, channel: ChannelPropertyValue) {
    properties.push(Property::new(INTERFACE_NAME, PropertyName::Channel { value: channel }));
}

/// Describes a channel.
/// number, channelMetadata.name, callSign, affiliateCallSign or uri must be specified.
#[derive(Debug, Serialize, Deserialize)]
pub struct Channel {
    /// A number that identifies the specified channel such as 5 or 12.1
    pub number: Option<String>,
    /// Specifies a channel by call sign such as PBS.
    #[serde(rename = "callSign")]
    pub call_sign: Option<String>,
    /// Specifies a channel by local affiliate call sign such as KCTS9.
    #[serde(rename = "affiliateCallSign")]
    pub affiliate_call_sign: Option<String>,
    /// The URI of the channel such as "entity://provider/channel/12307"
    pub uri: Option<String>,
}

/// Provides additional information about the specified channel.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelMetadata {
    /// Another value that identifies the channel such as "FOX".
    pub name: Option<String>,
    /// A URL to an image that describes the channel
    pub image: Option<String>,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveChangeChannel {
    /// Describes a channel.
    pub channel: Channel,
    /// Provides additional information about the specified channel.
    #[serde(rename = "channelMetadata")]
    pub channel_metadata: Option<Vec<ChannelMetadata>>,
}

impl Directive for DirectiveChangeChannel { const NAME: &'static str = "ChangeChannel"; }

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveSkipChannels {
    /// Provides additional information about the specified channel.
    #[serde(rename = "channelCount")]
    pub channel_count: u64,
}

impl Directive for DirectiveSkipChannels { const NAME: &'static str = "SkipChannels"; }
