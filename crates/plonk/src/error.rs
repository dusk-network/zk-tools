// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! A collection of all possible errors encountered in PLONK.

use dusk_bytes::Error as DuskBytesError;
#[cfg(feature = "alloc")]
use dusk_zk_composer::Error as CircuitError;

/// Defines all possible errors that can be encountered in PLONK.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Error {
    // FFT errors
    /// This error occurs when an error triggers on any of the fft module
    /// functions.
    InvalidEvalDomainSize {
        /// Log size of the group
        log_size_of_group: u32,
        /// Two adacity generated
        adacity: u32,
    },

    // Prover/Verifier errors
    /// This error occurs when a proof verification fails.
    ProofVerificationError,
    /// This error occurs when the circuit is not provided with all of the
    /// required inputs.
    CircuitInputsNotFound,
    /// This error occurs when we want to verify a Proof but the pi_constructor
    /// attribute is uninitialized.
    UninitializedPIGenerator,
    /// PublicInput serialization error
    InvalidPublicInputBytes,
    /// This error occurs when the Prover structure already contains a
    /// preprocessed circuit inside, but you call preprocess again.
    CircuitAlreadyPreprocessed,
    /// This error occurs when proof creation fails because the constraint
    /// system is not satisfied: the witness assignment or the appended
    /// constants do not match the compiled circuit description.
    CircuitUnsatisfied,

    // Preprocessing errors
    /// This error occurs when an error triggers during the preprocessing
    /// stage.
    MismatchedPolyLen,

    // KZG10 errors
    /// This error occurs when the user tries to create PublicParameters
    /// and supplies the max degree as zero.
    DegreeIsZero,
    /// This error occurs when the user tries to trim PublicParameters
    /// to a degree that is larger than the maximum degree.
    TruncatedDegreeTooLarge,
    /// This error occurs when the user tries to trim PublicParameters
    /// down to a degree that is zero.
    TruncatedDegreeIsZero,
    /// This error occurs when the user tries to commit to a polynomial whose
    /// degree is larger than the supported degree for that proving key.
    PolynomialDegreeTooLarge,
    /// This error occurs when the user tries to commit to a polynomial whose
    /// degree is zero.
    PolynomialDegreeIsZero,
    /// This error occurs when the pairing check fails at being equal to the
    /// Identity point.
    PairingCheckFailure,

    // Serialization errors
    /// Dusk-bytes serialization error
    BytesError(DuskBytesError),
    /// This error occurs when there are not enough bytes to read out of a
    /// slice during deserialization.
    NotEnoughBytes,
    /// This error occurs when a malformed point is decoded from a byte array.
    PointMalformed,
    /// The provided public inputs doesn't match the circuit definition
    PublicInputNotFound {
        /// Expected public input wasn't found
        index: usize,
    },
    /// The provided public inputs length doesn't match the processed verifier
    InconsistentPublicInputsLen {
        /// Expected value
        expected: usize,
        /// Provided value
        provided: usize,
    },
    /// Circuit construction or compressed-description error.
    #[cfg(feature = "alloc")]
    Circuit(CircuitError),
    /// Legacy proving was requested but this build disables legacy proving.
    LegacyProvingDisabled,
    /// The requested proving version is no longer supported.
    UnsupportedProvingVersion,
}

