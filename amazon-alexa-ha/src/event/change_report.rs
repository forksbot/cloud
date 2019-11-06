//! # Proactive state reporting. Includes endpoint health and properties.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    Header,
    ErrorType,
    Event,
    EventWithoutEndpoint,
    Endpoint,
    properties::{Properties, Property, PropertyName},
};

use crate::{
    property_types::ConnectivityPropertyValue,
};

/// ChangeReport events are sent to notify Alexa of state changes.
/// You specify a property as proactivelyReported during discovery,
/// and then you send Alexa a ChangeReport event whenever that property value changes, regardless of why the property changed.
///
/// For example, if the "Kitchen Light" endpoint turns on, you would notify Alexa by sending a ChangeReport event that indicates the powerState property of the
/// Alexa.PowerController interface has changed its value to "ON".
///
/// * Use the payload of the ChangeReport to provide the new property value and the reason for the change.
/// * You use the context of a ChangeReport to report the state of any additional properties.
/// * If multiple properties have changed, you can send Alexa multiple change report events containing a payload with a single property, or a single change report event that contains a payload with multiple property values.
/// * Make sure to identify the customer and the endpoint for the change report in the endpoint object.
pub type ChangeReport = super::Response<Properties, Event<ChangeReportPayload>>;

/// If your endpoint supports a property as retrievable, then you should report its value when you receive a ReportState directive from Alexa.
///
/// If the endpoint is currently unreachable but you can report all endpoint property values because they are cached,
/// then return the StateReport and include all of the property values.
///
/// However, specify the value of the connectivity property of EndpointHealth as UNREACHABLE.
/// If you cannot report the state of all the properties because the endpoint is unreachable and you have not cached the values,
/// you should send an ErrorResponse of type BRIDGE_UNREACHABLE or ENDPOINT_UNREACHABLE.
pub type StateReport = super::Response<Properties, Event<super::EmptyPayload>>;

#[allow(non_camel_case_types)]
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize)]
pub struct ChangeReportPayload {
    change: ChangeReportPayloadInner
}

#[allow(non_camel_case_types)]
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize)]
pub struct ChangeReportPayloadInner {
    cause: ChangeReportCause,
    properties: Properties,
}

/// The cause attribute is used to describe the cause of a property value change when you send a ChangeReport event.
/// https://developer.amazon.com/de/docs/smarthome/state-reporting-for-a-smart-home-skill.html#cause-object
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ChangeReportCause {
    ///	Indicates that the event was caused by a customer interaction with an application.
    /// For example, a customer switches on a light or locks a door using the Alexa app or an app provided by a device vendor.
    APP_INTERACTION,
    ///	Indicates that the event was caused by a physical interaction with an endpoint.
    /// For example, manually switching on a light or manually locking a door lock.
    PHYSICAL_INTERACTION,
    ///	Indicates that the event was caused by the periodic poll of an endpoint, which found a change in value.
    /// For example, you might poll a temperature sensor every hour and send the updated temperature to Alexa.
    PERIODIC_POLL,
    ///	Indicates that the event was caused by the application of a device rule.
    /// For example, a customer configures a rule to switch on a light if a motion sensor detects motion.
    /// In this case, Alexa receives an event from the motion sensor, and another event from the light to indicate that its state change was caused by the rule.
    RULE_TRIGGER,
    ///	Indicates that the event was caused by a voice interaction.
    /// For example, a user speaking to their Echo device.
    VOICE_INTERACTION,
}

impl ChangeReport {
    pub fn new(cause: ChangeReportCause, changed_properties: Properties, context_properties: Option<Properties>, endpoint: Endpoint) -> Self {
        super::Response {
            context: context_properties,
            event: Event {
                header: Header::new("Alexa", "ChangeReport"),
                endpoint,
                payload: ChangeReportPayload { change: ChangeReportPayloadInner { cause, properties: changed_properties } },
            },
        }
    }
}

impl StateReport {
    /// If the endpoint is currently unreachable but you can report all endpoint property values because they are cached,
    /// then return the StateReport and include all of the property values.
    pub fn unreachable_with_cached_props(mut properties: Vec<Property>, endpoint: Endpoint) -> Self {
        properties.push(Property {
            namespace: "Alexa.EndpointHealth",
            instance: None,
            name: PropertyName::Connectivity { value: ConnectivityPropertyValue::UNREACHABLE },
            calendar_payload: None,
            time_of_sample: None,
            uncertainty_in_milliseconds: None,
        });
        StateReport {
            context: Some(Properties { properties }),
            event: Event {
                header: Header::new("Alexa", "StateReport"),
                endpoint,
                payload: super::EmptyPayload {},
            },
        }
    }
    /// If your endpoint supports a property as retrievable, then you should report its value when you receive a ReportState directive from Alexa.
    pub fn reachable(mut properties: Vec<Property>, endpoint: Endpoint) -> Self {
        properties.push(Property {
            namespace: "Alexa.EndpointHealth",
            instance: None,
            name: PropertyName::Connectivity { value: ConnectivityPropertyValue::OK },
            calendar_payload: None,
            time_of_sample: None,
            uncertainty_in_milliseconds: None,
        });
        StateReport {
            context: Some(Properties { properties }),
            event: Event {
                header: Header::new("Alexa", "StateReport"),
                endpoint,
                payload: super::EmptyPayload {},
            },
        }
    }
}
