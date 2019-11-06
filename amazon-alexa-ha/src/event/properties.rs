use serde::{Deserialize, Serialize};
use crate::{
    property_types::*,
    controller::*
};
use chrono::{Utc, DateTime};

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize)]
pub struct Properties {
    pub properties: Vec<Property>,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize)]
#[serde(tag = "name")]
pub enum PropertyName {
    /// The Alexa.SecurityPanelController interface uses the armState property to represent the state of the system.
    #[serde(rename = "armState")]
    ArmState { value: security_panel::SupportedArmState },

    /// The Alexa.SecurityPanelController interface uses the armState property to represent the state of the system.
    #[serde(rename = "burglaryAlarm")]
    BurglaryAlarm { value: AlarmPropertyValueTagged },
    /// The Alexa.SecurityPanelController interface uses the armState property to represent the state of the system.
    #[serde(rename = "fireAlarm")]
    FireAlarm { value: AlarmPropertyValueTagged },
    /// The Alexa.SecurityPanelController interface uses the armState property to represent the state of the system.
    #[serde(rename = "waterAlarm")]
    WaterAlarm { value: AlarmPropertyValueTagged },

    /// Indicates the brightness of an appliance such as a smart light bulb. Valid value is an integer from 0-100, inclusive.
    /// If the brightness property is used to represent a change, then valid values are from -100 to 100 inclusive.
    #[serde(rename = "brightness")]
    Brightness { value: i64 },

    /// Represents a channel as a number or call sign. It is a structure containing number, CallSign and affiliateCallSign attributes.
    /// Only one of these attributes is required for the channel to be valid.
    /// Used by the ChannelController interface.
    #[serde(rename = "channel")]
    Channel { value: ChannelPropertyValue },

    /// Used for properties that represent the color of an endpoint.
    /// It is a structure containing hue, saturation and brightness fields, which are each double precision numbers.
    /// For use by endpoints that are able to change color.
    #[serde(rename = "color")]
    Color { value: ColorPropertyValue },

    /// The color temperature in Kelvin degrees.
    #[serde(rename = "colorTemperatureInKelvin")]
    ColorTemperatureInKelvin { value: u64 },

    /// Represents the connectivity state of an endpoint. Used by the EndpointHealth interface.
    #[serde(rename = "connectivity")]
    Connectivity { value: ConnectivityPropertyValue },

    Calendar,

    /// Represents a specific date and time in Coordinated Universal Time (UTC).
    /// The DateTime is a string that uses the RFC 3339 profile of the ISO 8601 format, YYYY-MM-DDThh:mm:ssZ.
    /// DateTime strings are always specified in UTC with no offsets.
    #[serde(rename = "cookCompletionTime")]
    CookCompletionTime { value: DateTime<Utc> },

    /// Represents the detection state of a sensor. Used by the MotionSensor and ContactSensor capabilities.
    #[serde(rename = "detectionState")]
    DetectionState { value: DetectionStatePropertyValue },

    /// Represents the detection state of a sensor. Used by the EventDetectionSensor.
    #[serde(rename = "humanPresenceDetectionState")]
    HumanPresenceDetectionState { value: event_detection_sensor::HumanPresenceDetectionStateValue },

    /// Represents a time period. The time period is represented in ISO 8601 duration format.
    /// For cooking operations, you should limit the duration value to the time portion only.
    /// This can be a positive or negative value.
    ///
    /// The format is PT*H*H*M*M*SS*S, where:
    ///
    /// P: Required and indicates a duration.
    /// T: Required and indicates a time.
    /// H: Indicates hour and is preceded by the number of hours, if hours are specified.
    /// M: Indicates minutes, and is preceded by the number of minutes, if minutes are specified.
    /// S: Indicates seconds, preceded by the number of seconds, if seconds are specified.
    ///
    /// Example 3 minute and 15 second duration: PT3M15S
    /// Example delta of -30 seconds: PT-30S
    #[serde(rename = "cookDuration")]
    DurationCookDuration { value: std::time::Duration },

    /// Indicates the brightness of an appliance such as a smart light bulb. Valid value is an integer from 0-100, inclusive.
    #[serde(rename = "enablementMode")]
    EnablementMode { value: EnablementModePropertyValue },

    /// Provides a cooking power level from a list of values. A descendant of the polymorphic PowerLevel.
    #[serde(rename = "powerLevel")]
    EnumeratedPowerLevel {
        value: EnumeratedPowerLevelPropertyValue,
        #[serde(rename = "@type")]
        json_type: String,
    },

