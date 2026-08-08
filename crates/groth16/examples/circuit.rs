// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_groth16::prelude::*;
use rand_core::OsRng;

#[derive(Default)]
struct ProductCircuit {
    left: BlsScalar,
    right: BlsScalar,
    result: BlsScalar,
}

impl Circuit for ProductCircuit {
    fn circuit<B: ComposerBackend>(&self, composer: &mut Composer<B>) -> Result<(), CircuitError> {
        let left = composer.append_witness(self.left);
        let right = composer.append_witness(self.right);
        let product = composer.gate_mul(Constraint::new().mult(1).a(left).b(right));
        let result = composer.append_public(self.result);
        composer.assert_equal(product, result);
        Ok(())
    }
}

fn main() {
    // This is a single-party trusted setup. Its randomness must be destroyed.
    let (prover, verifier) =
        Compiler::trusted_setup::<ProductCircuit, _>(&mut OsRng).expect("setup should succeed");
    let circuit = ProductCircuit {
        left: BlsScalar::from(6u64),
        right: BlsScalar::from(7u64),
        result: BlsScalar::from(42u64),
    };
    let (proof, public_inputs) = prover
        .prove(&mut OsRng, &circuit)
        .expect("proof generation should succeed");
    verifier
        .verify(&proof, &public_inputs)
        .expect("proof should verify");
}
