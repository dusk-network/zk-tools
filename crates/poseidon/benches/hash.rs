// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use dusk_groth16::Compiler as GrothCompiler;
use dusk_plonk::prelude::*;
use dusk_poseidon::{Domain, HADES_WIDTH, Hash, HashGadget};
use ff::Field;
use rand::SeedableRng;
use rand::rngs::StdRng;

const CAPACITY: usize = 11;

#[derive(Default)]
struct SpongeCircuit {
    message: [BlsScalar; HADES_WIDTH - 1],
    output: BlsScalar,
}

impl SpongeCircuit {
    pub fn new(
        message: [BlsScalar; HADES_WIDTH - 1],
        output: BlsScalar,
    ) -> Self {
        SpongeCircuit { message, output }
    }
}

impl Circuit for SpongeCircuit {
    fn circuit<B: ComposerBackend>(
        &self,
        composer: &mut Composer<B>,
    ) -> Result<(), CircuitError> {
        let mut w_message = [Composer::<B>::ZERO; HADES_WIDTH - 1];
        w_message
            .iter_mut()
            .zip(self.message)
            .for_each(|(witness, scalar)| {
                *witness = composer.append_witness(scalar);
            });

        let output_witness =
            HashGadget::digest(composer, Domain::Merkle4, &w_message);
        composer.assert_equal_constant(output_witness[0], 0, Some(self.output));

        Ok(())
    }
}

// Benchmark for running sponge on 5 BlsScalar, one permutation
fn bench_sponge(c: &mut Criterion) {
    // Prepare benchmarks and initialize variables
    let label = b"sponge benchmark";
    let mut rng = StdRng::seed_from_u64(0xc10d);
    let pp = PublicParameters::setup(1 << CAPACITY, &mut rng).unwrap();
    let (plonk_prover, plonk_verifier) =
        Compiler::compile::<SpongeCircuit>(&pp, label)
            .expect("Circuit should compile successfully");
    let (groth_prover, groth_verifier) =
        GrothCompiler::trusted_setup::<SpongeCircuit, _>(&mut rng)
            .expect("Groth16 setup should succeed");
    let message = [
        BlsScalar::random(&mut rng),
        BlsScalar::random(&mut rng),
        BlsScalar::random(&mut rng),
        BlsScalar::random(&mut rng),
    ];
    let public_inputs = Hash::digest(Domain::Merkle4, &message);
    let circuit = SpongeCircuit::new(message, public_inputs[0]);
    let (mut plonk_proof, _) = plonk_prover
        .prove(&mut rng, &circuit)
        .expect("PLONK proof generation should succeed");
    let (mut groth_proof, groth_public_inputs) = groth_prover
        .prove(&mut rng, &circuit)
        .expect("Groth16 proof generation should succeed");
    assert_eq!(groth_public_inputs, public_inputs);

    // Benchmark sponge native
    c.bench_function("hash 4 BlsScalar", |b| {
        b.iter(|| {
            let _ = Hash::digest(Domain::Merkle4, black_box(&circuit.message));
        })
    });

    // Benchmark PLONK proof creation
    c.bench_function("hash 4 BlsScalar PLONK proof generation", |b| {
        b.iter(|| {
            (plonk_proof, _) = plonk_prover
                .prove(&mut rng, black_box(&circuit))
                .expect("PLONK proof generation should succeed");
        })
    });

    // Benchmark PLONK proof verification
    c.bench_function("hash 4 BlsScalar PLONK proof verification", |b| {
        b.iter(|| {
            plonk_verifier
                .verify(black_box(&plonk_proof), &public_inputs)
                .expect("PLONK proof verification should succeed");
        })
    });

    // Benchmark Groth16 proof creation
    c.bench_function("hash 4 BlsScalar Groth16 proof generation", |b| {
        b.iter(|| {
            (groth_proof, _) = groth_prover
                .prove(&mut rng, black_box(&circuit))
                .expect("Groth16 proof generation should succeed");
        })
    });

    // Benchmark Groth16 proof verification
    c.bench_function("hash 4 BlsScalar Groth16 proof verification", |b| {
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
    targets = bench_sponge
}
criterion_main!(benches);
