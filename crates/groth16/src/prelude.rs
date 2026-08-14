// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Common Groth16 and circuit-construction imports.

pub use dusk_curves::bls12_381::BlsScalar;
#[cfg(feature = "alloc")]
pub use dusk_zk_composer::{
    Circuit, Composer, ComposerBackend, Constraint, Error as CircuitError, R1cs, Witness,
};

pub use crate::Error;
#[cfg(feature = "alloc")]
pub use crate::{
    Compiler, EIP2537_PROOF_SIZE, Eip2537Proof, PreparedVerifyingKey, Proof, Prover, ProvingKey,
    SolidityError, Verifier, VerifyingKey, encode_eip2537_scalar,
};
