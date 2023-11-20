use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::consts::PUBLIC_KEY;

use super::{err, Result};

fn decode_hex_signature(hex: &str) -> Result<[u8; 64]> {
    const SIGNATURE_LENGTH: usize = 64;
    let mut decoded: [u8; SIGNATURE_LENGTH] = [0; SIGNATURE_LENGTH];
    let mut len = 0;
    for i in (0..hex.len()).step_by(2) {
        decoded[i / 2] = u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| e.to_string())?;
        len += 1;
    }

    if len == SIGNATURE_LENGTH {
        Ok(decoded)
    } else {
        err(format!("Invalid signature length: {} bytes.", len))
    }
}

pub const PUBLIC_KEY_LENGTH: usize = 32;

pub fn validate(sighex: &str, timestamp: &str, body: &str) -> Result<bool> {
    let verkey = VerifyingKey::from_bytes(PUBLIC_KEY).map_err(|e| format!("{e}"))?;
    let decoded = decode_hex_signature(sighex)?;
    let signature = Signature::from_bytes(&decoded);
    let message = format!("{timestamp}{body}");

    Ok(verkey.verify(message.as_bytes(), &signature).is_ok())
}
