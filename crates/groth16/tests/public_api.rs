// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_bytes::Serializable;
use dusk_groth16::prelude::*;
use rand_chacha::{ChaCha20Rng, rand_core::SeedableRng};

#[derive(Default)]
struct SquareCircuit {
    secret: BlsScalar,
    square: BlsScalar,
}

impl Circuit for SquareCircuit {
    fn circuit<B: ComposerBackend>(&self, composer: &mut Composer<B>) -> Result<(), CircuitError> {
        let secret = composer.append_witness(self.secret);
        let square = composer.gate_mul(Constraint::new().mult(1).a(secret).b(secret));
        let public = composer.append_public(self.square);
        composer.assert_equal(square, public);
        Ok(())
    }
}

#[test]
fn exported_keys_and_proof_round_trip() {
    let mut rng = ChaCha20Rng::from_seed([21u8; 32]);
    let (prover, verifier) = Compiler::trusted_setup::<SquareCircuit, _>(&mut rng).unwrap();
    let prover =
        Prover::from_key(ProvingKey::try_from_bytes(&prover.into_key().to_bytes()).unwrap());
    let verifier =
        Verifier::from_key(VerifyingKey::try_from_bytes(&verifier.into_key().to_bytes()).unwrap());
    let circuit = SquareCircuit {
        secret: BlsScalar::from(9u64),
        square: BlsScalar::from(81u64),
    };
    let (proof, public_inputs) = prover.prove(&mut rng, &circuit).unwrap();
    let decoded = Proof::from_bytes(&proof.to_bytes()).unwrap();
    verifier.verify(&decoded, &public_inputs).unwrap();
}
