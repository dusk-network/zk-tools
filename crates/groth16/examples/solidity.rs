// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Generate deterministic Solidity integration-test fixtures.
//!
//! The fixed RNG seed makes these artifacts reproducible and unsuitable for
//! production trusted setup. Applications should export keys created with
//! securely generated randomness instead.

use std::{env, error::Error, fs, path::Path};

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

fn main() -> Result<(), Box<dyn Error>> {
    let output = env::args()
        .nth(1)
        .ok_or("usage: solidity <output-directory>")?;
    let output = Path::new(&output);
    fs::create_dir_all(output)?;

    let mut rng = ChaCha20Rng::from_seed([21u8; 32]);
    let (prover, verifier) = Compiler::trusted_setup::<SquareCircuit, _>(&mut rng)?;
    let circuit = SquareCircuit {
        secret: BlsScalar::from(9u64),
        square: BlsScalar::from(81u64),
    };
    let (proof, public_inputs) = prover.prove(&mut rng, &circuit)?;

    fs::write(
        output.join("SquareVerifier.sol"),
        verifier.key().solidity_verifier("SquareVerifier")?,
    )?;
    fs::write(output.join("verifying-key.bin"), verifier.key().to_bytes())?;
    fs::write(output.join("proof.bin"), proof.to_bytes())?;
    fs::write(
        output.join("proof.eip2537.bin"),
        proof.to_eip2537().as_bytes(),
    )?;
    fs::write(
        output.join("public-input.bin"),
        encode_eip2537_scalar(&public_inputs[0]),
    )?;

    verifier.verify(&proof, &public_inputs)?;
    Ok(())
}
