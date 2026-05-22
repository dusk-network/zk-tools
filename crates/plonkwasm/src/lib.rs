// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_bytes::Serializable;
use dusk_plonk::prelude::{BlsScalar, Circuit, Error as PlonkError, Proof, Prover, Verifier};
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofOutput {
    /// Serialized `dusk_plonk::Proof` bytes.
    pub proof: Vec<u8>,
    /// Serialized public inputs as concatenated 32-byte scalar encodings.
    pub public_inputs: Vec<u8>,
}

/// Computes a PLONK proof for a concrete circuit using serialized prover keys.
///
/// The caller supplies the circuit because a wasm proof binary must link the
/// circuit implementation at compile time.
pub fn prove<C>(prover_key: &[u8], seed: [u8; 32], circuit: &C) -> Result<ProofOutput, String>
where
    C: Circuit,
{
    let prover = Prover::try_from_bytes(prover_key).map_err(format_plonk_error)?;
    let mut rng = ChaCha20Rng::from_seed(seed);
    let (proof, public_inputs) = prover
        .prove(&mut rng, circuit)
        .map_err(format_plonk_error)?;

    Ok(ProofOutput {
        proof: proof.to_bytes().to_vec(),
        public_inputs: serialize_public_inputs(&public_inputs),
    })
}

/// Verifies a serialized PLONK proof using serialized verifier keys and public inputs.
pub fn verify(verifier_key: &[u8], proof: &[u8], public_inputs: &[u8]) -> Result<(), String> {
    let verifier = Verifier::try_from_bytes(verifier_key).map_err(format_plonk_error)?;
    let proof = Proof::from_bytes(
        proof
            .try_into()
            .map_err(|_| format!("invalid proof length: {}", proof.len()))?,
    )
    .map_err(|err| format!("{err:?}"))?;
    let public_inputs = deserialize_public_inputs(public_inputs)?;

    verifier
        .verify(&proof, &public_inputs)
        .map_err(format_plonk_error)
}

/// Serializes public inputs in the same order returned by `dusk-plonk`.
pub fn serialize_public_inputs(public_inputs: &[BlsScalar]) -> Vec<u8> {
    public_inputs
        .iter()
        .flat_map(|input| input.to_bytes())
        .collect()
}

/// Deserializes public inputs from concatenated 32-byte scalar encodings.
pub fn deserialize_public_inputs(public_inputs: &[u8]) -> Result<Vec<BlsScalar>, String> {
    if public_inputs.len() % 32 != 0 {
        return Err(format!(
            "public inputs length must be a multiple of 32 bytes, got {}",
            public_inputs.len()
        ));
    }

    public_inputs
        .chunks_exact(32)
        .map(|chunk| {
            let bytes: [u8; 32] = chunk
                .try_into()
                .map_err(|_| "public input chunk must be 32 bytes".to_string())?;
            Option::<BlsScalar>::from(BlsScalar::from_bytes(&bytes))
                .ok_or_else(|| "invalid public input scalar encoding".to_string())
        })
        .collect()
}

pub mod wasm {
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    struct ApiResponse<T: Serialize> {
        ok: bool,
        result: Option<T>,
        error: Option<String>,
    }

    /// Allocates wasm memory for a JavaScript caller to write a request.
    pub fn alloc(len: usize) -> *mut u8 {
        let mut bytes = vec![0u8; len].into_boxed_slice();
        let ptr = bytes.as_mut_ptr();
        core::mem::forget(bytes);
        ptr
    }

    /// Releases memory previously returned by `alloc` or `respond_from_request`.
    pub unsafe fn free(ptr: *mut u8, len: usize) {
        if !ptr.is_null() && len != 0 {
            drop(Vec::from_raw_parts(ptr, len, len));
        }
    }

    /// Converts a raw JSON request into a packed `(ptr, len)` JSON response.
    ///
    /// Circuit-specific wasm crates expose their `#[no_mangle]` functions and
    /// delegate request handling here to keep the ABI consistent.
    pub unsafe fn respond_from_request<T, F>(
        request_ptr: *const u8,
        request_len: usize,
        handler: F,
    ) -> u64
    where
        T: Serialize,
        F: FnOnce(&[u8]) -> Result<T, String>,
    {
        let request = core::slice::from_raw_parts(request_ptr, request_len);
        let response = match handler(request) {
            Ok(result) => ApiResponse {
                ok: true,
                result: Some(result),
                error: None,
            },
            Err(error) => ApiResponse::<T> {
                ok: false,
                result: None,
                error: Some(error),
            },
        };

        let bytes = serde_json::to_vec(&response).unwrap_or_else(|err| {
            format!(r#"{{"ok":false,"result":null,"error":"response encode failed: {err}"}}"#)
                .into_bytes()
        });
        let mut bytes = bytes.into_boxed_slice();
        let ptr = bytes.as_mut_ptr() as u64;
        let len = bytes.len() as u64;
        core::mem::forget(bytes);

        (ptr << 32) | len
    }

    pub fn decode_seed(hex: &str) -> Result<[u8; 32], String> {
        let bytes = decode_hex(hex)?;
        bytes
            .try_into()
            .map_err(|bytes: Vec<u8>| format!("seed must be 32 bytes, got {}", bytes.len()))
    }

    pub fn encode_hex(bytes: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            out.push(HEX[(byte >> 4) as usize] as char);
            out.push(HEX[(byte & 0x0f) as usize] as char);
        }
        out
    }

    pub fn decode_hex(hex: &str) -> Result<Vec<u8>, String> {
        let hex = hex.strip_prefix("0x").unwrap_or(hex);
        if hex.len() % 2 != 0 {
            return Err("hex string must have even length".to_string());
        }

        hex.as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                let high = decode_hex_nibble(pair[0])?;
                let low = decode_hex_nibble(pair[1])?;
                Ok((high << 4) | low)
            })
            .collect()
    }

    fn decode_hex_nibble(nibble: u8) -> Result<u8, String> {
        match nibble {
            b'0'..=b'9' => Ok(nibble - b'0'),
            b'a'..=b'f' => Ok(nibble - b'a' + 10),
            b'A'..=b'F' => Ok(nibble - b'A' + 10),
            _ => Err(format!("invalid hex character: {}", nibble as char)),
        }
    }
}

fn format_plonk_error(err: PlonkError) -> String {
    format!("{err:?}")
}
