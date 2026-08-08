// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Groth16 proving and verification keys.

use alloc::vec::Vec;

use dusk_bytes::{DeserializableSlice, Serializable};
use dusk_curves::bls12_381::{G1Affine, G2Affine, G2Prepared, Gt, multi_miller_loop_result};

use crate::Error;

const KEY_MAGIC: &[u8; 8] = b"DUSKG16K";
const KEY_VERSION: u8 = 1;
const PROVING_KEY_KIND: u8 = 1;
const VERIFYING_KEY_KIND: u8 = 2;

/// Circuit-specific Groth16 proving key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvingKey {
    pub(crate) shape_digest: [u8; 32],
    pub(crate) variables: usize,
    pub(crate) public_inputs: usize,
    pub(crate) domain_size: usize,
    pub(crate) alpha_g1: G1Affine,
    pub(crate) beta_g1: G1Affine,
    pub(crate) beta_g2: G2Affine,
    pub(crate) delta_g1: G1Affine,
    pub(crate) delta_g2: G2Affine,
    pub(crate) a_query: Vec<G1Affine>,
    pub(crate) b_g1_query: Vec<G1Affine>,
    pub(crate) b_g2_query: Vec<G2Affine>,
    pub(crate) private_query: Vec<G1Affine>,
    pub(crate) h_query: Vec<G1Affine>,
}

impl ProvingKey {
    /// Digest of the R1CS shape bound by this key.
    pub const fn shape_digest(&self) -> &[u8; 32] {
        &self.shape_digest
    }

    /// Number of public inputs expected by this key.
    pub const fn public_inputs(&self) -> usize {
        self.public_inputs
    }

    /// Serialize this key using the versioned canonical key format.
    ///
    /// The encoding starts with the `DUSKG16K` magic, a format version and key
    /// kind, followed by fixed-width metadata, compressed subgroup points, and
    /// length-prefixed query vectors.
    pub fn to_bytes(&self) -> Vec<u8> {
        let capacity = 10
            + 32
            + 8 * 8
            + G1Affine::SIZE
                * (3 + self.a_query.len()
                    + self.b_g1_query.len()
                    + self.private_query.len()
                    + self.h_query.len())
            + G2Affine::SIZE * (2 + self.b_g2_query.len());
        let mut bytes = Vec::with_capacity(capacity);
        write_header(&mut bytes, PROVING_KEY_KIND);
        bytes.extend_from_slice(&self.shape_digest);
        write_usize(&mut bytes, self.variables);
        write_usize(&mut bytes, self.public_inputs);
        write_usize(&mut bytes, self.domain_size);
        write_g1(&mut bytes, &self.alpha_g1);
        write_g1(&mut bytes, &self.beta_g1);
        write_g2(&mut bytes, &self.beta_g2);
        write_g1(&mut bytes, &self.delta_g1);
        write_g2(&mut bytes, &self.delta_g2);
        write_g1_vec(&mut bytes, &self.a_query);
        write_g1_vec(&mut bytes, &self.b_g1_query);
        write_g2_vec(&mut bytes, &self.b_g2_query);
        write_g1_vec(&mut bytes, &self.private_query);
        write_g1_vec(&mut bytes, &self.h_query);
        bytes
    }

    /// Decode and validate a key produced by [`ProvingKey::to_bytes`].
    ///
    /// Decoding rejects wrong magic/version/kind values, malformed or
    /// non-subgroup points, identity fixed parameters, overflowing lengths,
    /// inconsistent query sizes, truncation, and trailing data. It validates
    /// structure, not that a setup ceremony was honest.
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let mut reader = Reader::new(bytes);
        reader.header(PROVING_KEY_KIND)?;
        let shape_digest = reader.array()?;
        let variables = reader.usize()?;
        let public_inputs = reader.usize()?;
        let domain_size = reader.usize()?;
        let alpha_g1 = reader.g1()?;
        let beta_g1 = reader.g1()?;
        let beta_g2 = reader.g2()?;
        let delta_g1 = reader.g1()?;
        let delta_g2 = reader.g2()?;
        let a_query = reader.g1_vec()?;
        let b_g1_query = reader.g1_vec()?;
        let b_g2_query = reader.g2_vec()?;
        let private_query = reader.g1_vec()?;
        let h_query = reader.g1_vec()?;
        reader.finish()?;

