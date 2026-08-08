// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Common imports for defining proof-system-neutral circuits.

pub use dusk_curves::bls12_381::BlsScalar;
pub use dusk_jubjub::{JubJubAffine, JubJubExtended, JubJubScalar};

#[cfg(feature = "alloc")]
pub use crate::{
    Circuit, Composer, ComposerBackend, Constraint, TorsionFreeWitnessPoint,
    Witness, WitnessPoint,
};
#[cfg(all(feature = "alloc", feature = "plonkish"))]
pub use crate::{CircuitShape, Gate, Plonkish};
pub use crate::{Error, Error as CircuitError};
#[cfg(all(feature = "alloc", feature = "r1cs"))]
pub use crate::{
    LinearCombination, R1cs, R1csAssignment, R1csCircuit, R1csConstraint,
    R1csShape, Variable,
};
