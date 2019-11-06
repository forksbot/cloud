//! The Alexa.EqualizerController interface provides directives that are used to set or adjust the equalizer bands
//! and apply a sound mode to a smart entertainment device.
//! Implement this interface for devices that can set and adjust one or more equalizer bands to any integer value in a continuous range of values.

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::{
    directive::{Command, Directive},
    event::{
        discovery::{DeviceEndpoint, Capability, CapabilityAdditional},
        properties::{Property, PropertyName},
    },
    property_types::{EqualizerBandsPropertyValue, EqualizerBandsDeltaValue}
};

const INTERFACE_NAME: &'static str = "Alexa.EqualizerController";
const SUPPORTED_PROPERTIES: &'static [&'static str] = &["bands", "modes"];

/// The configurations object contains the configuration of available bands, including range values and/or the sound modes supported by the endpoint.
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct EqualizerConfiguration {
    /// Contains the bands supported by this endpoint.
    pub bands: EqualizerBands,
    /// Contains a list of equalizer modes supported by this endpoint.
    pub modes: EqualizerModes,
}

/// Contains a list of equalizer modes supported by this endpoint.
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct EqualizerModes {
    /// List of objects that specify a name attribute for the mode.
    pub supported: Vec<EqualizerModesSupported>,
}


/// Contains the bands supported by this endpoint.
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct EqualizerBands {
    /// List of objects that specify a name attribute for the band.
    pub supported: Vec<EqualizerBandsSupported>,
    /// An object that specifies a minimum and maximum value
    pub range: EqualizerBandRange,
}

/// An object that specifies a minimum and maximum value
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct EqualizerBandRange {
    /// The minimum value that can be set for this band.
    pub minimum: i64,
    /// The maximum value that can be set for this band.
    pub maximum: i64,
}

/// List of objects that specify a name attribute for the band.
/// A list of objects in the format: "name": "bandName", valid values for bandName are BASS, MIDRANGE, TREBLE.
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
#[serde(tag = "name")]
pub enum EqualizerBandsSupported {
    BASS,
    MIDRANGE,
    TREBLE,
}

/// List of objects that specify a name attribute for the mode.
/// A list of objects in the format: "name": "modeName".
/// Valid values for modeName are MOVIE, MUSIC, NIGHT, SPORT, TV.
/// https://developer.amazon.com/de/docs/device-apis/alexa-equalizercontroller.html#discovery
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
#[serde(tag = "name")]
pub enum EqualizerModesSupported {
    MOVIE,
    MUSIC,
    NIGHT,
    SPORT,
    TV,
}

/// Request to set the sound mode for an endpoint.
/// Used by the SetMode directive as payload.
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct SetMode {
    /// Name of the mode or equalizer mode support by the endpoint.
    pub mode: EqualizerModesSupported,
}


/// Add Alexa.EqualizerController capability to a device endpoint for discovery responses
/// assuming support for "modes" and "bands".
pub fn add_to_endpoint(endpoint: &mut DeviceEndpoint, configurations: EqualizerConfiguration) {
    endpoint.capabilities.push(Capability::new(CapabilityAdditional::EqualizerController { configurations }));
}

/// Add Alexa.EqualizerController property to response or report
#[inline]
pub fn add_to_response_context(properties: &mut Vec<Property>, brightness: i64) {
    properties.push(Property::new(INTERFACE_NAME, PropertyName::Brightness { value: brightness }));
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveSetBands {
   bands: Vec<EqualizerBandsPropertyValue>
}

impl Directive for DirectiveSetBands { const NAME: &'static str = "SetBands"; }

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveAdjustBands {
    bands: EqualizerBandsDeltaValue
}

impl Directive for DirectiveAdjustBands { const NAME: &'static str = "AdjustBands"; }

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveResetBands {
    bands: EqualizerBandsSupported
}

impl Directive for DirectiveResetBands { const NAME: &'static str = "ResetBands"; }

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveSetMode {
    mode: SetMode
}

impl Directive for DirectiveSetMode { const NAME: &'static str = "SetMode"; }