        let private_variables = variables
            .checked_sub(public_inputs.checked_add(1).ok_or(Error::InvalidEncoding)?)
            .ok_or(Error::InvalidEncoding)?;
        if domain_size == 0
            || !domain_size.is_power_of_two()
            || bool::from(alpha_g1.is_identity())
            || bool::from(beta_g1.is_identity())
            || bool::from(beta_g2.is_identity())
            || bool::from(delta_g1.is_identity())
            || bool::from(delta_g2.is_identity())
            || a_query.len() != variables
            || b_g1_query.len() != variables
            || b_g2_query.len() != variables
            || private_query.len() != private_variables
            || h_query.len() != domain_size - 1
        {
            return Err(Error::InvalidEncoding);
        }

        Ok(Self {
            shape_digest,
            variables,
            public_inputs,
            domain_size,
            alpha_g1,
            beta_g1,
            beta_g2,
            delta_g1,
            delta_g2,
            a_query,
            b_g1_query,
            b_g2_query,
            private_query,
            h_query,
        })
    }
}

/// Circuit-specific Groth16 verification key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyingKey {
    pub(crate) alpha_g1: G1Affine,
    pub(crate) beta_g2: G2Affine,
    pub(crate) gamma_g2: G2Affine,
    pub(crate) delta_g2: G2Affine,
    pub(crate) public_query: Vec<G1Affine>,
}

impl VerifyingKey {
    /// Number of public inputs expected by this key.
    pub fn public_inputs(&self) -> usize {
        self.public_query.len().saturating_sub(1)
    }

    /// Public-input query, including the constant-one entry at index zero.
    pub fn public_query(&self) -> &[G1Affine] {
        &self.public_query
    }

    /// Precompute the fixed pairing terms used during verification.
    pub fn prepare(&self) -> PreparedVerifyingKey {
        let beta_g2 = G2Prepared::from(self.beta_g2);
        PreparedVerifyingKey {
            alpha_beta: multi_miller_loop_result(&[(&self.alpha_g1, &beta_g2)]),
            gamma_g2: G2Prepared::from(self.gamma_g2),
            delta_g2: G2Prepared::from(self.delta_g2),
            key: self.clone(),
        }
    }

    /// Serialize this key using the versioned canonical key format.
    ///
    /// The format shares the `DUSKG16K` header with proving keys and uses a
    /// distinct key-kind byte.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(
            10 + G1Affine::SIZE * (1 + self.public_query.len()) + G2Affine::SIZE * 3 + 8,
        );
        write_header(&mut bytes, VERIFYING_KEY_KIND);
        write_g1(&mut bytes, &self.alpha_g1);
        write_g2(&mut bytes, &self.beta_g2);
        write_g2(&mut bytes, &self.gamma_g2);
        write_g2(&mut bytes, &self.delta_g2);
        write_g1_vec(&mut bytes, &self.public_query);
        bytes
    }

    /// Decode and validate a key produced by [`VerifyingKey::to_bytes`].
    ///
    /// Decoding validates the header, canonical subgroup point encodings,
    /// non-identity fixed parameters, nonempty public query, exact lengths,
    /// and absence of trailing data. Algebraic setup consistency remains a
    /// property of the key source.
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let mut reader = Reader::new(bytes);
        reader.header(VERIFYING_KEY_KIND)?;
        let alpha_g1 = reader.g1()?;
        let beta_g2 = reader.g2()?;
        let gamma_g2 = reader.g2()?;
        let delta_g2 = reader.g2()?;
        let public_query = reader.g1_vec()?;
        reader.finish()?;
        if public_query.is_empty()
            || bool::from(alpha_g1.is_identity())
            || bool::from(beta_g2.is_identity())
            || bool::from(gamma_g2.is_identity())
            || bool::from(delta_g2.is_identity())
        {
            return Err(Error::InvalidEncoding);
        }
        Ok(Self {
            alpha_g1,
            beta_g2,
            gamma_g2,
            delta_g2,
            public_query,
        })
    }
}

