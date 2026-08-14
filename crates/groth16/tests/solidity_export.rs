// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

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
fn committed_solidity_verifier_is_deterministically_generated() {
    let mut rng = ChaCha20Rng::from_seed([21u8; 32]);
    let (_, verifier) = Compiler::trusted_setup::<SquareCircuit, _>(&mut rng).unwrap();
    let generated = verifier.key().solidity_verifier("SquareVerifier").unwrap();
    assert_eq!(
        generated,
        include_str!("solidity/test/fixtures/SquareVerifier.sol")
    );
}