    /// Represents equalizer band configurations. EqualizerBands provides a list of bands and their values.
    /// Each list element has two fields, name and value.
    #[serde(rename = "EqualizerBands")]
    EqualizerBands {
        value: Vec<EqualizerBandsPropertyValue>,
    },

    /// Represents an equalizer mode. The following modes are supported: MOVIE, MUSIC, NIGHT, SPORT, or TV.
    #[serde(rename = "mode")]
    EqualizerMode { value: String },

    /// Indicates the brightness of an appliance such as a smart light bulb. Valid value is an integer from 0-100, inclusive.
    #[serde(rename = "featureAvailability")]
    FeatureAvailability {
        value: FeatureAvailabilityPropertyValue,
    },

    /// The Alexa.LockController interface uses the lockState property to represent the state of a lock.
    #[serde(rename = "lockState")]
    LockState { value: LockStatePropertyValue },

    /// Represents the input state of an audio or video endpoint. Its value is a string that indicates the input device. Used by the InputController interface.
    /// The following are the valid values for input name:
    ///
    /// AUX 1, AUX 2, AUX 3, AUX 4, AUX 5, AUX 6, AUX 7, BLURAY, CABLE, CD, COAX 1, COAX 2, COMPOSITE 1, DVD, GAME, HD RADIO, HDMI 1, HDMI 2, HDMI 3, HDMI 4, HDMI 5, HDMI 6, HDMI 7, HDMI 8, HDMI 9, HDMI 10, HDMI ARC, INPUT 1, INPUT 2, INPUT 3, INPUT 4, INPUT 5, INPUT 6, INPUT 7, INPUT 8, INPUT 9, INPUT 10, IPOD, LINE 1, LINE 2, LINE 3, LINE 4, LINE 5, LINE 6, LINE 7, MEDIA PLAYER, OPTICAL 1, OPTICAL 2, PHONO, PLAYSTATION, PLAYSTATION 3, PLAYSTATION 4, SATELLITE, SMARTCAST, TUNER, TV, USB DAC, VIDEO 1, VIDEO 2, VIDEO 3, XBOX
    #[serde(rename = "input")]
    Input { value: String },

    /// Provides a power level represented as an integer on a number scale.
    #[serde(rename = "powerLevel")]
    PowerLevel {
        value: u64,
    },

    /// The Alexa.RangeController interface uses the rangeValue property as the primary property.
    /// The property values are numbers, and you specify the minimum value, maximum value, and precision in your discover response.
    /// If the rangeValue property is used to represent a change, then valid values include negative numbers.
    #[serde(rename = "rangeValue")]
    RangeValue {
        value: u64,
    },

    #[serde(rename = "mode")]
    Mode { value: String },

    /// Represents the mute state of an audio device. A single boolean value where true indicates the device is muted and false indicates the device is not muted. Used by the Speaker interface.
    #[serde(rename = "muted")]
    MuteState { value: bool },

    /// Indicates the brightness of an appliance such as a smart light bulb. Valid value is an integer from 0-100, inclusive.
    #[serde(rename = "name")]
    Name {
        #[serde(rename = "firstName")]
        first_name: String,
        #[serde(rename = "lastName")]
        last_name: String,
        #[serde(rename = "nickNames")]
        nick_names: Vec<String>,
    },

    /// Represent a percentage value. Integer with a valid range of 0-100.
    #[serde(rename = "percentage")]
    Percentage { value: u64 },

    /// Use playbackState to indicate the state of an endpoint that plays media.
    #[serde(rename = "playbackState")]
    PlaybackState { value: PlaybackStatePropertyValue },

    /// Use powerState to indicate whether the power to a device is on or off. The valid values are ON or OFF.
    #[serde(rename = "powerState")]
    PowerState { value: PowerStatePropertyValue },

    /// Use the quantity object to represent an amount of liquid.
    /// You can use the following values for quantity unit. The values are strings:
    /// * MILLILITER (1/1000 of a liter)
    /// * US_FLUID_OUNCE (1/128 of a gallon)
    #[serde(rename = "quantity")]
    Quantity { value: String, unit: String },

    /// Represent a percentage value. Integer with a valid range of 0-100.
    #[serde(rename = "recordingState")]
    RecordingState { value: RecordingStatePropertyValue },

    /// Represent a percentage value. Integer with a valid range of 0-100.
    #[serde(rename = "lowerSetpoint")]
    TemperatureLowerSetpoint { value: TemperaturePropertyValue },

    /// ThermostatMode represents the heating and cooling modes for a thermostat.
    #[serde(rename = "percentage")]
    ThermostatMode { value: ThermostatModes },

