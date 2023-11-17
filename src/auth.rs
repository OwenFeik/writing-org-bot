use ed25519_dalek::{Signature, Verifier, VerifyingKey};

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

pub fn validate(sighex: &str, timestamp: &str, body: &str) -> Result<bool> {
    const PUBLIC_KEY_LENGTH: usize = 32;
    const PUBLIC_KEY: &[u8; PUBLIC_KEY_LENGTH] = &[
        0xDC, 0x72, 0xF9, 0xAB, 0x92, 0x57, 0x1D, 0x06, 0x98, 0x7A, 0xF6, 0x5C, 0x54, 0x2B, 0x7B,
        0x41, 0xF5, 0x6E, 0x01, 0x38, 0xEC, 0x9C, 0x01, 0x6B, 0xF9, 0x52, 0xD3, 0xC2, 0x62, 0x5F,
        0xB3, 0x00,
    ];

    let verkey = VerifyingKey::from_bytes(PUBLIC_KEY).map_err(|e| format!("{e}"))?;
    let decoded = decode_hex_signature(sighex)?;
    let signature = Signature::from_bytes(&decoded);
    let message = format!("{timestamp}{body}");

    Ok(verkey.verify(message.as_bytes(), &signature).is_ok())
}
