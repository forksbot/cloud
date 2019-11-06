//! The Alexa.Calendar interface describes the messages used to find meetings on an organizational calendar.
//! For more information, see Build Skills for Conferencing Devices.
//! https://developer.amazon.com/de/docs/device-apis/alexa-calendar.html

use crate::{
    event::{
        discovery::{DeviceEndpoint, Capability, CapabilityAdditional, CapabilityProperties},
        properties::{Property, PropertyName, PropertyCalenderPayload},
    },
    directive::Command,
    utils_serde::ArrayOfStaticStrings
};
use serde::{Serialize, Serializer};
use chrono::{DateTime, Utc};

const INTERFACE_NAME: &'static str = "Alexa.Calendar";
const SUPPORTED_PROPERTIES: &'static [&'static str] = &[];

/// Add Alexa.Calendar capability to a device endpoint for discovery responses
pub fn add_to_endpoint(endpoint: &mut DeviceEndpoint, proactively_reported: bool) {
    endpoint.capabilities.push(Capability::new(CapabilityAdditional::Calendar {
        properties: CapabilityProperties {
            supported: ArrayOfStaticStrings(SUPPORTED_PROPERTIES),
            proactively_reported,
            retrievable: true,
            non_controllable: false
        }
    }));
}

/// Add Alexa.Calendar property to response or report
#[inline]
pub fn add_to_response_context(properties: &mut Vec<Property>, organizer_name: String, calendar_event_id: String) {
    properties.push(Property::calendar(INTERFACE_NAME, PropertyCalenderPayload {
        organizer_name,
        calendar_event_id,
    }));
}
