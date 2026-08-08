// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

// to be able to use this module, the "poseidon" feature needs to be in scope

use criterion::{Criterion, criterion_group, criterion_main};
use dusk_groth16::Compiler as GrothCompiler;
use dusk_plonk::prelude::*;
use dusk_poseidon::{Domain, Hash};
use poseidon_merkle::zk::opening_gadget;
use poseidon_merkle::{Item, Opening, Tree};
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

// set max circuit size to 2^16 gates
const CAPACITY: usize = 16;

// set height of the poseidon merkle tree
const HEIGHT: usize = 17;

type PoseidonTree = Tree<(), HEIGHT>;
type PoseidonItem = Item<()>;

// Create a circuit for the opening
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
struct OpeningCircuit {
    opening: Opening<(), HEIGHT>,
    leaf: PoseidonItem,
}

impl Default for OpeningCircuit {
    fn default() -> Self {
        let mut tree = PoseidonTree::new();
        let empty_item = PoseidonItem {
            hash: BlsScalar::zero(),
            data: (),
        };
        tree.insert(0, empty_item);
        let opening = tree.opening(0).expect("There is a leaf at position 0");
        Self {
            opening,
            leaf: empty_item,
        }
    }
}

impl OpeningCircuit {
    /// Create a new OpeningCircuit
    pub fn new(opening: Opening<(), HEIGHT>, leaf: PoseidonItem) -> Self {
        Self { opening, leaf }
    }
}

impl Circuit for OpeningCircuit {
    fn circuit<B: ComposerBackend>(
        &self,
        composer: &mut Composer<B>,
    ) -> Result<(), CircuitError> {
        // append the leaf and opening gadget to the circuit
        let leaf = composer.append_witness(self.leaf.hash);
        let computed_root = opening_gadget(composer, &self.opening, leaf);

        // append the public root as public input to the circuit
        // and ensure it is equal to the computed root
        let constraint = Constraint::new()
            .left(-BlsScalar::one())
            .a(computed_root)
            .public(self.opening.root().hash);
        composer.append_gate(constraint);

        Ok(())
    }
}

fn bench_zk(c: &mut Criterion) {
    // create the prover and verifier circuit descriptions
    let label = b"merkle opening";
    let setup_rng = &mut StdRng::seed_from_u64(0xdea1);
    let pp = PublicParameters::setup(1 << CAPACITY, setup_rng).unwrap();
    let (plonk_prover, plonk_verifier) =
        Compiler::compile::<OpeningCircuit>(&pp, label)
            .expect("Circuit should compile successfully");
    let (groth_prover, groth_verifier) =
        GrothCompiler::trusted_setup::<OpeningCircuit, _>(setup_rng)
            .expect("Groth16 setup should succeed");

    // create a new tree and insert 100 leaves at random positions
    let tree = &mut PoseidonTree::new();
    let rng = &mut rand::rngs::StdRng::seed_from_u64(0xbeef);
    for _ in 0..100 {
        let pos = rng.next_u64() % u32::MAX as u64;
        let leaf = PoseidonItem {
            hash: Hash::digest(Domain::Other, &[pos.into()])[0],
            data: (),
        };
        tree.insert(pos, leaf);
    }

    // insert new leaf in the tree at random position to create opening
    let pos = rng.next_u64() % u32::MAX as u64;
    let leaf = PoseidonItem {
        hash: Hash::digest(Domain::Other, &[pos.into()])[0],
        data: (),
    };
    tree.insert(pos, leaf);

    // create a new opening circuit for the last leaf we inserted
    let opening = tree.opening(pos).unwrap();
    // sanity check
    assert!(opening.verify(leaf));
    let circuit = OpeningCircuit::new(opening, leaf);
    let public_inputs = [opening.root().hash];

    let (mut plonk_proof, _) = plonk_prover
        .prove(rng, &circuit)
        .expect("PLONK proof generation should succeed");
    let (mut groth_proof, groth_public_inputs) = groth_prover
        .prove(rng, &circuit)
        .expect("Groth16 proof generation should succeed");
    assert_eq!(groth_public_inputs, public_inputs);

    c.bench_function("opening PLONK proof generation", |b| {
        b.iter(|| {
            (plonk_proof, _) = plonk_prover
                .prove(rng, &circuit)
                .expect("PLONK proof generation should succeed");
        })
    });
    c.bench_function("opening PLONK proof verification", |b| {
        b.iter(|| {
            plonk_verifier
                .verify(&plonk_proof, &public_inputs)
                .expect("PLONK proof verification should succeed");
        })
    });
    c.bench_function("opening Groth16 proof generation", |b| {
        b.iter(|| {
            (groth_proof, _) = groth_prover
                .prove(rng, &circuit)
                .expect("Groth16 proof generation should succeed");
        })
    });
    c.bench_function("opening Groth16 proof verification", |b| {
        b.iter(|| {
            groth_verifier
                .verify(&groth_proof, &public_inputs)
                .expect("Groth16 proof verification should succeed");
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_zk
}
criterion_main!(benches);
