// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

mod common;

use common::prove_and_verify;
use dusk_plonk::prelude::*;
use dusk_poseidon::{Domain, Hash};
use ff::Field;
use poseidon_merkle::zk::opening_gadget;
use poseidon_merkle::{Item, Opening, Tree};
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

// set max circuit size to 2^15 gates
const CAPACITY: usize = 15;

// set height of the poseidon merkle tree
const HEIGHT: usize = 17;

type PoseidonItem = Item<()>;

// Create a circuit for the opening
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
struct OpeningCircuit {
    opening: Opening<(), HEIGHT>,
    leaf: PoseidonItem,
}

impl Default for OpeningCircuit {
    fn default() -> Self {
        let empty = Item {
            hash: BlsScalar::zero(),
            data: (),
        };
        let mut tree = Tree::new();
        tree.insert(0, empty);
        let opening = tree.opening(0).expect("There is a leaf at position 0");
        Self {
            opening,
            leaf: empty,
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

#[test]
fn opening() {
    let label = b"merkle opening";
    let mut rng = StdRng::seed_from_u64(0xdea1);
    let pp = PublicParameters::setup(1 << CAPACITY, &mut rng).unwrap();

    let mut tree = Tree::new();
    let mut leaf = PoseidonItem::new(BlsScalar::zero(), ());
    let mut position = 0;
    for _ in 0..100 {
        let hash =
            Hash::digest(Domain::Other, &[BlsScalar::random(&mut rng)])[0];
        position = rng.next_u64() % tree.capacity();
        leaf = PoseidonItem::new(hash, ());
        tree.insert(position, leaf);
    }
    let opening = tree.opening(position).unwrap();
    assert!(opening.verify(leaf));

    let circuit = OpeningCircuit::new(opening, leaf);

    let public_inputs = [opening.root().hash];
    prove_and_verify(&mut rng, &pp, label, &circuit, &public_inputs);
}
