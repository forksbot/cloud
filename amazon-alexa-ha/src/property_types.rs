//! # Capability Property Schemas
//! This module contains the schemas for properties that are used by the Alexa interfaces.

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use super::utils_serde::DurationISO8601;

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub enum ChannelPropertyValue {
    #[serde(rename = "number")]
    Number(String),
    #[serde(rename = "callSign")]
    CallSign(String),
    #[serde(rename = "affiliateCallSign")]
    AffiliateCallSign(String),
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct ColorPropertyValue {
    pub hue: f64,
    pub saturation: f64,
    pub brightness: f64,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub enum ConnectivityPropertyValue {
    OK,
    UNREACHABLE,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub enum EnablementModePropertyValue {
    ENABLED,
    DISABLED,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub enum EnumeratedPowerLevelPropertyValue {
    LOW,
    MEDIUM,
    HIGH,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub enum FeatureAvailabilityPropertyValue {
    /// Indicates that the feature is enabled.
    ENABLED,
    /// Indicates that the feature is disabled.
    DISABLED,
    /// Indicates that the feature is available, but the user must purchase a subscription before they can use the feature.
    SUBSCRIPTION_REQUIRED,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub enum LockStatePropertyValue {
    /// The device is currently locked.
    LOCKED,
    /// The device is currently unlocked.
    UNLOCKED,
    /// The lock can't transition to locked or unlocked because the locking mechanism is jammed.
    JAMMED,
}


#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub enum UpDownPropertyValue {
    UP,
    DOWN,
}

/// Used by the security panel controller. A condition is either ok or in alarm mode.
/// This is the tagged variant and produces json like so: {"value": "OK"}
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
#[serde(tag = "value")]
pub enum AlarmPropertyValueTagged {
    OK,
    ALARM,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct EqualizerBandsPropertyValue {
    /// The name of the equalizer band supported by the endpoint. Supported values: BASS, TREBLE, MIDRANGE.
    pub name: String,
    /// The discrete frequency value for the equalizer band.
    pub value: u64,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct EqualizerBandsDeltaValue {
    /// The name of the equalizer band supported by the endpoint. Supported values: BASS, TREBLE, MIDRANGE.
    pub name: String,
    /// Represents adjustment value for the specified equalizer band.
    #[serde(rename = "levelDelta")]
    pub level_delta: i64,
    /// Specifies how the band should be adjusted.
    #[serde(rename = "levelDirection")]
    pub level_direction: UpDownPropertyValue,
}


#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub enum PlaybackStateEnumPropertyValue {
    ///	The endpoint is playing the media.
    PLAYING,
    /// The endpoint paused the media.
    PAUSED,
    /// The endpoint is not playing the media.
    STOPPED,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub enum PowerStatePropertyValue {
    ON,
    OFF,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub enum RecordingStatePropertyValue {
    RECORDING,
    NOT_RECORDING,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub enum DetectionStatePropertyValue {
    DETECTED,
    NOT_DETECTED,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TemperatureScaleUnits {
    CELSIUS,
    FAHRENHEIT,
    KELVIN,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum ThermostatModes {
    ///	Indicates automatic heating or cooling based on the current temperature and the setpoint.
    AUTO,
    ///	Indicates cooling mode.
    COOL,
    ///	Indicates heating mode.
    HEAT,
    ///	Indicates economical mode.
    ECO,
    ///	Indicates that heating and cooling is turned off, but the device may still have power.
    OFF,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum VolumeUnit {
    ///	Metric	ISO Standard unit of volume. Can be sent to Alexa as LITRE.
    LITER,
    ///	Metric	1/1000 LITER. Can be sent to Alexa as MILLILITRE.
    MILLILITER,
    ///	Metric	5 MILLILITER.
    TEASPOON,
    ///	Imperial	Exactly 4.54609 liters
    UK_GALLON,
    ///	U.S. Customary	Exactly 3.785411784 liters
    US_FLUID_GALLON,
    ///	U.S. Customary	1/128 US_FLUID_GALLON
    US_FLUID_OUNCE,
    ///	U.S. Customary	Exactly 4.40488377086 liters
    US_DRY_GALLON,
    ///	U.S. Customary	1/128 US_DRY_GALLON
    US_DRY_OUNCE,
    ///	Metric	15 MILLILITER, also equal to 3 TEASPOON
    UK_TABLESPOON,
    ///	Metric	20 MILLILITER, also equal to 4 TEASPOON
    AU_TABLESPOON,
    /// or CUBIC_CENTIMETRE	Metric	1 MILLILITER. Note that Alexa recognizes both spellings.
    CUBIC_CENTIMETER,
    /// or CUBIC_METRE	Metric	1000 LITER. Note that Alexa recognizes both spellings.
    CUBIC_METER,
    ///	Imperial	1/160 UK_GALLON
    UK_OUNCE,
    ///	Imperial	1/4 UK_GALLON (2 UK_PINT)
    UK_QUART,
    ///	Imperial	1/8 UK_GALLON (2 UK_CUP)
    UK_PINT,
    ///	Imperial	1/16 UK_GALLON (2 UK_GILL)
    UK_CUP,
    ///	Imperial	1/32 UK_GALLON (5 UK_OUNCE)
    UK_GILL,
    ///	Imperial	1/8 UK_OUNCE
    UK_DRAM,
    ///	U.S. Customary	1/4 US_FLUID_GALLON
    US_FLUID_QUART,
    ///	U.S. Customary	1/8 US_FLUID_GALLON
    US_FLUID_PINT,
    ///	U.S. Customary	1/16 US_FLUID_GALLON
    US_FLUID_CUP,
    ///	U.S. Customary	1/2 US_FLUID_OUNCE
    US_TABLESPOON,
    ///	U.S. Customary	1/6 US_FLUID_OUNCE
    US_TEASPOON,
    ///	U.S. Customary	1/8 US_FLUID_OUNCE
    US_DRAM,
    ///	U.S. Customary	1/4 US_DRY_GALLON
    US_DRY_QUART,
    ///	U.S. Customary	1/8 US_DRY_GALLON
    US_DRY_PINT,
    ///	U.S. Customary	1/16 US_DRY_GALLON
    US_DRY_CUP,
    ///	Imperial & U.S. Customary	Defined as exactly 16.387064 MILLILITER. Also 1/231 US_FLUID_GALLON
    CUBIC_INCH,
    ///	Imperial & U.S. Customary	Defined as exactly 28.316846592 LITER. Also 1728 CUBIC_INCH or 576⁄77 US_FLUID_GALLON.
    CUBIC_FOOT,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum WeightUnit {
    ///	Metric	1/1000 KILOGRAM
    GRAM,
    ///	Metric	ISO Standard unit of mass/weight
    KILOGRAM,
    ///	Imperial	1/16 POUND.
    OUNCE,
    ///	Imperial	Exactly 0.45359237 KILOGRAM
    POUND,
    ///	Metric	500 GRAM
    METRIC_POUND,
    ///	Metric	1/1000 MILLIGRAM
    MICROGRAM,
    ///	Metric	1/1000 GRAM
    MILLIGRAM,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct PlaybackStatePropertyValue {
    /// Use playbackState to indicate the state of an endpoint that plays media.
    pub state: PlaybackStateEnumPropertyValue,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TemperaturePropertyValue {
    /// The temperature.
    pub value: f64,
    /// The scale of the temperature.
    pub scale: TemperatureScaleUnits,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct TimeInterval {
    /// The start time for this time interval
    pub start: Option<chrono::DateTime<Utc>>,
    /// The end time for this time interval
    pub end: Option<chrono::DateTime<Utc>>,
    /// The time period of this time interval
    pub duration: Option<DurationISO8601>,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct WeightPropertyValue {
    /// The weight in the specified units.
    pub value: f64,
    /// An string enumeration value that indicates the unit of measure.
    pub unit: WeightUnit,
}

/// Use capabilityResources to provide a set of friendlyNames for the ToggleController, RangeController, and ModeController interfaces.
/// https://developer.amazon.com/de/docs/device-apis/resources-and-assets.html#capability-resources
#[derive(Serialize, Deserialize)]
pub struct CapabilityResource {
    /// Friendly names that customers can use to interact with.
    /// * When @type is asset, contains an AssetString that references an item from a localized catalog of strings.
    /// * When @type is text, contains a TextString to represent a literal string value.
    #[serde(rename = "friendlyNames")]
    pub friendly_names: Vec<FriendlyName>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum FriendlyName {
    #[serde(rename = "asset")]
    Asset { value: FriendlyNameAsset },
    #[serde(rename = "text")]
    Text { value: FriendlyNameText },
}

#[derive(Serialize, Deserialize)]
pub struct FriendlyNameText {
    /// The literal representation of a string.
    pub text: String,
    /// The locale in which the string is localized. Currently, the only supported value is en-US.
    pub locale: Locale,
}

#[derive(Serialize, Deserialize)]
pub enum Locale {
    #[serde(rename = "en-US")]
    EnUs
}

#[derive(Serialize, Deserialize)]
pub struct FriendlyNameAsset {
    /// The ID of the localized string in the global or skill catalog.
    #[serde(rename = "assetId")]
    pub asset_id: String,
}
