// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_plonk::prelude::{
    BlsScalar, Circuit, Compiler, Composer, Constraint, Error as PlonkError, PublicParameters,
};
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

const TRANSCRIPT_LABEL: &[u8] = b"plonkwasm-test-v1";
const TEST_CIRCUIT_CAPACITY: usize = 1 << 8;

#[derive(Debug, Clone, Copy, Default)]
struct TestCircuit {
    left: BlsScalar,
    right: BlsScalar,
    product: BlsScalar,
}

impl TestCircuit {
    fn new(left: u64, right: u64) -> Self {
        let left = BlsScalar::from(left);
        let right = BlsScalar::from(right);
        let product = left * right;

        Self {
            left,
            right,
            product,
        }
    }
}

impl Circuit for TestCircuit {
    fn circuit(&self, composer: &mut Composer) -> Result<(), PlonkError> {
        let left = composer.append_witness(self.left);
        let right = composer.append_witness(self.right);

        let constraint = Constraint::new().mult(1).a(left).b(right);
        let computed = composer.gate_mul(constraint);
        let public_product = composer.append_public(self.product);
        composer.assert_equal(computed, public_product);

        Ok(())
    }
}

#[test]
fn prove_returns_proof_that_verifies() {
    let mut rng = ChaCha20Rng::from_seed([7; 32]);
    let pp = PublicParameters::setup(TEST_CIRCUIT_CAPACITY, &mut rng).unwrap();
    let (prover, verifier) = Compiler::compile::<TestCircuit>(&pp, TRANSCRIPT_LABEL).unwrap();

    let output = plonkwasm::prove(&prover.to_bytes(), [9; 32], &TestCircuit::new(13, 17)).unwrap();
    let verifier_bytes = verifier.to_bytes();
    plonkwasm::verify(&verifier_bytes, &output.proof, &output.public_inputs).unwrap();
}

#[test]
fn verify_rejects_invalid_public_inputs() {
    let mut rng = ChaCha20Rng::from_seed([7; 32]);
    let pp = PublicParameters::setup(TEST_CIRCUIT_CAPACITY, &mut rng).unwrap();
    let (prover, verifier) = Compiler::compile::<TestCircuit>(&pp, TRANSCRIPT_LABEL).unwrap();

    let output = plonkwasm::prove(&prover.to_bytes(), [9; 32], &TestCircuit::new(13, 17)).unwrap();
    let wrong_public_inputs = BlsScalar::from(222).to_bytes().to_vec();

    assert!(plonkwasm::verify(&verifier.to_bytes(), &output.proof, &wrong_public_inputs).is_err());
}
