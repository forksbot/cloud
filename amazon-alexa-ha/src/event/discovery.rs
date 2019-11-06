use serde::{Deserialize, Serialize, Serializer};
use uuid::Uuid;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use super::{
    Header,
    ErrorType,
    Event,
    EventWithoutEndpoint,
    Endpoint,
};

use crate::{
    controller::{
        camera_stream::CameraConfiguration,
        equalizer::EqualizerConfiguration,
        event_detection_sensor::{DetectionModes, EventDetectionSensor},
        mode::ModeConfiguration,
        range::RangeConfiguration,
        rtc_session::RTCConfiguration,
        security_panel::SecurityPanelConfiguration
    },
    CapabilityType,
    CapabilityVersion,
    display_categories::DisplayCategory,
    utils_serde::ArrayOfStaticStrings,
    property_types::CapabilityResource,
};

/// There are no reportable properties currently defined for this interface.
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize)]
pub struct AddOrUpdateReportPayload {
    /// An object containing a bearer token to identify the customer to Alexa.
    pub scope: super::super::Scope,
    /// The devices associated with the user's account, and the capabilities that your skill supports for them.
    /// If there are no devices associated with the customer account, return an empty array for this property.
    /// The maximum number of endpoints you can return is 300.
    pub endpoints: Vec<DeviceEndpoint>,
}

/// There are no reportable properties currently defined for this interface.
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize)]
pub struct DeleteReportPayload {
    /// An object containing a bearer token to identify the customer to Alexa.
    pub scope: super::super::Scope,
    /// The endpoints to delete from the customer account.
    pub endpoints: Vec<DeviceEndpointID>,
}

/// Just the ID of an endpoint
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize)]
pub struct DeviceEndpointID {
    #[serde(rename = "endpointId")]
    pub endpoint_id: String,
}


#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize)]
pub struct DeviceEndpoints {
    /// The devices associated with the user's account, and the capabilities that your skill supports for them.
    /// If there are no devices associated with the customer account, return an empty array for this property.
    /// The maximum number of endpoints you can return is 300.
    endpoints: Vec<DeviceEndpoint>
}

/// The endpoint object represents a connected device or component associated with a customer's device cloud account.
/// An endpoint describes one of the following:
///
/// * A physical device
/// * A virtual device
/// * A group or cluster of devices
/// * A software component
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize)]
pub struct DeviceEndpoint {
    /// The identifier for the endpoint. The identifier must be unique across all devices for the skill.
    /// In addition, the identifier must be consistent for all discovery requests for the same device.
    /// An identifier can contain letters or numbers, spaces, and the following special characters: _ - = # ; : ? @ &.
    /// The identifier cannot exceed 256 characters.
    #[serde(rename = "endpointId")]
    pub endpoint_id: String,
    /// The name of the manufacturer of the device. This value can contain up to 128 characters.
    #[serde(rename = "manufacturerName")]
    pub manufacturer_name: String,
    /// The description of the device. The description should contain the manufacturer name or how the device is connected.
    /// For example, "Smart Lock by Sample Manufacturer" or "WiFi Thermostat connected via SmartHub".
    /// This value can contain up to 128 characters.
    pub description: String,
    /// The name used by the customer to identify the device.
    /// This value can contain up to 128 characters, and should not contain special characters or punctuation.
    #[serde(rename = "friendlyName")]
    pub friendly_name: String,
    /// Additional information about the endpoint.
    #[serde(rename = "additionalAttributes")]
    pub additional_attributes: AdditionalAttributes,
    /// In the Alexa app, the category that your device is displayed in.
    #[serde(rename = "displayCategories")]
    pub display_categories: Vec<DisplayCategory>,
    /// The capability interfaces that your skill supports for the endpoint, such as Alexa.BrightnessController or Alexa.PowerController.
    pub capabilities: Vec<Capability>,
    /// Information about the methods that the device uses to connect to the internet and smart home hubs.
    pub connections: Vec<Connection>,
    /// Information about the device that your skill uses. The contents of this property cannot exceed 5000 bytes. The API doesn't read or use this data.
    pub cookie: HashMap<String, String>,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize)]
