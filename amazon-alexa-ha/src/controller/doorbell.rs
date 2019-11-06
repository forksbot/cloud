//! The Alexa.DoorbellEventSource interface describes an endpoint that is capable of raising doorbell events.
//! Alexa plays doorbell announcements on all Echo devices.
//! Alexa uses doorbell events for announcements, mobile notifications, routines, and other use cases.
//! https://developer.amazon.com/de/docs/device-apis/alexa-doorbelleventsource.html

use crate::{
    display_categories::DisplayCategory,
    directive::Command,
    event::{
        discovery::{DeviceEndpoint, Capability, CapabilityAdditional},
        properties::{Property, PropertyName}, self,
    },
    {Header, Endpoint},
    event::properties::PropertyCalenderPayload,
};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::event::change_report::ChangeReportCause;

const INTERFACE_NAME: &'static str = "Alexa.DoorbellEventSource";
const SUPPORTED_PROPERTIES: &'static [&'static str] = &[];

/// Add Alexa.DoorbellEventSource capability to a device endpoint for discovery responses
pub fn add_to_endpoint(endpoint: &mut DeviceEndpoint, proactively_reported: bool) {
    endpoint.capabilities.push(Capability::new(CapabilityAdditional::DoorbellEventSource { proactively_reported }));
}

/// Add Alexa.DoorbellEventSource property to response or report
#[inline]
pub fn add_to_response_context(properties: &mut Vec<Property>, organizer_name: String, calendar_event_id: String) {
    properties.push(Property::calendar(INTERFACE_NAME, PropertyCalenderPayload {
        organizer_name,
        calendar_event_id,
    }));
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct DoorbellPressReportPayload {
    /// Describes why this event occurred.
    pub cause: ChangeReportCause,
    /// When the activation event occurred, specified in UTC.
    pub timestamp: DateTime<Utc>,
}

/// If the InitializeCameraStreams directive was successfully handled, you should respond with an Response event.
/// The payload for this message contains the camera streams for the specified endpoint.
pub type DoorbellPressReport = event::Response<event::EmptyPayload, event::Event<DoorbellPressReportPayload>>;

impl DoorbellPressReport {
    pub fn new(cause: ChangeReportCause, endpoint: Endpoint) -> Self {
        let header = Header::new(INTERFACE_NAME, "Response");
        event::Response::new(event::Event::new(header, endpoint, DoorbellPressReportPayload { cause, timestamp: Utc::now() }))
    }
}