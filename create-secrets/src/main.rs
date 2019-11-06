#![deny(warnings)]

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

/**
* The build process of this service will:
*
* - Download missing jwks formatted public key files to be able to verify google access tokens.
* - Compute the jwks for the OHX auth private key.
* - Generate a random seed (binary data, 64 bytes). This is used to initialize TOTP.
*/

use std::io::prelude::*;
use std::io::{BufWriter, BufReader};
use std::fs::File;

use serde::{Serialize, Deserialize};
use serde_json::json;
use ring::signature::KeyPair;

use cloud_auth_lib::credentials::pem_to_der;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Credentials {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_provider_x509_cert_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_x509_cert_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_uri: Option<String>,
    pub r#type: Option<String>,
    pub project_id: String,
    pub private_key_id: String,
    pub private_key: String,
    pub client_email: String,
    pub client_id: String,
}

fn retrieve_jwks_for_google_account(
    mut path: std::path::PathBuf,
    account_mail: &str,
) -> Result<(), failure::Error> {
    path.push(&format!("{}.json", account_mail));

    if std::path::Path::new(&path).exists() {
        return Ok(());
    }

    if path.exists() {
        info!("Found existing {}. Not downloading again.", path.to_str().unwrap());
        return Ok(());
    }


    let mut resp = ureq::get(&format!("https://www.googleapis.com/service_accounts/v1/jwk/{}", account_mail)).call();
    if !resp.ok() {
        return Err(failure::err_msg(format!("Failed to download jwk for {}", account_mail)));
    }
    let file = File::create(path)?;
    let mut buffer = BufWriter::new(file);
    buffer.write_all(&resp.text()?.as_bytes())?;
    buffer.flush()?;
    Ok(())
}

fn create_random_seed(mut path: std::path::PathBuf, filename: &str) -> Result<(), failure::Error> {
    path.push(filename);

    let mut buffer = BufWriter::new(File::create(path)?);
    let mut buf = [0u8; 64];
    use ring::rand::{SecureRandom,SystemRandom};
    let rand = SystemRandom::new();
    rand.fill(&mut buf);
    buffer.write_all(&buf)?;
    buffer.flush()?;
    Ok(())
}

fn create_jwks(mut path: std::path::PathBuf, filename: &str, source: &str) -> Result<(), failure::Error> {
    path.push(filename);

    if std::path::Path::new(&path).exists() {
        return Ok(());
    }
    let filename = path.clone();
    path.pop();
    path.push(source);
    let source = path;

    let mut read_buffer = BufReader::new(File::open(source)?);
    let mut buffer = String::new();
    read_buffer.read_to_string(&mut buffer)?;

    let buffer = pem_to_der(&buffer)?;

    let key_pair = ring::signature::RsaKeyPair::from_pkcs8(&buffer).map_err(|f| failure::err_msg(f.description_()))?;

    use base64::{encode_config, Config, CharacterSet};

    let config = Config::new(CharacterSet::UrlSafe, false);
    let e = encode_config(key_pair.public_key().exponent().big_endian_without_leading_zero(), config.clone());
    let n = encode_config(key_pair.public_key().modulus().big_endian_without_leading_zero(), config);

    let mut buffer = BufWriter::new(File::create(filename)?);
    let jwks = json!({
        "keys": [
        {
          "kid": "d9c3af41-68aa-4a33-a3d9-6118ef0aac65",
          "e": e,
          "n": n,
          "kty": "RSA",
          "alg": "RS256",
          "use": "sig"
        }
      ]
    });
    buffer.write_all(&jwks.to_string().as_bytes())?;
    buffer.flush()?;

    Ok(())
}

fn add_private_key_to_credentials_file(mut path: std::path::PathBuf, filename: &str, source: &str) -> Result<(), failure::Error> {
    path.push(filename);

    let mut c: Credentials = serde_json::from_reader(BufReader::new(File::open(&path)?))?;
    if c.private_key.is_empty() {
        let mut read_buffer = BufReader::new(File::create(source)?);
        read_buffer.read_to_string(&mut c.private_key)?;
        if c.r#type.is_none() {
            c.r#type = Some("service_account".to_owned());
        }
        if c.auth_uri.is_none() {
            c.auth_uri = Some("https://openhabx.com/auth".to_owned());
        }
        if c.token_uri.is_none() {
            c.token_uri = Some("https://oauth.openhabx.com/token".to_owned());
        }
        if c.project_id.is_empty() {
            c.project_id = "openhabx".to_owned();
        }
        if c.private_key_id.is_empty() {
            c.private_key_id = "d9c3af41-68aa-4a33-a3d9-6118ef0aac65".to_owned();
        }
        if c.client_id.is_empty() {
            c.client_id = "1".to_owned();
        }
        if c.client_email.is_empty() {
            c.client_email = "admin@openhabx.com".to_owned();
        }

        let mut buffer = File::create(&path)?;
        serde_json::to_writer_pretty(&buffer, &c)?;
        buffer.flush()?;
    }

    Ok(())
}

fn create_ohx_certificate(mut path: std::path::PathBuf, filename: &str) -> Result<(), failure::Error> {
    path.push(filename);
    if std::path::Path::new(&path).exists() {
        info!("Not creating private/public key-pair for OHX. Already existing..");
        return Ok(());
    }

    let mut path = path.to_path_buf();
    path.push(filename);

    use rcgen::Certificate;
    let subject_alt_names = vec!["oauth.openhabx.com".to_string(), "vault.openhabx.com".to_string(), "openhabx.com".to_string()];

    let mut params = rcgen::CertificateParams::new(subject_alt_names);
    let cert = Certificate::from_params(params)?;
    let pem = cert.serialize_pem()?;
    let private_key = cert.serialize_private_key_pem();

    let file = File::create(path)?;
    let mut buffer = BufWriter::new(file);
    buffer.write_all(&private_key.as_bytes())?;
    buffer.flush()?;

    path.pop();
    path.push("ohx_oauth_key.pub");
    let file = File::create(path)?;
    let mut buffer = BufWriter::new(file);
    buffer.write_all(&pem.as_bytes())?;
    buffer.flush()?;

    Ok(())
}

fn main() -> Result<(), failure::Error> {
    simple_logger::init();
    let mut target_dir = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    target_dir.push("secrets");

    info!("The current directory is {}", &manifest_dir.display());

    retrieve_jwks_for_google_account(target_dir, "securetoken@system.gserviceaccount.com")?;
    retrieve_jwks_for_google_account(target_dir, "travisci-deployer@openhabx.iam.gserviceaccount.com")?;
    retrieve_jwks_for_google_account(target_dir, "openhabx-device@openhabx.iam.gserviceaccount.com")?;

    create_ohx_certificate(manifest_dir.as_path(), "ohx_oauth_key.pem")?;
    create_jwks(manifest_dir.as_path(), "ohx_oauth_key.json", "ohx_oauth_key.pem")?;
    add_private_key_to_credentials_file(manifest_dir.as_path(), "ohx_oauth_key.key", "ohx_oauth_key.pem")?;

    create_random_seed(manifest_dir.as_path(), "random_seed.bin")?;
    Ok(())
}