    /// Represent a percentage value. Integer with a valid range of 0-100.
    #[serde(rename = "TimeInterval")]
    TimeInterval { value: TimeInterval },

    /// Indicates a quantity as a unit of volume in one of several different standard units. Can be a descendant type of the polymorphic FoodQuantity.
    #[serde(rename = "volume")]
    Volume { value: f64, unit: VolumeUnit },

    /// Used for properties that represent the audio volume level on a scale from 0 to 100.
    /// Its value is a single integer ranging from 0-100. Used by the Speaker interface.
    #[serde(rename = "volume")]
    VolumeLevel { value: u64 },

    /// Indicates a quantity as a unit of weight or mass in one of several different standard units.
    /// Can be a descendant type of the polymorphic FoodQuantity.
    #[serde(rename = "weight")]
    Weight { value: WeightPropertyValue },
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize)]
pub struct PropertyCalenderPayload {
    /// The name of the meeting organizer.
    #[serde(rename = "organizerName")]
    pub organizer_name: String,
    /// A meeting identifier that resolves in the conferencing system.
    #[serde(rename = "calendarEventId")]
    pub calendar_event_id: String,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize)]
pub struct Property {
    pub namespace: &'static str,
    /// Newer controllers like the ToggleController have an instance ID.
    pub instance: Option<String>,
    #[serde(flatten)]
    pub name: PropertyName,
    /// Only used for the calendar property.
    /// See https://developer.amazon.com/de/docs/device-apis/alexa-calendar.html#response
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub calendar_payload: Option<PropertyCalenderPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "timeOfSample")]
    pub time_of_sample: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "uncertaintyInMilliseconds")]
    pub uncertainty_in_milliseconds: Option<i64>,
}

impl Property {
    pub fn new(namespace: &'static str, name: PropertyName) -> Self {
        Property {
            namespace,
            instance: None,
            name,
            calendar_payload: None,
            time_of_sample: None,
            uncertainty_in_milliseconds: None,
        }
    }
    /// Only used for the calendar controller which diverges from all other controllers.
    /// https://developer.amazon.com/de/docs/device-apis/alexa-calendar.html#response
    pub fn calendar(namespace: &'static str, calendar: PropertyCalenderPayload) -> Self {
        Property {
            namespace,
            instance: None,
            name: PropertyName::Calendar,
            calendar_payload: Some(calendar),
            time_of_sample: None,
            uncertainty_in_milliseconds: None,
        }
    }
    pub fn with_instance(namespace: &'static str, name: PropertyName, instance: String) -> Self {
        Property {
            namespace,
            instance: Some(instance),
            name,
            calendar_payload: None,
            time_of_sample: None,
            uncertainty_in_milliseconds: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    #[test]
    fn property_deserialize_brightness() {
        let str = serde_json::to_string(&Property {
            namespace: "",
            instance: None,
            name: PropertyName::Brightness { value: 12 },
            calendar_payload: None,
            time_of_sample: None,
            uncertainty_in_milliseconds: None,
        })
            .unwrap();
        assert_eq!(
            str,
            "{\"namespace\":\"\",\"name\":\"brightness\",\"value\":12}"
        );
    }

    #[test]
    fn property_deserialize_channel() {
        let str = serde_json::to_string(&Property {
            namespace: "",
            instance: None,
            name: PropertyName::Channel {
                value: ChannelPropertyValue::AffiliateCallSign("abc".to_string()),
            },
            calendar_payload: None,
            time_of_sample: None,
            uncertainty_in_milliseconds: None,
        })
            .unwrap();
        assert_eq!(
            str,
            "{\"namespace\":\"\",\"name\":\"channel\",\"value\":{\"affiliateCallSign\":\"abc\"}}"
        );
    }

    #[test]
    fn property_deserialize_time() {
        let str = serde_json::to_string(&Property {
            namespace: "",
            instance: None,
            name: PropertyName::TimeInterval {
                value: TimeInterval {
                    start: Some(DateTime::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc)),
                    end: Some(DateTime::from_utc(
                        NaiveDateTime::from_timestamp(10, 0),
                        Utc,
                    )),
                    duration: Some(std::time::Duration::new(10, 0).into()),
                },
            },
            calendar_payload: None,
            time_of_sample: None,
            uncertainty_in_milliseconds: None,
        })
            .unwrap();
        assert_eq!(
            str,
            "{\"namespace\":\"\",\"name\":\"TimeInterval\",\"value\":{\"start\":\"1970-01-01T00:00:00Z\",\"end\":\"1970-01-01T00:00:10Z\",\"duration\":\"PT10S\"}}"
        );
    }
}
