// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use dusk_zk_composer::prelude::*;

#[derive(Clone, Debug, Default)]
struct ExternalBackend {
    marker: u8,
    constraints: usize,
    last_wires: [Witness; Constraint::WITNESSES],
    last_q_m: BlsScalar,
    last_public_input: Option<BlsScalar>,
}

impl ComposerBackend for ExternalBackend {
    fn new() -> Self {
        Self::default()
    }

    fn constraints(&self) -> usize {
        self.constraints
    }

    fn append_constraint(&mut self, constraint: &Constraint) {
        self.constraints += 1;
        self.last_wires = constraint.wires();
        self.last_q_m = *constraint.q_m();
        self.last_public_input = constraint.public_input().copied();

        // Exercise every read-only selector accessor available to external
        // backend implementations.
        let _ = (
            constraint.q_l(),
            constraint.q_r(),
            constraint.q_o(),
            constraint.q_f(),
            constraint.q_c(),
            constraint.q_arith(),
            constraint.q_range(),
            constraint.q_logic(),
            constraint.q_fixed_group_add(),
            constraint.q_variable_group_add(),
        );
    }
}

#[test]
fn external_backend_can_inspect_emitted_constraints() {
    let backend = ExternalBackend {
        marker: 42,
        ..ExternalBackend::default()
    };
    let mut composer = Composer::with_backend(backend);
    assert_eq!(composer.backend().marker, 42);
    let left = composer.append_witness(BlsScalar::from(2u64));
    let right = composer.append_witness(BlsScalar::from(3u64));
    let output = composer.gate_mul(Constraint::new().mult(1).a(left).b(right));

    let public = BlsScalar::from(6u64);
    let public_witness = composer.append_public(public);
    composer.assert_equal(output, public_witness);

    let backend = composer.into_backend();
    assert_eq!(backend.last_wires[0], output);
    assert_eq!(backend.last_wires[1], public_witness);
    assert_eq!(backend.last_q_m, BlsScalar::zero());
    assert_eq!(backend.last_public_input, None);
}

#[test]
fn external_backend_observes_zero_public_inputs() {
    let mut composer = Composer::<ExternalBackend>::initialized();
    let public = composer.append_public(BlsScalar::zero());

    let backend = composer.into_backend();
    assert_eq!(backend.last_wires[0], public);
    assert_eq!(backend.last_public_input, Some(BlsScalar::zero()));
}
