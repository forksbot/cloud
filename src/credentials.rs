use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;
use std::sync::Arc;
use chrono::Duration;

use crate::jwt::{create_jwt_encoded, verify_access_token, JWKSetDTO, TokenValidationResult, create_jwt_encoded_for_user};

/// This is not defined in the json file and computed
#[derive(Default)]
pub(crate) struct Keys {
    pub pub_key: BTreeMap<String, Arc<biscuit::jws::Secret>>,
    pub secret: Option<Arc<biscuit::jws::Secret>>,
}

/// Service account credentials
///
/// Especially the service account email is required to retrieve the public java web key set (jwks)
/// for verifying Google Cloud tokens.
///
/// The private key is used for signing java web tokens (jwk).
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Credentials {
    pub project_id: String,
    pub private_key_id: String,
    pub private_key: String,
    pub client_email: String,
    pub client_id: String,
    #[serde(default, skip)]
    pub(crate) keys: Keys,
}

impl Clone for Keys {
    fn clone(&self) -> Self {
        Self { pub_key: Default::default(), secret: None }
    }
}

/// Converts a PEM (ascii base64) encoded private key into the binary der representation
pub fn pem_to_der(pem_file_contents: &str) -> Result<Vec<u8>, failure::Error> {
    use base64::decode;

    let pem_file_contents = pem_file_contents.find("-----BEGIN")
        // Cut off the first BEGIN part
        .and_then(|i| Some(&pem_file_contents[i + 10..]))
        // Find the trailing ---- after BEGIN and cut that off
        .and_then(|str| str.find("-----").and_then(|i| Some(&str[i + 5..])))
        // Cut off -----END
        .and_then(|str| str.rfind("-----END").and_then(|i| Some(&str[..i])));
    if pem_file_contents.is_none() {
        return Err(failure::err_msg("Invalid private key in credentials file. Must be valid PEM."));
    }

    let base64_body = pem_file_contents.unwrap().replace("\n", "");
    Ok(decode(&base64_body)?)
}


impl Credentials {
    pub fn load_and_check<L, T>(credentials_file: &str, jwks_files: &[&str], scope: Option<L>) -> Result<(Credentials, String, TokenValidationResult), failure::Error>
        where L: IntoIterator<Item=T>, T: AsRef<str> {
        let mut credentials: Credentials = serde_json::from_str(credentials_file)?;
        for jwks_file in jwks_files {
            credentials.add_jwks_public_keys(serde_json::from_str(jwks_file)?);
        }
        credentials.compute_secret()?;
        let access_token = create_jwt_encoded(&credentials, scope, Duration::hours(1), Some(credentials.client_id.clone()))?;
        let validation_result = verify_access_token(&credentials, &access_token)?.unwrap();
        Ok((credentials, access_token, validation_result))
    }

    pub fn load_and_check_for_user<L, T>(credentials_file: &str, jwks_files: &[&str], scope: Option<L>, user_id: String) -> Result<(Credentials, String, TokenValidationResult), failure::Error>
        where L: IntoIterator<Item=T>, T: AsRef<str> {
        let mut credentials: Credentials = serde_json::from_str(credentials_file)?;
        for jwks_file in jwks_files {
            credentials.add_jwks_public_keys(serde_json::from_str(jwks_file)?);
        }
        credentials.compute_secret()?;
        let access_token = create_jwt_encoded_for_user(&credentials, scope, Duration::hours(1), Some(credentials.client_id.clone()), user_id.clone(), user_id)?;
        let validation_result = verify_access_token(&credentials, &access_token)?.unwrap();
        Ok((credentials, access_token, validation_result))
    }

    /// Find the secret in the jwt set that matches the given key id, if any.
    /// Used for jws validation
    pub fn decode_secret(&self, kid: &str) -> Option<Arc<biscuit::jws::Secret>> {
        self.keys.pub_key.get(kid).and_then(|f| Some(f.clone()))
    }

    pub fn encode_secret(&self) -> Option<&Arc<biscuit::jws::Secret>> {
        self.keys.secret.as_ref()
    }

    /// Compute the Rsa keypair. It is not used for jwt verification, but for creating own jwts and sign them.
    pub fn compute_secret(&mut self) -> Result<(), failure::Error> {
        use ring::signature;
        use biscuit::jws::Secret;

        let vec = pem_to_der(&self.private_key)?;
        let key_pair = signature::RsaKeyPair::from_pkcs8(&vec).map_err(|f| failure::err_msg(f.description_()))?;
        self.keys.secret = Some(Arc::new(Secret::RsaKeyPair(Arc::new(key_pair))));
        Ok(())
    }

    /// There hereby added key in pkcs8-der format is used for jwt verification.
    /// This is similar to the keys added by [add_jwks].
    #[allow(dead_code)]
    pub fn add_pub_key(&mut self, key_id: String, pub_der: Vec<u8>) -> Result<(), failure::Error> {
        use biscuit::jws::Secret;
        self.keys.pub_key.insert(key_id, Arc::new(Secret::PublicKey(pub_der)));
        Ok(())
    }

    /// Add a JSON Web Key Set (JWKS) to allow verification of incoming http requests with bearer tokens in jwts format.
    pub fn add_jwks_public_keys(&mut self, jwkset: JWKSetDTO) {
        for entry in jwkset.keys.iter() {
            if !entry.headers.key_id.is_some() {
                continue;
            }

            let key_id = entry.headers.key_id.as_ref().unwrap().to_owned();
            self.keys.pub_key.insert(key_id, Arc::new(entry.ne.jws_public_key_secret()));
        }
    }
}
