//! Encrypt/Decrypt payload messages
use super::Error;

use ring::aead::*;
use ring::pbkdf2::*;
use ring::rand::SystemRandom;
use ring::rand::SecureRandom;

use futures::Stream;
use futures::Sink;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{fmt, io};
use tokio::io::AsyncRead;
use std::mem::size_of;
use bytes::ByteOrder;
use ring::error::Unspecified;

/// A nonce sequence. This is a simple counter, seeded with a random number (using the system
/// random generator, see Rings documentation for [`ring::rand::SystemRandom`])
pub struct CounterNonce {
    counter: u64,
    additional: [u8; 4],
}

impl CounterNonce {
    /// Initialize the counter nonce with a random seed
    pub fn new() -> Self {
        let mut nonce_bytes: [u8; size_of::<u64>()] = [0; size_of::<u64>()];
        let mut additional: [u8; 4] = [0; 4];
        let rand = SystemRandom::new();
        rand.fill(&mut nonce_bytes).unwrap();
        rand.fill(&mut additional).unwrap();
        CounterNonce {
            counter: bytes::BigEndian::read_u64(&nonce_bytes),
            additional,
        }
    }
}

impl ring::aead::NonceSequence for CounterNonce {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        let mut nonce_bytes: [u8; 12] = [0; 12];
        bytes::BigEndian::write_u64(&mut nonce_bytes, self.counter);
        nonce_bytes[8..12].copy_from_slice(&self.additional);
        self.counter += 1;
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    }
}

pub type EncryptionKey = [u8; 32];

/// An encrypt / decrypt stream for tokios AsyncRead/AsyncWrite.
///
/// Keys can be updated at any time, multiple known keys are supported.
/// Before
pub struct EncodeDecode {
    key_id: u64,
    keys: smallvec::SmallVec<[EncryptionKey; 3]>,
    salt: [u8; 8],
    nonce: CounterNonce,
}

fn encrypt() -> Result<(), Error> {
    // The password will be used to generate a key
    let password = b"nice password";

    // Usually the salt has some random data and something that relates to the user
    // like an username
    let salt = [0, 1, 2, 3, 4, 5, 6, 7];

    // Keys are sent as &[T] and must have 32 bytes
    let mut key = [0; 32];
    derive(PBKDF2_HMAC_SHA512, unsafe { std::num::NonZeroU32::new_unchecked(100u32) }, &salt, &password[..], &mut key);

    // Your private data
    let content = b"content to encrypt".to_vec();
    println!("Content to encrypt's size {}", content.len());

    // Additional data that you would like to send and it would not be encrypted but it would
    // be signed
    let additional_data: [u8; 0] = [];

    // Ring uses the same input variable as output
    let mut in_out = content.clone();

    // The input/output variable need some space for a suffix
    println!("Tag len {}", CHACHA20_POLY1305.tag_len());
    for _ in 0..CHACHA20_POLY1305.tag_len() {
        in_out.push(0);
    }

    // Random data must be used only once per encryption
    let mut nonce_bytes: [u8; 12] = [0; 12];
    let rand = SystemRandom::new();
    rand.fill(&mut nonce_bytes).unwrap();
    let nonce = Nonce::assume_unique_for_key(nonce_bytes.clone());

    // Sealing key used to encrypt data
    let mut sealing_key = LessSafeKey::new(UnboundKey::new(&CHACHA20_POLY1305, &key)?);

    // Encrypt data into in_out variable
    sealing_key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)?;

    println!("Encrypted data's size {}", in_out.len());

    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let mut opening_key = LessSafeKey::new(UnboundKey::new(&CHACHA20_POLY1305, &key)?);
    let decrypted_data = opening_key.open_in_place(nonce, Aad::empty(), &mut in_out).unwrap();

    println!("{:?}", String::from_utf8(decrypted_data.to_vec()).unwrap());
    assert_eq!(content, decrypted_data);
    Ok(())
}
