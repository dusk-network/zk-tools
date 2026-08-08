// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use dusk_groth16::prelude::*;
use rand_core::OsRng;

#[derive(Default)]
struct BenchCircuit {
    value: BlsScalar,
}

impl Circuit for BenchCircuit {
    fn circuit<B: ComposerBackend>(&self, composer: &mut Composer<B>) -> Result<(), CircuitError> {
        let value = composer.append_witness(self.value);
        composer.component_range_bits::<16>(value);
        let public = composer.append_public(self.value);
        composer.assert_equal(value, public);
        Ok(())
    }
}

fn benchmark(c: &mut Criterion) {
    let (prover, verifier) =
        Compiler::trusted_setup::<BenchCircuit, _>(&mut OsRng).expect("setup should succeed");
    let circuit = BenchCircuit {
        value: BlsScalar::from(42u64),
    };
    let (proof, public_inputs) = prover
        .prove(&mut OsRng, &circuit)
        .expect("proof generation should succeed");

    c.bench_function("groth16 prove 16-bit range", |bencher| {
        bencher.iter(|| prover.prove(&mut OsRng, black_box(&circuit)))
    });
    c.bench_function("groth16 verify one public input", |bencher| {
        bencher.iter(|| verifier.verify(black_box(&proof), black_box(&public_inputs)))
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