/// Verification key with fixed G2 pairing terms precomputed.
#[derive(Clone, Debug)]
pub struct PreparedVerifyingKey {
    pub(crate) key: VerifyingKey,
    pub(crate) alpha_beta: Gt,
    pub(crate) gamma_g2: G2Prepared,
    pub(crate) delta_g2: G2Prepared,
}

impl PreparedVerifyingKey {
    /// Access the underlying canonical verification key.
    pub const fn key(&self) -> &VerifyingKey {
        &self.key
    }
}

fn write_header(bytes: &mut Vec<u8>, kind: u8) {
    bytes.extend_from_slice(KEY_MAGIC);
    bytes.push(KEY_VERSION);
    bytes.push(kind);
}

fn write_usize(bytes: &mut Vec<u8>, value: usize) {
    bytes.extend_from_slice(&(value as u64).to_le_bytes());
}

fn write_g1(bytes: &mut Vec<u8>, point: &G1Affine) {
    bytes.extend_from_slice(&point.to_bytes());
}

fn write_g2(bytes: &mut Vec<u8>, point: &G2Affine) {
    bytes.extend_from_slice(&point.to_bytes());
}

fn write_g1_vec(bytes: &mut Vec<u8>, points: &[G1Affine]) {
    write_usize(bytes, points.len());
    for point in points {
        write_g1(bytes, point);
    }
}

