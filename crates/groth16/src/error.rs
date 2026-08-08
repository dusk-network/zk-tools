// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Groth16 errors.

use core::fmt;

/// Errors returned by setup, proving, verification, or decoding.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// Circuit construction failed.
    Circuit,
    /// The scalar field cannot provide the requested FFT domain.
    InvalidEvaluationDomain,
    /// The proving circuit differs from the setup circuit.
    InvalidCircuitShape,
    /// The supplied witness does not satisfy the circuit.
    CircuitUnsatisfied,
    /// Public-input length differs from the verification key.
    InvalidPublicInputCount {
        /// Expected number of public inputs.
        expected: usize,
        /// Supplied number of public inputs.
        provided: usize,
    },
    /// A query and scalar vector have inconsistent lengths.
    InvalidQueryLength,
    /// The proof equation does not hold.
    ProofVerification,
    /// Encoded data is malformed or truncated.
    InvalidEncoding,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Circuit => formatter.write_str("circuit construction failed"),
            Self::InvalidEvaluationDomain => formatter.write_str("invalid FFT evaluation domain"),
            Self::InvalidCircuitShape => {
                formatter.write_str("circuit shape differs from the setup circuit")
            }
            Self::CircuitUnsatisfied => formatter.write_str("circuit assignment is unsatisfied"),
            Self::InvalidPublicInputCount { expected, provided } => write!(
                formatter,
                "invalid public-input count: expected {expected}, provided {provided}"
            ),
            Self::InvalidQueryLength => formatter.write_str("query and scalar lengths differ"),
            Self::ProofVerification => formatter.write_str("Groth16 proof verification failed"),
            Self::InvalidEncoding => formatter.write_str("invalid encoded data"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[cfg(feature = "alloc")]
impl From<dusk_zk_composer::Error> for Error {
    fn from(_: dusk_zk_composer::Error) -> Self {
        Self::Circuit
    }
}
