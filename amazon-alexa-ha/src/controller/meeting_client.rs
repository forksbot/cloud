//! The Alexa.InputController interface describes messages for changing the input of an entertainment device.

use serde::{Serialize, Deserialize};

use crate::{
    property_types::ChannelPropertyValue,
    directive::{Command, Directive},
    event::{
        discovery::{DeviceEndpoint, Capability, CapabilityAdditional},
        properties::{Property, PropertyName},
    },
    utils_serde::ArrayOfStaticStrings,
};

const INTERFACE_NAME: &'static str = "Alexa.MeetingClientController";

/// Add Alexa.MeetingClientController capability to a device endpoint for discovery responses
#[inline]
pub fn add_to_endpoint(endpoint: &mut DeviceEndpoint, supports_scheduled_meeting: bool) {
    endpoint.capabilities.push(Capability::new(CapabilityAdditional::MeetingClientController { supports_scheduled_meeting }));
}

#[derive(Serialize, Deserialize)]
pub struct Meeting {
    /// The meeting provider, such as Amazon Chime or Skype for Business.
    pub provider: String,
    /// The protocol for the meeting, one of SIP, SIPS, H323.
    pub protocol: String,
    /// The meeting endpoint. Typically an IP address or URL.
    pub endpoint: String,
    /// The identifier for the meeting. This field is not always specified.
    pub id: Option<String>,
    /// The PIN for the meeting. This field is not always specified.
    pub pin: Option<String>,
}

/// Support the JoinMeeting directive so that users can join a meeting when they know the id of the meeting that they want to join.
/// You must support the JoinMeeting directive.
///
/// The JoinScheduledMeeting directive uses the BearerTokenWithPartition type of endpoint scope.
/// For more information, see BearerTokenWithPartition scope.
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveJoinMeeting {
    /// The name of the input. The name must be unique across inputs for this endpoint.
    pub meeting: Meeting
}

impl Directive for DirectiveJoinMeeting { const NAME: &'static str = "JoinMeeting"; }

/// Support the EndMeeting directive so that users can end a meeting.
/// You must support the EndMeeting directive.
/// When Alexa sends the EndMeeting directive, you should end all active meetings.
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveEndMeeting {}

impl Directive for DirectiveEndMeeting { const NAME: &'static str = "EndMeeting"; }
