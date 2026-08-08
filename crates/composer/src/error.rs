// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Circuit-construction errors.

/// Errors produced while constructing or decoding a circuit.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The built circuit has a different gate count from its description.
    InvalidCircuitSize(usize, usize),
    /// A BLS scalar encoding is malformed.
    BlsScalarMalformed,
    /// A JubJub scalar cannot be represented canonically.
    JubJubScalarMalformed,
    /// A fixed-base generator is not a nonidentity prime-order point.
    JubJubGeneratorNotPrimeOrder,
    /// A JubJub point is not an on-curve prime-order subgroup member.
    JubJubPointNotTorsionFree,
    /// A JubJub extended point has no affine image because `Z` is zero.
    JubJubPointDegenerate,
    /// A width-two non-adjacent-form digit is outside `[-1, 0, 1]`.
    UnsupportedWNAF2k,
    /// A compressed circuit description is malformed.
    InvalidCompressedCircuit,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidCircuitSize(description, circuit) => write!(
                f,
                "circuit gate count differs from its description: description = {description}, circuit = {circuit}"
            ),
            Self::BlsScalarMalformed => write!(f, "BLS scalar bytes malformed"),
            Self::JubJubScalarMalformed => {
                write!(f, "JubJub scalar bytes malformed")
            }
            Self::JubJubGeneratorNotPrimeOrder => {
                write!(f, "JubJub generator is not a prime-order point")
            }
            Self::JubJubPointNotTorsionFree => {
                write!(f, "JubJub point is not in the prime-order subgroup")
            }
            Self::JubJubPointDegenerate => {
                write!(f, "JubJub point has a zero Z coordinate")
            }
            Self::UnsupportedWNAF2k => {
                write!(f, "WNAF2k digit is outside [-1, 0, 1]")
            }
            Self::InvalidCompressedCircuit => {
                write!(f, "invalid compressed circuit")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
