//! The Alexa.CameraStreamController interface describes the messages used retrieve camera streams from camera endpoints.
//! https://developer.amazon.com/de/docs/device-apis/alexa-camerastreamcontroller.html

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::{
    display_categories::DisplayCategory,
    directive::Command,
    event::{
        discovery::{DeviceEndpoint, Capability, CapabilityAdditional},
        properties::{Property, PropertyName}, self,
    },
    {Header, Endpoint},
    directive::Directive
};

const INTERFACE_NAME: &'static str = "Alexa.CameraStreamController";

/// Discovery response camera configuration
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct CameraConfiguration {
    /// Protocols for the stream such as RTSP
    pub protocols: Vec<String>,
    /// An array of resolution objects, which describe the resolutions of the stream. Each resolution contains a width and height property.
    pub resolutions: Vec<CameraResolution>,
    /// Describes the authorization type. Possible values are "BASIC", "DIGEST" or "NONE"
    #[serde(rename = "authorizationTypes")]
    pub authorization_types: Vec<String>,
    /// The video codec for the stream. Possible values are "H264", "MPEG2", "MJPEG", or "JPG".
    #[serde(rename = "videoCodecs")]
    pub video_codecs: Vec<String>,
    /// The audio codec for the stream. Possible values are "G711", "AAC", or "NONE".
    #[serde(rename = "audioCodecs")]
    pub audio_codecs: Vec<String>,
}

/// Discovery response camera configuration
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct CameraStream {
    /// Protocol for the stream such as RTSP
    pub protocols: String,
    /// An array of resolution objects, which describe the resolutions of the stream. Each resolution contains a width and height property.
    pub resolutions: Vec<CameraResolution>,
    /// Describes the authorization type. Possible values are "BASIC", "DIGEST" or "NONE"
    #[serde(rename = "authorizationType")]
    pub authorization_type: String,
    /// The video codec for the stream. Possible values are "H264", "MPEG2", "MJPEG", or "JPG".
    #[serde(rename = "videoCodec")]
    pub video_codec: String,
    /// The audio codec for the stream. Possible values are "G711", "AAC", or "NONE".
    #[serde(rename = "audioCodec")]
    pub audio_codec: String,
    /// Indicates the timeout value for the stream.
    #[serde(rename = "idleTimeoutSeconds")]
    pub idle_timeout_seconds: u64,
    /// The time that the stream expires, specified in UTC.
    #[serde(rename = "expirationTime")]
    pub expiration_time: DateTime<Utc>,
    /// The URI for the camera stream. This may be a temporary URI that expires at the time specified by expirationTime.
    /// If the URI expires, and an error occurs, Alexa will make a new call to InitializeCameraStreams to get a new, unexpired URI.
    /// For example: rtsp://username:password@link.to.video:443/feed1.mp4
    pub uri: String,
}

/// Discovery response camera configuration
#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct CameraStreams {
    #[serde(rename = "cameraStreams")]
    pub camera_streams: Vec<CameraStream>
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct CameraResolution {
    pub width: i64,
    pub height: i64,
}


/// Add Alexa.CameraStreamController capability to a device endpoint for discovery responses.
/// Sets endpoint display category to CAMERA.
#[inline]
pub fn add_to_endpoint(endpoint: &mut DeviceEndpoint, camera_config: Vec<CameraConfiguration>) {
    endpoint.display_categories.push(DisplayCategory::CAMERA);
    endpoint.capabilities.push(Capability::new(CapabilityAdditional::CameraStreamController { camera_config }));
}

/// If the InitializeCameraStreams directive was successfully handled, you should respond with an Response event.
/// The payload for this message contains the camera streams for the specified endpoint.
pub type InitializeCameraStreamsResponse = event::Response<event::EmptyPayload, event::Event<CameraStreams>>;

impl InitializeCameraStreamsResponse {
    pub fn new(camera_streams: Vec<CameraStream>, endpoint: Endpoint) -> Self {
        let header = Header::new(INTERFACE_NAME, "Response");
        event::Response::new(event::Event::new(header, endpoint, CameraStreams { camera_streams }))
    }
}

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Deserialize)]
pub struct DirectiveInitializeCameraStreams {
    #[serde(rename = "cameraStreams")]
    pub camera_streams: Vec<CameraConfiguration>
}

impl Directive for DirectiveInitializeCameraStreams { const NAME: &'static str = "InitializeCameraStreams"; }