fn write_g2_vec(bytes: &mut Vec<u8>, points: &[G2Affine]) {
    write_usize(bytes, points.len());
    for point in points {
        write_g2(bytes, point);
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        let end = self.offset.checked_add(N).ok_or(Error::InvalidEncoding)?;
        let slice = self
            .bytes
            .get(self.offset..end)
            .ok_or(Error::InvalidEncoding)?;
        self.offset = end;
        slice.try_into().map_err(|_| Error::InvalidEncoding)
    }

    fn header(&mut self, kind: u8) -> Result<(), Error> {
        if &self.array::<8>()? != KEY_MAGIC
            || self.array::<1>()?[0] != KEY_VERSION
            || self.array::<1>()?[0] != kind
        {
            return Err(Error::InvalidEncoding);
        }
        Ok(())
    }

    fn usize(&mut self) -> Result<usize, Error> {
        usize::try_from(u64::from_le_bytes(self.array()?)).map_err(|_| Error::InvalidEncoding)
    }

    fn g1(&mut self) -> Result<G1Affine, Error> {
        G1Affine::from_slice(&self.array::<{ G1Affine::SIZE }>()?)
            .map_err(|_| Error::InvalidEncoding)
    }

    fn g2(&mut self) -> Result<G2Affine, Error> {
        G2Affine::from_slice(&self.array::<{ G2Affine::SIZE }>()?)
            .map_err(|_| Error::InvalidEncoding)
    }

    fn g1_vec(&mut self) -> Result<Vec<G1Affine>, Error> {
        let length = self.usize()?;
        let byte_length = length
            .checked_mul(G1Affine::SIZE)
            .ok_or(Error::InvalidEncoding)?;
        if byte_length > self.bytes.len().saturating_sub(self.offset) {
            return Err(Error::InvalidEncoding);
        }
        (0..length).map(|_| self.g1()).collect()
    }

    fn g2_vec(&mut self) -> Result<Vec<G2Affine>, Error> {
        let length = self.usize()?;
        let byte_length = length
            .checked_mul(G2Affine::SIZE)
            .ok_or(Error::InvalidEncoding)?;
        if byte_length > self.bytes.len().saturating_sub(self.offset) {
            return Err(Error::InvalidEncoding);
        }
        (0..length).map(|_| self.g2()).collect()
    }

    fn finish(self) -> Result<(), Error> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(Error::InvalidEncoding)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dusk_curves::bls12_381::BlsScalar;
    use sha2::{Digest, Sha256};

    fn g1(value: u64) -> G1Affine {
        G1Affine::from(G1Affine::generator() * BlsScalar::from(value))
    }

    fn g2(value: u64) -> G2Affine {
        G2Affine::from(G2Affine::generator() * BlsScalar::from(value))
    }

    fn proving_key() -> ProvingKey {
        ProvingKey {
            shape_digest: [7u8; 32],
            variables: 3,
            public_inputs: 1,
            domain_size: 2,
            alpha_g1: g1(2),
            beta_g1: g1(3),
            beta_g2: g2(3),
            delta_g1: g1(4),
            delta_g2: g2(4),
            a_query: vec![g1(5), g1(6), g1(7)],
            b_g1_query: vec![g1(8), g1(9), g1(10)],
            b_g2_query: vec![g2(8), g2(9), g2(10)],
            private_query: vec![g1(11)],
            h_query: vec![g1(12)],
        }
    }

    fn verifying_key() -> VerifyingKey {
        VerifyingKey {
            alpha_g1: g1(2),
            beta_g2: g2(3),
            gamma_g2: g2(4),
            delta_g2: g2(5),
            public_query: vec![g1(6), g1(7)],
        }
    }

    #[test]
    fn proving_key_encoding_round_trip_and_rejections() {
        let key = proving_key();
        let bytes = key.to_bytes();
        assert_eq!(ProvingKey::try_from_bytes(&bytes).unwrap(), key);
        assert_eq!(
            <[u8; 32]>::from(Sha256::digest(&bytes)),
            [
                76, 141, 15, 48, 163, 67, 42, 114, 118, 170, 46, 187, 134, 50, 115, 194, 126, 152,
                141, 27, 247, 185, 51, 183, 180, 20, 107, 211, 230, 247, 21, 139,
            ]
        );

        let mut trailing = bytes.clone();
        trailing.push(0);
        assert_eq!(
            ProvingKey::try_from_bytes(&trailing),
            Err(Error::InvalidEncoding)
        );
        assert_eq!(
            ProvingKey::try_from_bytes(&bytes[..bytes.len() - 1]),
            Err(Error::InvalidEncoding)
        );

        let mut wrong_version = bytes.clone();
        wrong_version[8] ^= 1;
        assert_eq!(
            ProvingKey::try_from_bytes(&wrong_version),
            Err(Error::InvalidEncoding)
        );

        let mut wrong_kind = bytes.clone();
        wrong_kind[9] = VERIFYING_KEY_KIND;
        assert_eq!(
            ProvingKey::try_from_bytes(&wrong_kind),
            Err(Error::InvalidEncoding)
        );

        let mut malformed_point = bytes.clone();
        malformed_point[66..66 + G1Affine::SIZE].fill(0);
        assert_eq!(
            ProvingKey::try_from_bytes(&malformed_point),
            Err(Error::InvalidEncoding)
        );

        let mut identity_parameter = bytes.clone();
        identity_parameter[66..66 + G1Affine::SIZE]
            .copy_from_slice(&G1Affine::identity().to_bytes());
        assert_eq!(
            ProvingKey::try_from_bytes(&identity_parameter),
            Err(Error::InvalidEncoding)
        );

        // The first vector length follows the fixed-size header and points.
        let mut length_overflow = bytes;
        length_overflow[402..410].fill(0xff);
        assert_eq!(
            ProvingKey::try_from_bytes(&length_overflow),
            Err(Error::InvalidEncoding)
        );
    }

    #[test]
    fn verifying_key_encoding_round_trip_and_rejections() {
        let key = verifying_key();
        let bytes = key.to_bytes();
        assert_eq!(VerifyingKey::try_from_bytes(&bytes).unwrap(), key);
        assert_eq!(
            <[u8; 32]>::from(Sha256::digest(&bytes)),
            [
                239, 42, 0, 51, 184, 105, 2, 88, 35, 59, 244, 251, 117, 4, 175, 230, 247, 214, 74,
                99, 102, 247, 184, 90, 177, 195, 215, 221, 105, 26, 187, 112,
            ]
        );

        let mut trailing = bytes.clone();
        trailing.push(0);
        assert_eq!(
            VerifyingKey::try_from_bytes(&trailing),
            Err(Error::InvalidEncoding)
        );

        let mut identity_parameter = bytes.clone();
        identity_parameter[10..10 + G1Affine::SIZE]
            .copy_from_slice(&G1Affine::identity().to_bytes());
        assert_eq!(
            VerifyingKey::try_from_bytes(&identity_parameter),
            Err(Error::InvalidEncoding)
        );

        let mut empty_query = key;
        empty_query.public_query.clear();
        assert_eq!(
            VerifyingKey::try_from_bytes(&empty_query.to_bytes()),
            Err(Error::InvalidEncoding)
        );
    }
}
