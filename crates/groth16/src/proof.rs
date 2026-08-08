// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Groth16 proof representation.

use dusk_bytes::{DeserializableSlice, Serializable};
use dusk_curves::bls12_381::{G1Affine, G2Affine};

/// Canonical compressed size of a Groth16 proof.
pub const PROOF_SIZE: usize = G1Affine::SIZE * 2 + G2Affine::SIZE;

/// A Groth16 proof: two G1 elements and one G2 element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Proof {
    pub(crate) a: G1Affine,
    pub(crate) b: G2Affine,
    pub(crate) c: G1Affine,
}

impl Proof {
    /// First G1 proof element.
    pub const fn a(&self) -> &G1Affine {
        &self.a
    }

    /// G2 proof element.
    pub const fn b(&self) -> &G2Affine {
        &self.b
    }

    /// Second G1 proof element.
    pub const fn c(&self) -> &G1Affine {
        &self.c
    }
}

impl Serializable<PROOF_SIZE> for Proof {
    type Error = dusk_bytes::Error;

    fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        let g1_size = G1Affine::SIZE;
        let g2_end = g1_size + G2Affine::SIZE;
        bytes[..g1_size].copy_from_slice(&self.a.to_bytes());
        bytes[g1_size..g2_end].copy_from_slice(&self.b.to_bytes());
        bytes[g2_end..].copy_from_slice(&self.c.to_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8; Self::SIZE]) -> Result<Self, Self::Error> {
        let g1_size = G1Affine::SIZE;
        let g2_end = g1_size + G2Affine::SIZE;
        let proof = Self {
            a: G1Affine::from_slice(&bytes[..g1_size])?,
            b: G2Affine::from_slice(&bytes[g1_size..g2_end])?,
            c: G1Affine::from_slice(&bytes[g2_end..])?,
        };
        if bool::from(proof.a.is_identity())
            || bool::from(proof.b.is_identity())
            || bool::from(proof.c.is_identity())
        {
            return Err(dusk_bytes::Error::InvalidData);
        }
        Ok(proof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dusk_curves::bls12_381::BlsScalar;
    use sha2::{Digest, Sha256};

    #[test]
    fn proof_encoding_round_trip() {
        assert_eq!(PROOF_SIZE, 192);
        let proof = Proof {
            a: G1Affine::from(G1Affine::generator() * BlsScalar::from(2u64)),
            b: G2Affine::from(G2Affine::generator() * BlsScalar::from(3u64)),
            c: G1Affine::from(G1Affine::generator() * BlsScalar::from(4u64)),
        };
        let bytes = proof.to_bytes();
        assert_eq!(Proof::from_bytes(&bytes).unwrap(), proof);
        assert_eq!(
            <[u8; 32]>::from(Sha256::digest(bytes)),
            [
                40, 62, 158, 16, 207, 83, 168, 47, 57, 95, 15, 195, 98, 63, 128, 20, 180, 160, 58,
                207, 57, 222, 117, 183, 203, 219, 25, 178, 85, 233, 226, 251,
            ]
        );
    }

    #[test]
    fn identity_and_malformed_proof_encodings_are_rejected() {
        let valid = Proof {
            a: G1Affine::generator(),
            b: G2Affine::generator(),
            c: G1Affine::generator(),
        };

        for proof in [
            Proof {
                a: G1Affine::identity(),
                ..valid
            },
            Proof {
                b: G2Affine::identity(),
                ..valid
            },
            Proof {
                c: G1Affine::identity(),
                ..valid
            },
        ] {
            assert_eq!(
                Proof::from_bytes(&proof.to_bytes()),
                Err(dusk_bytes::Error::InvalidData)
            );
        }

        let mut malformed = valid.to_bytes();
        malformed[..G1Affine::SIZE].fill(0);
        assert_eq!(
            Proof::from_bytes(&malformed),
            Err(dusk_bytes::Error::InvalidData)
        );
    }
}
