use ring::aead::CHACHA20_POLY1305;
use ring::aead::{Nonce, OpeningKey, SealingKey, UnboundKey};
use ring::error::Unspecified;

use crate::responder_type::MyResponder;
use biscuit::jws::Secret;
use cloud_vault::jwt;

const SECRET: &[u8] = include_bytes!("../secrets/random_seed.bin");

struct NullNonceSequence();

impl ring::aead::NonceSequence for NullNonceSequence {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        Ok(Nonce::assume_unique_for_key([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]))
    }
}

pub fn hash_of_token(token: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.input(token);
    let result = hasher.result();
    let config = base64::Config::new(base64::CharacterSet::UrlSafe, false);
    base64::encode_config(result.as_slice(), config)
}

#[cfg(test)]
#[inline]
fn err_wrap<T>(r: Result<T, impl std::fmt::Debug>) -> Result<T, MyResponder> {
    r.map_err(|e| MyResponder::InternalError(format!("Failed to bincode::deserialize {:?}", e)))
}

pub fn decrypt_unsigned_jwt_token(token_base64: &[u8]) -> Result<jwt::AuthClaimsJWT, MyResponder> {
    use miniz_oxide::inflate::decompress_to_vec;
    use ring::aead::BoundKey;
    use biscuit::jwa::SignatureAlgorithm;

    let nonce_seq = NullNonceSequence();

    // "token_base64" is := BASE64(AES_ENCRYPTED(COMPRESSED(BINARY_SERIALIZED(jwt))))
    // Therefore it 1.) need to be base64 decoded into bytes
    let config = base64::Config::new(base64::CharacterSet::UrlSafe, false);
    let mut buffer = base64::decode_config(&token_base64, config)?;
    // 2. decrypted
    let key = UnboundKey::new(&CHACHA20_POLY1305, &SECRET[0..32])?;
    let mut key = OpeningKey::new(key, nonce_seq);
    let buffer = key.open_within(ring::aead::Aad::empty(), &mut buffer, 0..)?;
    // 3. decompressed
    let buffer = &decompress_to_vec(buffer)?[..];
    // 4. deserialize
//    Ok(err_wrap(bincode::deserialize(buffer))?)

    let decoded: jwt::AuthClaimsJWT = serde_json::from_slice(buffer)?;
    let decoded = decoded.into_decoded(&Secret::None, SignatureAlgorithm::None).unwrap();
    Ok(decoded)
}

/// Serialize, compress, encrypt and base64 a jwt
pub fn encrypt_unsigned_jwt_token(mut jwt: jwt::AuthClaimsJWT) -> Result<String, MyResponder> {
    use miniz_oxide::deflate::compress_to_vec;
    use ring::aead::BoundKey;
    use biscuit::jwa::SignatureAlgorithm;

    let nonce_seq = NullNonceSequence();

    // The jwt must be encoded first and should not carry a signature (Secret::None)
    jwt.header_mut().unwrap().registered.algorithm = SignatureAlgorithm::None;
    let jwt = jwt.encode(&Secret::None)?;

    // We need to convert jwt so that result := BASE64(AES_ENCRYPTED(COMPRESSED(BINARY_SERIALIZED(jwt))))
    // 1. Binary serialize
    //let mut buffer: Vec<u8> = err_wrap(bincode::serialize(&jwt))?;
    let buffer: Vec<u8> = serde_json::to_vec(&jwt)?;
    // 2. Compress
    let mut buffer = compress_to_vec(&buffer, 9);
    // 3. encrypt
    let key = UnboundKey::new(&CHACHA20_POLY1305, &SECRET[0..32])?;
    let mut key = SealingKey::new(key, nonce_seq);
    key.seal_in_place_append_tag(ring::aead::Aad::empty(), &mut buffer)?;

    // 4. to base64 string
    let config = base64::Config::new(base64::CharacterSet::UrlSafe, false);
    Ok(base64::encode_config(&buffer, config))
}

#[test]
fn test_enc_dec() {
    use cloud_vault::credentials::Credentials;
    use ring::aead::BoundKey;

    let credentials: Credentials = serde_json::from_value(serde_json::json!({
        "project_id": "project_id",
        "private_key_id": "key_id",
        "private_key": "",
        "client_email": "some@email.com",
        "client_id": "client_id"
    })).unwrap();

    let encoded: Vec<u8> = err_wrap(bincode::serialize(&credentials)).unwrap();
    let mut buffer = encoded.clone();
    let nonce_seq = NullNonceSequence();
    let key = UnboundKey::new(&CHACHA20_POLY1305, &SECRET[0..32]).unwrap();
    let mut key = SealingKey::new(key, nonce_seq);
    key.seal_in_place_append_tag(ring::aead::Aad::empty(), &mut buffer).unwrap();
    let nonce_seq = NullNonceSequence();
    let key = UnboundKey::new(&CHACHA20_POLY1305, &SECRET[0..32]).unwrap();
    let mut key = OpeningKey::new(key, nonce_seq);
    let enc_slice = key.open_within(ring::aead::Aad::empty(), &mut buffer, 0..).unwrap();
    assert_eq!(enc_slice.len(), encoded.len());
    assert_eq!(enc_slice, &encoded[..]);
    let c: Credentials = bincode::deserialize(&enc_slice[..]).unwrap();
    assert_eq!(c.project_id, "project_id");
}

// Test bincode
//    let test_bincode_jwt = jwt.encode(&Secret::None).unwrap();
//    let test_bincode: Vec<u8> = bincode::serialize(&test_bincode_jwt).unwrap();
//    let _c: jwt::AuthClaimsJWT = bincode::deserialize(&test_bincode[..]).unwrap();
//assert_eq!(c.project_id, "project_id");

#[test]
fn encrypt_decrypt_test() {
    use cloud_vault::credentials::Credentials;

    let credentials: Credentials = serde_json::from_value(serde_json::json!({
        "project_id": "project_id",
        "private_key_id": "key_id",
        "private_key": "",
        "client_email": "some@email.com",
        "client_id": "client_id"
    })).unwrap();

    let jwt = jwt::create_jwt(
        &credentials,
        Some(vec!("demo")),
        chrono::Duration::hours(1),
        Some("client_id".to_owned()),
        Some("user_id".to_owned()),
        &credentials.client_email
    ).unwrap();

    let encoded = encrypt_unsigned_jwt_token(jwt).unwrap();

    let decoded = decrypt_unsigned_jwt_token(&encoded.as_bytes()).unwrap();
    let payload = decoded.payload().unwrap();

    assert_eq!(payload.registered.subject.as_ref().unwrap().to_string(), "some@email.com");
    assert_eq!(payload.private.scope.as_ref().unwrap().to_string(), "demo ");
    assert_eq!(payload.private.client_id.as_ref().unwrap().to_string(), "client_id");
    assert_eq!(payload.private.uid.as_ref().unwrap().to_string(), "user_id");
}