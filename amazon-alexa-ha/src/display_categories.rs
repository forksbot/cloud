use serde::{Deserialize, Serialize};

/// When you provide the display category in your discovery response, your endpoint appears in the correct category in the Alexa app, with the correct iconography. This makes it easier for users to find and monitor your devices.
/// https://developer.amazon.com/de/docs/device-apis/alexa-discovery.html#display-categories
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub enum DisplayCategory {
    ///	A combination of devices set to a specific state. Use activity triggers for scenes when the state changes must occur in a specific order. For example, for a scene named "watch Netflix" you might power on the TV first, and then set the input to HDMI1.
    ACTIVITY_TRIGGER,
    ///	A media device with video or photo functionality.
    CAMERA,
    ///	An endpoint that detects and reports changes in contact between two surfaces.
    CONTACT_SENSOR,
    ///	A door.
    DOOR,
    ///	A doorbell.
    DOORBELL,
    ///	A fan.
    FAN,
    ///	A light source or fixture.
    LIGHT,
    ///	A microwave oven.
    MICROWAVE,
    ///	An endpoint that detects and reports movement in an area.
    MOTION_SENSOR,
    ///	An endpoint that can't be described by one of the other categories.
    OTHER,
    ///	A combination of devices set to a specific state. Use scene triggers for scenes when the order of the state change is not important. For example, for a scene named "bedtime" you might turn off the lights and lower the thermostat, in any order.
    SCENE_TRIGGER,
    ///	A security panel.
    SECURITY_PANEL,
    ///	An endpoint that locks.
    SMARTLOCK,
    ///	A module that is plugged into an existing electrical outlet, and then has a device plugged into it. For example, a user can plug a smart plug into an outlet, and then plug a lamp into the smart plug. A smart plug can control a variety of devices.
    SMARTPLUG,
    ///	A speaker or speaker system.
    SPEAKER,
    ///	A switch wired directly to the electrical system. A switch can control a variety of devices.
    SWITCH,
    ///	An endpoint that reports temperature, but does not control it. The temperature data of the endpoint is not shown in the Alexa app.
    TEMPERATURE_SENSOR,
    ///	An endpoint that controls temperature, stand-alone air conditioners, or heaters with direct temperature control.
    THERMOSTAT,
    ///	A television.
    TV,
}