pub struct AdditionalAttributes {
    /// The name of the manufacturer of the device. This value can contain up to 256 alphanumeric characters, and can contain punctuation.
    pub manufacturer: String,
    /// The name of the model of the device. This value can contain up to 256 alphanumeric characters, and can contain punctuation.
    pub model: String,
    /// The serial number of the device. This value can contain up to 256 alphanumeric characters, and can contain punctuation.
    #[serde(rename = "serialNumber")]
    pub serial_number: String,
    /// The firmware version of the device. This value can contain up to 256 alphanumeric characters, and can contain punctuation.
    #[serde(rename = "firmwareVersion")]
    pub firmware_version: String,
    /// The software version of the device. This value can contain up to 256 alphanumeric characters, and can contain punctuation.
    #[serde(rename = "softwareVersion")]
    pub software_version: String,
    /// Your custom identifier for the device. This identifier should be globally unique in your systems across different customer accounts.
    /// This value can contain up to 256 alphanumeric characters, and can contain punctuation.
    #[serde(rename = "customIdentifier")]
    pub custom_identifier: String,
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct Capability {
    /// The type of capability. Currently, the only available type is AlexaInterface.
    #[serde(rename = "type")]
    pub capability_type: CapabilityType,
    /// The name of the capability interface.
    pub interface: CapabilityAdditional,
    /// The version of the interface. Different interfaces have different versions from each other.
    /// Always check the documentation for an interface to verify the current version.
    pub version: CapabilityVersion,
}

/// Encodes the name of the capability interface via the "interface" tag
/// and controller dependant additional information.
#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "interface")]
pub enum CapabilityAdditional {
    #[serde(rename = "Alexa.BrightnessController")]
    BrightnessController { properties: CapabilityProperties },
    #[serde(rename = "Alexa.ChannelController")]
    ChannelController { properties: CapabilityProperties },
    #[serde(rename = "Alexa.Calendar")]
    Calendar { properties: CapabilityProperties },
    #[serde(rename = "Alexa.CameraStreamController")]
    CameraStreamController {
        #[serde(rename = "cameraStreamConfigurations")]
        camera_config: Vec<CameraConfiguration>,
    },
    #[serde(rename = "Alexa.ColorController")]
    ColorController { properties: CapabilityProperties },
    #[serde(rename = "Alexa.ColorTemperatureController")]
    ColorTemperatureController { properties: CapabilityProperties },
    #[serde(rename = "Alexa.ContactSensor")]
    ContactSensor { properties: CapabilityProperties },
    #[serde(rename = "Alexa.DoorbellEventSource")]
    DoorbellEventSource {
        /// True if your skill sends change reports when the properties change. The default is false.
        #[serde(rename = "proactivelyReported")]
        proactively_reported: bool,
    },
    #[serde(rename = "Alexa.EqualizerController")]
    EqualizerController { configurations: EqualizerConfiguration },
    #[serde(rename = "Alexa.EventDetectionSensor")]
    EventDetectionSensor {
        properties: CapabilityProperties,
        configuration: EventDetectionSensor,
    },
    #[serde(rename = "Alexa.InputController")]
    InputController { inputs: ArrayOfStaticStrings },
    #[serde(rename = "Alexa.LockController")]
    LockController { properties: CapabilityProperties },
    #[serde(rename = "Alexa.MediaMetadata")]
    MediaMetadata {
        /// True if your skill sends change reports when the properties change. The default is false.
        #[serde(rename = "proactivelyReported")]
        proactively_reported: bool,
    },
    #[serde(rename = "Alexa.MeetingClientController")]
    MeetingClientController {
        /// True if the device supports the JoinScheduledMeeting directive; otherwise, false.
        #[serde(rename = "supportsScheduledMeeting")]
        supports_scheduled_meeting: bool,
    },
    #[serde(rename = "Alexa.ModeController")]
    ModeController {
        properties: CapabilityProperties,
        #[serde(rename = "capabilityResources")]
        capability_resources: CapabilityResource,
        configuration: ModeConfiguration,
    },
    #[serde(rename = "Alexa.PercentageController")]
    PercentageController { properties: CapabilityProperties },
    #[serde(rename = "Alexa.PowerController")]
    PowerController { properties: CapabilityProperties },
    #[serde(rename = "Alexa.PowerLevelController")]
    PowerLevelController { properties: CapabilityProperties },
    #[serde(rename = "Alexa.RangeController")]
    RangeController {
        properties: CapabilityProperties,
        #[serde(rename = "capabilityResources")]
        capability_resources: CapabilityResource,
        configuration: RangeConfiguration,
    },
    #[serde(rename = "Alexa.RTCSessionController")]
    RTCSessionController { configuration: RTCConfiguration },
    #[serde(rename = "Alexa.SecurityPanelController")]
    SecurityPanelController {
        properties: CapabilityProperties,
        configuration: SecurityPanelConfiguration,
    },
}

impl Capability {
    pub fn new(interface: CapabilityAdditional) -> Self {
        Capability {
            capability_type: CapabilityType::AlexaInterface,
            interface,
            version: CapabilityVersion::THREE,
        }
    }
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct CapabilityProperties {
    /// The properties of the interface which are supported by your skill.
    pub supported: ArrayOfStaticStrings,
    /// True if your skill sends change reports when the properties change. The default is false.
    #[serde(rename = "proactivelyReported")]
    pub proactively_reported: bool,
    /// True if you respond to state report requests and report the values of the properties. The default is false.
    pub retrievable: bool,
    /// You can model properties of an endpoint that customers can't change by setting nonControllable to true.
    #[serde(rename = "nonControllable")]
    pub non_controllable: bool,
}

/// The connections property represents the methods that a device uses to connect to the internet and smart home hubs.
/// https://developer.amazon.com/de/docs/device-apis/alexa-discovery.html#connections-object
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Connection {
    ///	macAddress	The unique identifier for the network interface controller (NIC).
    TCP_IP {
        #[serde(rename = "macAddress")]
        mac_address: Option<String>
    },
    ///	macAddress	The unique identifier for the network interface controller (NIC).
    ZIGBEE {
        #[serde(rename = "macAddress")]
        mac_address: Option<String>
    },
    ///	homeId  The Home ID for a Z-Wave network that the endpoint connects to. The format is 0x00000000 with UTF-8 characters.
    /// nodeId	The Node ID for the endpoint in a Z-Wave network that the endpoint connects to. The format is 0x00 with UTF-8 characters.
    ZWAVE {
        #[serde(rename = "homeId")]
        home_id: Option<String>,
        #[serde(rename = "nodeId")]
        node_id: Option<String>,
    },
    ///	value	The connection information for a connection when you can't identify the type of the connection more specifically.
    /// The information that you provide in this field should be stable and specific.
    /// This value can contain up to 256 alphanumeric characters, and can contain punctuation.
    UNKNOWN { value: String },
}

/// If you handle a Discover directive successfully, respond with an Discover.Response event.
/// In your response, return all of the devices associated with the end-user's device cloud account,
/// and the capabilities that your skill supports for them.
pub type DiscoverResponse = super::Response<super::EmptyPayload, EventWithoutEndpoint<DeviceEndpoints>>;

impl DiscoverResponse {
    /// Create a new discovery response with the given endpoints.
    pub fn new(endpoints: Vec<DeviceEndpoint>) -> Self {
        let header = Header::new("Alexa.Discovery", "Discover.Response");
        super::Response::new(EventWithoutEndpoint::new(header, DeviceEndpoints { endpoints }))
    }
}

/// You send an AddOrUpdateReport event proactively when a customer adds a new endpoint to their account,
/// or makes changes to an existing endpoint, such as renaming a scene. Send your AddOrUpdateReport message to the Alexa event gateway.
/// You can include all the endpoints associated with the customer account, or only the new or updated endpoints.
/// You can choose based on your skill implementation.
pub type AddOrUpdateReport = super::Response<super::EmptyPayload, Event<AddOrUpdateReportPayload>>;

impl AddOrUpdateReport {
    pub fn new(payload: AddOrUpdateReportPayload, endpoint: Endpoint) -> Self {
        let header = Header::new("Alexa.Discovery", "AddOrUpdateReport");
        super::Response::new(Event::new(header, endpoint, payload))
    }
}

/// You send a DeleteReport event proactively when a customer removes an endpoint from their account.
pub type DeleteReport = super::Response<super::EmptyPayload, Event<DeleteReportPayload>>;

impl DeleteReport {
    pub fn new(payload: DeleteReportPayload, endpoint: Endpoint) -> Self {
        let header = Header::new("Alexa.Discovery", "DeleteReport");
        super::Response::new(Event::new(header, endpoint, payload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_detection() {
        let input = r#"{
                        "type": "AlexaInterface",
                        "interface": "Alexa.EventDetectionSensor",
                        "version": "3",
                        "properties": {
                            "supported": [
                                {
                                    "name": "humanPresenceDetectionState"
                                }
                            ],
                            "proactivelyReported": true,
                            "retrievable": false
                        },
                        "configuration": {
                          "detectionMethods": ["AUDIO", "VIDEO"],
                          "detectionModes": {
                            "humanPresence": {
                              "featureAvailability": "ENABLED",
                              "supportsNotDetected": false
                             }
                          }
                        }
                      }"#;
        let output: Capability = serde_json::from_str(&input).unwrap();
        match output.interface {
            CapabilityAdditional::EventDetectionSensor { properties, configuration } => {
                assert_eq!(configuration.detection_methods.get(0).unwrap(), "AUDIO");
                assert_eq!(configuration.detection_modes.human_presence.supports_not_detected, false);
                assert_eq!(configuration.detection_modes.human_presence.feature_availability, "ENABLED");
            }
            _ => {
                panic!("")
            }
        }
    }
}
