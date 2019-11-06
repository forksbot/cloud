#![deny(warnings)]

pub mod dto;
pub mod oauth_clients;
pub mod token;
pub mod jwt;
pub mod tools;
pub mod login;
mod credentials;
mod rocket_helper;
mod errors;

pub use rocket_helper::*;
pub use errors::CloudAuthError;
pub use credentials::Credentials;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

/// Converts a PEM (ascii base64) encoded private key into the binary der representation
pub fn pem_to_der(pem_file_contents: &str) -> Result<Vec<u8>, CloudAuthError> {
    use base64::decode;

    let pem_file_contents = pem_file_contents.find("-----BEGIN")
        // Cut off the first BEGIN part
        .and_then(|i| Some(&pem_file_contents[i + 10..]))
        // Find the trailing ---- after BEGIN and cut that off
        .and_then(|str| str.find("-----").and_then(|i| Some(&str[i + 5..])))
        // Cut off -----END
        .and_then(|str| str.rfind("-----END").and_then(|i| Some(&str[..i])));
    if pem_file_contents.is_none() {
        return Err(CloudAuthError::Generic("Invalid private key in credentials file. Must be valid PEM."));
    }

    let base64_body = pem_file_contents.unwrap().replace("\n", "");
    Ok(decode(&base64_body)?)
}

