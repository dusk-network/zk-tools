// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use dusk_curves::bls12_381::BlsScalar;
use dusk_groth16::Compiler as GrothCompiler;
use dusk_jubjub::{GENERATOR_EXTENDED, JubJubAffine, JubJubScalar};
use dusk_plonk::prelude::*;
use dusk_poseidon::{decrypt, decrypt_gadget, encrypt};
use ff::Field;
use once_cell::sync::Lazy;
use rand::SeedableRng;
use rand::rngs::StdRng;

const MESSAGE_LEN: usize = 2;

static PUB_PARAMS: Lazy<PublicParameters> = Lazy::new(|| {
    let mut rng = StdRng::seed_from_u64(0xfab);

    const CAPACITY: usize = 11;
    PublicParameters::setup(1 << CAPACITY, &mut rng)
        .expect("Setup of public params should pass")
});
static LABEL: &[u8] = b"hash-gadget-tester";

#[derive(Debug)]
struct DecryptionCircuit {
    pub cipher: Vec<BlsScalar>,
    pub shared_secret: JubJubAffine,
    pub nonce: BlsScalar,
}

impl DecryptionCircuit {
    pub fn random(rng: &mut StdRng) -> Self {
        let mut message = [BlsScalar::zero(); MESSAGE_LEN];
        message
            .iter_mut()
            .for_each(|s| *s = BlsScalar::random(&mut *rng));
        let shared_secret =
            GENERATOR_EXTENDED * JubJubScalar::random(&mut *rng);
        let shared_secret = shared_secret.into();
        let nonce = BlsScalar::random(&mut *rng);
        let cipher = encrypt(message, &shared_secret, &nonce)
            .expect("encryption should not fail");

        Self {
            cipher,
            shared_secret,
            nonce,
        }
    }
}

impl Default for DecryptionCircuit {
    fn default() -> Self {
        let message = [BlsScalar::zero(); MESSAGE_LEN];
        let mut cipher = message.to_vec();
        cipher.push(BlsScalar::zero());
        let shared_secret = JubJubAffine::identity();
        let nonce = BlsScalar::zero();

        Self {
            cipher,
            shared_secret,
            nonce,
        }
    }
}

impl Circuit for DecryptionCircuit {
    fn circuit<B: ComposerBackend>(
        &self,
        composer: &mut Composer<B>,
    ) -> Result<(), CircuitError> {
        // append all variables to the circuit
        let mut cipher_wit = Vec::with_capacity(MESSAGE_LEN + 1);
        self.cipher
            .iter()
            .for_each(|c| cipher_wit.push(composer.append_witness(*c)));
        let secret_wit = composer.append_point(self.shared_secret)?;
        let nonce_wit = composer.append_witness(self.nonce);

        // decrypt the cipher with the gadget
        let _cipher_result =
            decrypt_gadget(composer, &cipher_wit, &secret_wit, &nonce_wit)
                .expect("decryption should pass");

        Ok(())
    }
}

fn bench_decryption(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(0x42424242);

    let (plonk_prover, plonk_verifier) =
        Compiler::compile::<DecryptionCircuit>(&PUB_PARAMS, LABEL)
            .expect("compilation should pass");
    let (groth_prover, groth_verifier) =
        GrothCompiler::trusted_setup::<DecryptionCircuit, _>(&mut rng)
            .expect("Groth16 setup should succeed");

    let circuit: DecryptionCircuit = DecryptionCircuit::random(&mut rng);
    let public_inputs = Vec::new();
    let (mut plonk_proof, _) = plonk_prover
        .prove(&mut rng, &circuit)
        .expect("PLONK proof generation should succeed");
    let (mut groth_proof, groth_public_inputs) = groth_prover
        .prove(&mut rng, &circuit)
        .expect("Groth16 proof generation should succeed");
    assert_eq!(groth_public_inputs, public_inputs);

    // Benchmark native cipher decryption
    c.bench_function("decrypt 2 BlsScalar", |b| {
        b.iter(|| {
            _ = decrypt(
                black_box(&circuit.cipher),
                black_box(&circuit.shared_secret),
                black_box(&circuit.nonce),
            );
        })
    });

    // Benchmark PLONK proof creation
    c.bench_function("decrypt 2 BlsScalar PLONK proof generation", |b| {
        b.iter(|| {
            (plonk_proof, _) = plonk_prover
                .prove(&mut rng, black_box(&circuit))
                .expect("PLONK proof generation should succeed");
        })
    });

    // Benchmark PLONK proof verification
    c.bench_function("decrypt 2 BlsScalar PLONK proof verification", |b| {
        b.iter(|| {
            plonk_verifier
                .verify(black_box(&plonk_proof), &public_inputs)
                .expect("PLONK proof verification should succeed");
        })
    });

    // Benchmark Groth16 proof creation
    c.bench_function("decrypt 2 BlsScalar Groth16 proof generation", |b| {
        b.iter(|| {
            (groth_proof, _) = groth_prover
                .prove(&mut rng, black_box(&circuit))
                .expect("Groth16 proof generation should succeed");
        })
    });

    // Benchmark Groth16 proof verification
    c.bench_function("decrypt 2 BlsScalar Groth16 proof verification", |b| {
        b.iter(|| {
            groth_verifier
                .verify(black_box(&groth_proof), &public_inputs)
                .expect("Groth16 proof verification should succeed");
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_decryption
}
criterion_main!(benches);