#[cfg(feature = "std")]
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidEvalDomainSize {
                log_size_of_group,
                adacity,
            } => write!(
                f,
                "Log-size of the EvaluationDomain group > TWO_ADACITY\
            Size: {:?} > TWO_ADACITY = {:?}",
                log_size_of_group, adacity
            ),
            Self::ProofVerificationError => {
                write!(f, "proof verification failed")
            }
            Self::CircuitInputsNotFound => {
                write!(f, "circuit inputs not found")
            }
            Self::UninitializedPIGenerator => {
                write!(f, "PI generator uninitialized")
            }
            Self::InvalidPublicInputBytes => {
                write!(f, "invalid public input bytes")
            }
            Self::MismatchedPolyLen => {
                write!(f, "the length of the wires is not the same")
            }
            Self::CircuitAlreadyPreprocessed => {
                write!(f, "circuit has already been preprocessed")
            }
            Self::CircuitUnsatisfied => write!(
                f,
                "the circuit is not satisfied: the witness assignment or the \
                 appended constants do not match the compiled circuit \
                 description"
            ),
            Self::DegreeIsZero => {
                write!(f, "cannot create PublicParameters with max degree 0")
            }
            Self::TruncatedDegreeTooLarge => {
                write!(f, "cannot trim more than the maximum degree")
            }
            Self::TruncatedDegreeIsZero => write!(
                f,
                "cannot trim PublicParameters to a maximum size of zero"
            ),
            Self::PolynomialDegreeTooLarge => write!(
                f,
                "proving key is not large enough to commit to said polynomial"
            ),
            Self::PolynomialDegreeIsZero => {
                write!(f, "cannot commit to polynomial of zero degree")
            }
            Self::PairingCheckFailure => write!(f, "pairing check failed"),
            Self::NotEnoughBytes => write!(f, "not enough bytes left to read"),
            Self::PointMalformed => write!(f, "BLS point bytes malformed"),
            Self::BytesError(err) => write!(f, "{:?}", err),
            Self::PublicInputNotFound { index } => write!(
                f,
                "The public input of index {} is defined in the circuit description, but wasn't declared in the prove instance",
                index
            ),
            Self::InconsistentPublicInputsLen { expected, provided } => write!(
                f,
                "The provided public inputs set of length {} doesn't match the processed verifier: {}",
                provided, expected
            ),
            #[cfg(feature = "alloc")]
            Self::Circuit(err) => write!(f, "circuit error: {err}"),
            Self::LegacyProvingDisabled => {
                write!(f, "legacy proving is disabled in this build")
            }
            Self::UnsupportedProvingVersion => {
                write!(f, "requested proving version is unsupported")
            }
        }
    }
}

impl From<DuskBytesError> for Error {
    fn from(bytes_err: DuskBytesError) -> Self {
        Self::BytesError(bytes_err)
    }
}

#[cfg(feature = "alloc")]
impl From<CircuitError> for Error {
    fn from(error: CircuitError) -> Self {
        Self::Circuit(error)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[cfg(all(test, feature = "std"))]
mod tests {
    use dusk_bytes::{DeserializableSlice, Serializable};
    use dusk_curves::bls12_381::BlsScalar;

    use super::*;

    #[test]
    fn display_arms_are_exercised() {
        let bogus = [0u8; BlsScalar::SIZE - 1];
        let dusk_err = BlsScalar::from_slice(&bogus)
            .expect_err("decoding from a short slice must fail");
        let bytes_error: Error = dusk_err.into();
        assert!(matches!(bytes_error, Error::BytesError(_)));

        // Format each variant at least once so the `Display` impl gets covered.
        let all_errors: [Error; 19] = [
            Error::InvalidEvalDomainSize {
                log_size_of_group: 32,
                adacity: 28,
            },
            Error::ProofVerificationError,
            Error::CircuitInputsNotFound,
            Error::UninitializedPIGenerator,
            Error::InvalidPublicInputBytes,
            Error::CircuitAlreadyPreprocessed,
            Error::Circuit(CircuitError::InvalidCircuitSize(1, 2)),
            Error::CircuitUnsatisfied,
            Error::MismatchedPolyLen,
            Error::DegreeIsZero,
            Error::TruncatedDegreeTooLarge,
            Error::TruncatedDegreeIsZero,
            Error::PolynomialDegreeTooLarge,
            Error::PolynomialDegreeIsZero,
            Error::PairingCheckFailure,
            Error::NotEnoughBytes,
            Error::PointMalformed,
            Error::LegacyProvingDisabled,
            Error::UnsupportedProvingVersion,
        ];

        for e in all_errors {
            let s = e.to_string();
            assert!(!s.is_empty());
        }

        // Variants with payloads.
        assert!(
            Error::PublicInputNotFound { index: 7 }
                .to_string()
                .contains("index")
        );
        assert!(
            Error::InconsistentPublicInputsLen {
                expected: 1,
                provided: 2
            }
            .to_string()
            .contains("provided")
        );

        assert!(!bytes_error.to_string().is_empty());
        let _as_std_error: &dyn std::error::Error =
            &Error::ProofVerificationError;
    }
}
