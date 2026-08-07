// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use dusk_curves::bls12_381::BlsScalar;
use dusk_jubjub::{
    GENERATOR_EXTENDED, JubJubAffine, JubJubExtended, JubJubScalar,
};
use sha2::{Digest, Sha256};

use super::{
    Circuit, Composer, ComposerBackend, Constraint, Error, Plonkish,
    TorsionFreeWitnessPoint,
};

#[derive(Default)]
struct ShapeCircuit {
    public: BlsScalar,
    left: BlsScalar,
    right: BlsScalar,
}

impl Circuit for ShapeCircuit {
    fn circuit<B: ComposerBackend>(
        &self,
        composer: &mut Composer<B>,
    ) -> Result<(), Error> {
        composer.append_public(self.public);
        let left = composer.append_witness(self.left);
        let right = composer.append_witness(self.right);
        composer.gate_mul(Constraint::new().mult(1).a(left).b(right));
        Ok(())
    }
}

fn compose<C: Circuit>(circuit: &C) -> Composer<Plonkish> {
    let mut composer = Composer::<Plonkish>::initialized();
    circuit.circuit(&mut composer).expect("circuit must build");
    composer
}

#[derive(Clone, Debug, Default)]
struct RecordingBackend {
    constraints: usize,
    initialized: bool,
}

impl ComposerBackend for RecordingBackend {
    fn new() -> Self {
        Self::default()
    }

    fn constraints(&self) -> usize {
        self.constraints
    }

    fn append_constraint(&mut self, _constraint: &Constraint) {
        self.constraints += 1;
    }

    fn initialize(composer: &mut Composer<Self>) {
        composer.backend.initialized = true;
    }
}

#[test]
fn generic_composer_dispatches_to_the_selected_backend() {
    let mut composer = Composer::<RecordingBackend>::initialized();
    assert_eq!(composer.constraints(), 2);

    let left = composer.append_witness(BlsScalar::from(2u64));
    let right = composer.append_witness(BlsScalar::from(3u64));
    composer.gate_mul(Constraint::new().mult(1).a(left).b(right));

    let constraints = composer.constraints();
    let backend = composer.into_backend();
    assert!(backend.initialized);
    assert_eq!(backend.constraints, constraints);
}

#[test]
fn shape_excludes_assignments_but_tracks_public_rows() {
    let first = compose(&ShapeCircuit {
        public: BlsScalar::zero(),
        left: BlsScalar::from(2u64),
        right: BlsScalar::from(3u64),
    });
    let second = compose(&ShapeCircuit {
        public: BlsScalar::from(9u64),
        left: BlsScalar::from(11u64),
        right: BlsScalar::from(13u64),
    });

    assert_eq!(first.shape(), second.shape());
    assert_eq!(first.public_input_indexes(), second.public_input_indexes());
    assert_ne!(first.public_inputs(), second.public_inputs());
    assert!(first.is_satisfied());
    assert!(second.is_satisfied());
}

#[test]
fn zero_public_input_survives_compression() {
    let expected = compose(&ShapeCircuit::default());
    let bytes = ShapeCircuit::compress().expect("compression must succeed");
    let decoded = Composer::<Plonkish>::from_bytes(&bytes)
        .expect("description must decode");

    assert_eq!(decoded.shape(), expected.shape());
    assert_eq!(
        decoded.public_input_indexes(),
        expected.public_input_indexes()
    );
    assert_eq!(decoded.public_inputs(), vec![BlsScalar::zero()]);
    // Compressed descriptions intentionally contain no assignment: every
    // decoded witness and public value is zero, including compatibility rows
    // whose honest dummy assignment is nonzero.
}

#[test]
fn compressed_description_has_a_stable_digest() {
    let bytes = ShapeCircuit::compress().expect("compression must succeed");
    let digest: [u8; 32] = Sha256::digest(bytes).into();

    assert_eq!(
        digest,
        [
            172, 251, 115, 222, 227, 233, 193, 218, 135, 65, 181, 80, 187, 2,
            17, 200, 149, 197, 222, 198, 249, 241, 238, 254, 9, 150, 81, 235,
            159, 189, 39, 164,
        ]
    );
}

#[test]
fn identity_evaluator_covers_honest_gadget_families() {
    let mut composer = Composer::<Plonkish>::initialized();

    let left = composer.append_witness(BlsScalar::from(3u64));
    let right = composer.append_witness(BlsScalar::from(5u64));
    composer.gate_mul(Constraint::new().mult(1).a(left).b(right));

    let range = composer.append_witness(BlsScalar::from(0xabu64));
    composer.component_range_bits::<8>(range);
    composer.append_logic_xor::<4>(left, right);

    let point_a = composer
        .append_point(GENERATOR_EXTENDED)
        .expect("generator must be representable");
    let point_b = composer
        .append_point(GENERATOR_EXTENDED)
        .expect("generator must be representable");
    composer.component_add_point(
        TorsionFreeWitnessPoint::new_unchecked(point_a),
        TorsionFreeWitnessPoint::new_unchecked(point_b),
    );

    let scalar = composer.append_witness(JubJubScalar::from(17u64));
    composer
        .component_mul_generator(scalar, GENERATOR_EXTENDED)
        .expect("honest fixed-base multiplication must build");

    assert!(composer.is_satisfied());
}

#[test]
fn identity_evaluator_detects_a_corrupted_assignment() {
    let mut composer = Composer::<Plonkish>::initialized();
    let witness = composer.append_witness(BlsScalar::from(7u64));
    composer.assert_equal_constant(witness, BlsScalar::from(7u64), None);
    assert!(composer.is_satisfied());

    composer.witnesses[witness.index()] = BlsScalar::from(8u64);
    assert!(!composer.is_satisfied());
}

fn zero_z_point() -> JubJubExtended {
    let generator = JubJubAffine::from(GENERATOR_EXTENDED);
    JubJubExtended::from_raw_unchecked(
        generator.get_u(),
        generator.get_v(),
        BlsScalar::zero(),
        generator.get_u(),
        generator.get_v(),
    )
}

#[test]
fn point_validation_errors_remain_at_the_composer_boundary() {
    let mut composer = Composer::<Plonkish>::initialized();

    assert_eq!(
        composer.append_point(zero_z_point()).unwrap_err(),
        Error::JubJubPointDegenerate
    );
    assert_eq!(
        composer
            .append_constant_point(JubJubAffine::from_raw_unchecked(
                BlsScalar::zero(),
                BlsScalar::zero(),
            ))
            .unwrap_err(),
        Error::JubJubPointNotTorsionFree
    );
}

#[test]
fn build_rejects_a_different_gate_count() {
    let circuit = ShapeCircuit::default();
    let actual = circuit.size::<Plonkish>();

    assert_eq!(
        Composer::<Plonkish>::build(actual + 1, &circuit).unwrap_err(),
        Error::InvalidCircuitSize(actual, actual + 1)
    );
}
