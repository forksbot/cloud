//! # Error and Result Type

use std::error;
use std::fmt;

/// The main error type used throughout this crate. It wraps / converts from a few other error
/// types and implements [error::Error] so that you can use it in any situation where the
/// standard error type is expected.
#[derive(Debug)]
pub enum CloudAuthError {
    /// Generic errors are very rarely used and only used if no other error type matches
    Generic(&'static str),
    GenericOwned(String),
    /// An error returned by the Firestore API - Contains the numeric code, a string code and
    /// a context. If the APIError happened on a document query or mutation, the document
    /// path will be set as context.
    /// If the APIError happens on a user_* method, the user id will be set as context.
    /// For example: 400, CREDENTIAL_TOO_OLD_LOGIN_AGAIN
    APIError(usize, String, String),
    /// Should not happen. If jwt encoding / decoding fails or an value cannot be extracted or
    /// a jwt is badly formatted or corrupted
    JWT(biscuit::errors::Error),
    JWTValidation(biscuit::errors::ValidationError),
    /// Serialisation failed
    Ser {
        doc: Option<String>,
        ser: serde_json::Error,
    },
    /// When the credentials.json file contains an invalid private key this error is returned
    RSA(ring::error::KeyRejected),
    /// Disk access errors
    IO(std::io::Error),
    /// First the source url, then the response string
    HttpError(String,String),
    TooManyRequests,
    Timeout
}

impl std::convert::From<std::io::Error> for CloudAuthError {
    fn from(error: std::io::Error) -> Self {
        CloudAuthError::IO(error)
    }
}

impl std::convert::From<base64::DecodeError> for CloudAuthError {
    fn from(error: base64::DecodeError) -> Self {
        CloudAuthError::GenericOwned(error.to_string())
    }
}

impl std::convert::From<ring::error::Unspecified> for CloudAuthError {
    fn from(error: ring::error::Unspecified) -> Self {
        CloudAuthError::GenericOwned(error.to_string())
    }
}

impl std::convert::From<miniz_oxide::inflate::TINFLStatus> for CloudAuthError {
    fn from(_error: miniz_oxide::inflate::TINFLStatus) -> Self {
        CloudAuthError::Generic("Inflate error")
    }
}

impl std::convert::From<ring::error::KeyRejected> for CloudAuthError {
    fn from(error: ring::error::KeyRejected) -> Self {
        CloudAuthError::RSA(error)
    }
}

impl std::convert::From<serde_json::Error> for CloudAuthError {
    fn from(error: serde_json::Error) -> Self {
        CloudAuthError::Ser { doc: None, ser: error }
    }
}

impl std::convert::From<serde_urlencoded::ser::Error> for CloudAuthError {
    fn from(error: serde_urlencoded::ser::Error) -> Self {
        CloudAuthError::GenericOwned(error.to_string())
    }
}

impl std::convert::From<biscuit::errors::Error> for CloudAuthError {
    fn from(error: biscuit::errors::Error) -> Self {
        CloudAuthError::JWT(error)
    }
}

impl std::convert::From<biscuit::errors::ValidationError> for CloudAuthError {
    fn from(error: biscuit::errors::ValidationError) -> Self {
        CloudAuthError::JWTValidation(error)
    }
}

impl fmt::Display for CloudAuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CloudAuthError::TooManyRequests =>  write!(f, "Rate limited! Too many requests"),
            CloudAuthError::HttpError(ref url,ref status_line) => write!(f, "HTTP Error {} - {}", url, status_line),
            CloudAuthError::Timeout =>  write!(f, "Request expired"),
            CloudAuthError::Generic(m) => write!(f, "{}", m),
            CloudAuthError::GenericOwned(ref m) => write!(f, "{}", m),
            CloudAuthError::APIError(code, ref m, ref context) => {
                write!(f, "API Error! Code {} - {}. Context: {}", code, m, context)
            }
            CloudAuthError::JWT(ref e) => e.fmt(f),
            CloudAuthError::JWTValidation(ref e) => e.fmt(f),
            CloudAuthError::RSA(ref e) => e.fmt(f),
            CloudAuthError::IO(ref e) => e.fmt(f),
            CloudAuthError::Ser { ref doc, ref ser } => {
                if let Some(doc) = doc {
                    writeln!(f, "{} in document {}", ser, doc)
                } else {
                    ser.fmt(f)
                }
            }
        }
    }
}

impl error::Error for CloudAuthError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            CloudAuthError::JWT(ref e) => Some(e),
            CloudAuthError::JWTValidation(ref e) => Some(e),
            CloudAuthError::IO(ref e) => Some(e),
            CloudAuthError::Ser { ref ser, .. } => Some(ser),
            _ => None
        }
    }
}
