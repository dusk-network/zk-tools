// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Shared circuit-building API and backend implementations.

use alloc::vec::Vec;
use core::ops;

use dusk_curves::bls12_381::BlsScalar;
#[cfg(feature = "plonkish")]
use hashbrown::HashMap;

use crate::error::Error;
#[cfg(feature = "plonkish")]
use crate::identities::{IDENTITY_COUNT, WireValues, evaluate};
use crate::runtime::{Runtime, RuntimeEvent};

mod bits;
mod circuit;
#[cfg(feature = "plonkish")]
mod compress;
mod constraint_system;
mod fixed_base;
#[cfg(feature = "plonkish")]
mod gate;
mod logic;
mod point;
mod range;
mod select;
mod truncate;

#[cfg(all(feature = "plonkish", feature = "test-api"))]
pub mod test_support {
    //! Unstable seams for adversarial proof-system regression tests.
    //!
    //! This module is not part of the supported Plonkish API. It is gated so
    //! production users cannot accidentally depend on raw assignment or gate
    //! construction hooks.

    use dusk_curves::bls12_381::BlsScalar;
    use dusk_jubjub::{JubJubAffine, JubJubExtended, JubJubScalar};

    use super::constraint_system::WiredWitness;
    use super::{
        Circuit, Composer, ComposerBackend, Constraint, Gate, Plonkish,
        Witness, WitnessPoint,
    };
    pub use crate::bit_iterator::BitIterator8;

    /// A raw Plonkish wire used by adversarial tests.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Wire {
        /// Left wire.
        A,
        /// Right wire.
        B,
        /// Output wire.
        C,
        /// Fourth wire.
        D,
    }

    const fn wire(wire: Wire) -> WiredWitness {
        match wire {
            Wire::A => WiredWitness::A,
            Wire::B => WiredWitness::B,
            Wire::C => WiredWitness::C,
            Wire::D => WiredWitness::D,
        }
    }

    /// Read a raw constraint wire.
    pub const fn constraint_witness(
        constraint: &Constraint,
        selected: Wire,
    ) -> Witness {
        constraint.witness(wire(selected))
    }

    /// Construct an unallocated witness handle.
    pub const fn witness(index: usize) -> Witness {
        Witness::new(index)
    }

    /// Read all gates from a Plonkish.
    pub fn gates(composer: &Composer<Plonkish>) -> &[Gate] {
        composer.gates()
    }

    /// Read the raw witness assignment.
    pub fn witnesses(composer: &Composer<Plonkish>) -> &[BlsScalar] {
        &composer.witnesses
    }

    /// Mutably access the raw witness assignment.
    pub fn witnesses_mut(
        composer: &mut Composer<Plonkish>,
    ) -> &mut [BlsScalar] {
        &mut composer.witnesses
    }

    /// Build a raw range constraint.
    pub fn range(constraint: &Constraint) -> Constraint {
        Constraint::range(constraint)
    }

    /// Build a raw AND constraint.
    pub fn logic(constraint: &Constraint) -> Constraint {
        Constraint::logic(constraint)
    }

    /// Build a raw XOR constraint.
    pub fn logic_xor(constraint: &Constraint) -> Constraint {
        Constraint::logic_xor(constraint)
    }

    /// Build a raw variable-base group-addition constraint.
    pub fn group_add_variable_base(constraint: &Constraint) -> Constraint {
        Constraint::group_add_variable_base(constraint)
    }

    /// Construct a raw point from coordinate witnesses.
    pub const fn witness_point(x: Witness, y: Witness) -> WitnessPoint {
        WitnessPoint::new(x, y)
    }

    /// Invoke the runtime-width range-check core.
    pub fn range_check<B: ComposerBackend>(
        composer: &mut Composer<B>,
        witness: Witness,
        bits: usize,
    ) {
        composer.range_check(witness, bits);
    }

    /// Bind raw logic accumulators to their input witnesses.
    pub fn bind_logic_accumulators<
        B: ComposerBackend,
        const BIT_PAIRS: usize,
    >(
        composer: &mut Composer<B>,
        a: Witness,
        b: Witness,
        left: Witness,
        right: Witness,
    ) {
        composer.bind_logic_accumulators::<BIT_PAIRS>(a, b, left, right);
    }

    /// Bind a raw truncated logic accumulator to its input.
    pub fn bind_truncated_input<B: ComposerBackend, const BIT_PAIRS: usize>(
        composer: &mut Composer<B>,
        input: Witness,
        accumulator: Witness,
    ) {
        composer.bind_truncated_input::<BIT_PAIRS>(input, accumulator);
    }

    /// Invoke the canonical truncation guard.
    pub fn assert_canonical_truncation<B: ComposerBackend>(
        composer: &mut Composer<B>,
        high: Witness,
        low: Witness,
        bits: usize,
    ) {
        composer.assert_canonical_truncation(high, low, bits);
    }

    /// Recompose a little-endian bit interval.
    pub fn recompose_bits(
        bits: &[u8; 256],
        start: usize,
        end: usize,
    ) -> BlsScalar {
        super::bits::recompose_bits(bits, start, end)
    }

    /// Canonical JubJub scalar bit width.
    pub const JUBJUB_SCALAR_BITS: usize = super::fixed_base::JUBJUB_SCALAR_BITS;
    /// Fixed-base signed-digit row count.
    pub const FIXED_BASE_SIGNED_DIGIT_ROUNDS: usize =
        super::fixed_base::FIXED_BASE_SIGNED_DIGIT_ROUNDS;
    /// Fixed-base leading-zero row count.
    pub const FIXED_BASE_LEADING_ZERO_ROUNDS: usize =
        super::fixed_base::FIXED_BASE_LEADING_ZERO_ROUNDS;
    /// Largest sound fixed-base signed-digit width.
    pub const FIXED_BASE_MAX_SOUND_WIDTH: usize =
        super::fixed_base::FIXED_BASE_MAX_SOUND_WIDTH;
    /// Inverse of eight in the JubJub scalar field.
    pub const EIGHT_INV: JubJubScalar = super::point::EIGHT_INV;

    /// Inject fixed-base signed digits into the production gate layout.
    pub fn append_fixed_base_signed_digits<B: ComposerBackend>(
        composer: &mut Composer<B>,
        jubjub: Witness,
        generator: JubJubExtended,
        signed_digits: &[i8; FIXED_BASE_SIGNED_DIGIT_ROUNDS],
    ) -> Result<WitnessPoint, crate::Error> {
        composer.append_fixed_base_signed_digits(
            jubjub,
            generator,
            signed_digits,
        )
    }

    /// Inject the subgroup-check helper point into the production layout.
    pub fn assert_torsion_free_gates<B: ComposerBackend>(
        composer: &mut Composer<B>,
        point: WitnessPoint,
        q: JubJubAffine,
    ) {
        composer.assert_torsion_free_gates(point, q);
    }

    /// Append raw variable-base point-addition gates.
    pub fn add_point_gates<B: ComposerBackend>(
        composer: &mut Composer<B>,
        a: WitnessPoint,
        b: WitnessPoint,
    ) -> WitnessPoint {
        composer.add_point_gates(a, b)
    }

    /// Build a Plonkish directly for a test circuit.
    pub fn compose<C: Circuit>(
        circuit: &C,
    ) -> Result<Composer<Plonkish>, crate::Error> {
        let mut composer = Composer::<Plonkish>::initialized();
        circuit.circuit(&mut composer)?;
        Ok(composer)
    }

    /// Extension methods used by adversarial Plonkish tests.
    pub trait ComposerTestExt {
        /// Append a raw component constraint.
        fn append_custom_gate(&mut self, constraint: Constraint);
        /// Invoke the runtime-width range-check core.
        fn range_check(&mut self, witness: Witness, bits: usize);
        /// Bind raw logic accumulators to their inputs.
        fn bind_logic_accumulators<const BIT_PAIRS: usize>(
            &mut self,
            a: Witness,
            b: Witness,
            left: Witness,
            right: Witness,
        );
        /// Bind one raw truncated accumulator to its input.
        fn bind_truncated_input<const BIT_PAIRS: usize>(
            &mut self,
            input: Witness,
            accumulator: Witness,
        );
        /// Invoke the canonical truncation guard.
        fn assert_canonical_truncation(
            &mut self,
            high: Witness,
            low: Witness,
            bits: usize,
        );
        /// Inject fixed-base signed digits into the production layout.
        fn append_fixed_base_signed_digits(
            &mut self,
            jubjub: Witness,
            generator: JubJubExtended,
            signed_digits: &[i8; FIXED_BASE_SIGNED_DIGIT_ROUNDS],
        ) -> Result<WitnessPoint, crate::Error>;
        /// Inject the subgroup-check helper point.
        fn assert_torsion_free_gates(
            &mut self,
            point: WitnessPoint,
            q: JubJubAffine,
        );
        /// Append raw variable-base addition gates.
        fn add_point_gates(
            &mut self,
            a: WitnessPoint,
            b: WitnessPoint,
        ) -> WitnessPoint;
    }

    impl<B: ComposerBackend> ComposerTestExt for Composer<B> {
        fn append_custom_gate(&mut self, constraint: Constraint) {
            Composer::<B>::append_custom_gate(self, constraint);
        }

        fn range_check(&mut self, witness: Witness, bits: usize) {
            Composer::<B>::range_check(self, witness, bits);
        }

        fn bind_logic_accumulators<const BIT_PAIRS: usize>(
            &mut self,
            a: Witness,
            b: Witness,
            left: Witness,
            right: Witness,
        ) {
            Composer::<B>::bind_logic_accumulators::<BIT_PAIRS>(
                self, a, b, left, right,
            );
        }

        fn bind_truncated_input<const BIT_PAIRS: usize>(
            &mut self,
            input: Witness,
            accumulator: Witness,
        ) {
            Composer::<B>::bind_truncated_input::<BIT_PAIRS>(
                self,
                input,
                accumulator,
            );
        }

        fn assert_canonical_truncation(
            &mut self,
            high: Witness,
            low: Witness,
            bits: usize,
        ) {
            Composer::<B>::assert_canonical_truncation(self, high, low, bits);
        }

        fn append_fixed_base_signed_digits(
            &mut self,
            jubjub: Witness,
            generator: JubJubExtended,
            signed_digits: &[i8; FIXED_BASE_SIGNED_DIGIT_ROUNDS],
        ) -> Result<WitnessPoint, crate::Error> {
            Composer::<B>::append_fixed_base_signed_digits(
                self,
                jubjub,
                generator,
                signed_digits,
            )
        }

        fn assert_torsion_free_gates(
            &mut self,
            point: WitnessPoint,
            q: JubJubAffine,
        ) {
            Composer::<B>::assert_torsion_free_gates(self, point, q);
        }

        fn add_point_gates(
            &mut self,
            a: WitnessPoint,
            b: WitnessPoint,
        ) -> WitnessPoint {
            Composer::<B>::add_point_gates(self, a, b)
        }
    }

    /// Extension methods for raw custom-gate construction in tests.
    pub trait ConstraintTestExt {
        /// Read a raw wire.
        fn witness(&self, selected: Wire) -> Witness;
        /// Select a raw range gate.
        fn range(constraint: &Self) -> Self;
        /// Select a raw AND gate.
        fn logic(constraint: &Self) -> Self;
        /// Select a raw XOR gate.
        fn logic_xor(constraint: &Self) -> Self;
    }

    impl ConstraintTestExt for Constraint {
        fn witness(&self, selected: Wire) -> Witness {
            Constraint::witness(self, wire(selected))
        }

        fn range(constraint: &Self) -> Self {
            Constraint::range(constraint)
        }

        fn logic(constraint: &Self) -> Self {
            Constraint::logic(constraint)
        }

        fn logic_xor(constraint: &Self) -> Self {
            Constraint::logic_xor(constraint)
        }
    }
}

#[cfg(all(test, feature = "plonkish"))]
mod tests;

pub use circuit::Circuit;
pub use constraint_system::{
    Constraint, TorsionFreeWitnessPoint, Witness, WitnessPoint,
};
pub(crate) use constraint_system::{Selector, WiredWitness};
#[cfg(feature = "plonkish")]
pub use gate::Gate;

/// PLONKish circuit shape.
///
/// Witness and public-input values are deliberately excluded: setup keys bind
/// the gate layout, wire indexes, witness count, and public-input positions.
#[cfg(feature = "plonkish")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CircuitShape {
    gates: Vec<Gate>,
    public_input_indexes: Vec<usize>,
    witnesses: usize,
}

#[cfg(feature = "plonkish")]
impl CircuitShape {
    /// Gates in row order.
    pub fn gates(&self) -> &[Gate] {
        &self.gates
    }

    /// Sparse public-input row indexes in ascending order.
    pub fn public_input_indexes(&self) -> &[usize] {
        &self.public_input_indexes
    }

    /// Number of witnesses allocated by the circuit.
    pub const fn witnesses(&self) -> usize {
        self.witnesses
    }
}

/// Constraint-system implementation used by a generic [`Composer`].
///
/// Backends receive the low-level constraints emitted by the portable
/// Composer API and decide how to represent them. The PLONKish backend stores
/// width-four gates; other backends can lower the same operations into their
/// own constraint representation.
pub trait ComposerBackend: Clone + core::fmt::Debug {
    /// Create an empty backend constraint system.
    fn new() -> Self;

    /// Number of constraints emitted so far.
    fn constraints(&self) -> usize;

    /// Append one low-level Composer constraint in circuit emission order.
    ///
    /// Implementations use [`Constraint::wires`], the `q_*` selector
    /// accessors, and [`Constraint::public_input`] to inspect it.
    fn append_constraint(&mut self, constraint: &Constraint);

    /// Append backend-specific initialization constraints.
    ///
    /// The Composer allocates and constrains its canonical zero and one
    /// witnesses before invoking this hook.
    fn initialize(_composer: &mut Composer<Self>) {}
}

/// PLONKish width-four gate backend.
#[cfg(feature = "plonkish")]
#[derive(Debug, Clone, Default)]
pub struct Plonkish {
    constraints: Vec<Gate>,
    public_inputs: HashMap<usize, BlsScalar>,
}

#[cfg(feature = "plonkish")]
impl ComposerBackend for Plonkish {
    fn new() -> Self {
        Self::default()
    }

    fn constraints(&self) -> usize {
        self.constraints.len()
    }

    fn append_constraint(&mut self, constraint: &Constraint) {
        let row = self.constraints.len();
        if constraint.has_public_input() {
            let public = *constraint.coeff(Selector::PublicInput);
            self.public_inputs.insert(row, public);
        }
        self.constraints.push(Gate::from_constraint(constraint));
    }

    fn initialize(composer: &mut Composer<Self>) {
        composer.append_plonkish_compatibility_gates();
    }
}

/// Construct a circuit using backend `B`.
#[derive(Debug, Clone)]
pub struct Composer<B: ComposerBackend> {
    backend: B,

    /// Public inputs in circuit emission order.
    pub(crate) public_inputs: Vec<BlsScalar>,

    /// Witness values.
    pub(crate) witnesses: Vec<BlsScalar>,

    /// Circuit-construction runtime controller.
    pub(crate) runtime: Runtime,
}

impl<B: ComposerBackend> ops::Index<Witness> for Composer<B> {
    type Output = BlsScalar;

    fn index(&self, w: Witness) -> &Self::Output {
        &self.witnesses[w.index()]
    }
}

/// Backend-independent circuit builder operations.
impl<B: ComposerBackend> Composer<B> {
    /// Identity point representation inside the constraint system. The
    /// identity is the prime-order subgroup's neutral element, so it carries
    /// the [`TorsionFreeWitnessPoint`] membership by construction.
    pub const IDENTITY: TorsionFreeWitnessPoint =
        TorsionFreeWitnessPoint::new_unchecked(WitnessPoint::new(
            Self::ZERO,
            Self::ONE,
        ));
    /// `One` representation inside the constraint system.
    ///
    /// Every Composer reserves its second witness for one.
    pub const ONE: Witness = Witness::ONE;
    /// Zero representation inside the constraint system.
    ///
    /// Every Composer reserves its first witness for zero.
    pub const ZERO: Witness = Witness::ZERO;

    /// Constraints count
    pub fn constraints(&self) -> usize {
        self.backend.constraints()
    }

    /// Number of allocated witnesses.
    pub fn witness_count(&self) -> usize {
        self.witnesses.len()
    }

    /// Return all allocated witness values in allocation order.
    pub fn witnesses(&self) -> &[BlsScalar] {
        &self.witnesses
    }

    /// Return the selected backend representation.
    pub const fn backend(&self) -> &B {
        &self.backend
    }

    /// Consume the Composer and return its backend representation.
    pub fn into_backend(self) -> B {
        self.backend
    }

    /// Allocate a witness value into the composer and return its index.
    fn append_witness_internal(&mut self, witness: BlsScalar) -> Witness {
        let n = self.witnesses.len();

        // Bind the allocated witness
        self.witnesses.push(witness);

        Witness::new(n)
    }

    /// Append a new width-4 gate/constraint.
    fn append_custom_gate_internal(&mut self, constraint: Constraint) {
        if constraint.has_public_input() {
            self.public_inputs
                .push(*constraint.coeff(Selector::PublicInput));
        }

        self.backend.append_constraint(&constraint);
    }

    /// Circuit-construction runtime controller.
    pub(crate) fn runtime(&mut self) -> &mut Runtime {
        &mut self.runtime
    }

    /// Initialize the constraint system with the canonical constants and any
    /// backend-specific compatibility constraints.
    pub fn initialized() -> Self {
        Self::with_backend(B::new())
    }

    /// Initialize the constraint system from an existing backend value.
    ///
    /// This supports backends that carry configuration or caller-provided
    /// state while preserving the same canonical zero/one setup and backend
    /// initialization hook as [`Self::initialized`].
    pub fn with_backend(backend: B) -> Self {
        let mut slf = Self::uninitialized_with_backend(backend);

        let zero = slf.append_witness(0);
        let one = slf.append_witness(1);

        slf.assert_equal_constant(zero, 0, None);
        slf.assert_equal_constant(one, 1, None);

        B::initialize(&mut slf);

        slf
    }

    /// Create an empty constraint system.
    ///
    /// This shouldn't be used directly; instead, use [`Self::initialized`]
    #[cfg(feature = "plonkish")]
    pub(crate) fn uninitialized() -> Self {
        Self::uninitialized_with_backend(B::new())
    }

    fn uninitialized_with_backend(backend: B) -> Self {
        Self {
            backend,
            public_inputs: Vec::new(),
            witnesses: Vec::new(),
            runtime: Runtime::new(),
        }
    }

    /// Append the two historical compatibility constraints.
    ///
    /// They keep circuit row layouts byte-for-byte compatible with existing
    /// PLONKish descriptions and are installed only by the [`Plonkish`]
    /// backend's initialization hook.
    #[cfg(feature = "plonkish")]
    fn append_plonkish_compatibility_gates(&mut self) {
        let six = self.append_witness(BlsScalar::from(6));
        let one = self.append_witness(BlsScalar::from(1));
        let seven = self.append_witness(BlsScalar::from(7));
        let min_twenty = self.append_witness(-BlsScalar::from(20));

        // Add a dummy constraint so that we do not have zero polynomials
        let constraint = Constraint::new()
            .mult(1)
            .left(2)
            .right(3)
            .fourth(1)
            .constant(4)
            .output(4)
            .a(six)
            .b(seven)
            .d(one)
            .c(min_twenty);

        self.append_gate(constraint);

        // Retain the second historical compatibility constraint.
        let constraint = Constraint::new()
            .mult(1)
            .left(1)
            .right(1)
            .constant(127)
            .output(1)
            .a(min_twenty)
            .b(six)
            .c(seven);

        self.append_gate(constraint);
    }

    /// Allocate a witness value into the composer and return its index.
    pub fn append_witness<W: Into<BlsScalar>>(
        &mut self,
        witness: W,
    ) -> Witness {
        let witness = witness.into();

        let witness = self.append_witness_internal(witness);

        #[cfg(feature = "debug")]
        let v = self[witness];
        self.runtime().event(RuntimeEvent::WitnessAppended {
            #[cfg(feature = "debug")]
            w: witness,
            #[cfg(feature = "debug")]
            v,
        });

        witness
    }

    /// Append a raw component constraint.
    pub(crate) fn append_custom_gate(&mut self, constraint: Constraint) {
        self.runtime().event(RuntimeEvent::ConstraintAppended {
            #[cfg(feature = "debug")]
            c: constraint,
        });

        self.append_custom_gate_internal(constraint)
    }

    /// Append a new width-4 gate/constraint.
    ///
    /// The constraint added will enforce the following:
    /// `q_M · a · b  + q_L · a + q_R · b + q_O · o + q_F · d + q_C + PI = 0`.
    pub fn append_gate(&mut self, constraint: Constraint) {
        let constraint = Constraint::arithmetic(&constraint);

        self.append_custom_gate(constraint)
    }

    /// Evaluate an arithmetic constraint, allocate its output, and append the
    /// gate that constrains that output to the inputs.
    ///
    /// For an invertible output selector `q_O`, this solves
    ///
    /// `q_M·a·b + q_L·a + q_R·b + q_O·c + q_F·d + q_C + PI = 0`
    ///
    /// for `c`, appends `c` as a witness, wires it into the constraint, and
    /// appends exactly one active arithmetic gate.
    ///
    /// If `q_O` is zero, no output can be solved for: this returns `None` but
    /// still appends exactly one arithmetic gate enforcing the supplied
    /// polynomial on its input witnesses.
    ///
    /// The appended gate is the soundness boundary. Computing `c` from host
    /// witness values alone is not a circuit constraint: without this row, a
    /// malicious prover could replace `c` while preserving the circuit shape.
    ///
    /// # Circuit compatibility
    ///
    /// Direct callers now receive one arithmetic row and must regenerate their
    /// circuit-specific proving and verifier keys, and any cached compressed
    /// circuit description. [`Self::gate_add`] and [`Self::gate_mul`] still
    /// emit exactly one row, so their circuit layouts are unchanged.
    pub fn append_evaluated_output(
        &mut self,
        mut s: Constraint,
    ) -> Option<Witness> {
        let a = s.witness(WiredWitness::A);
        let b = s.witness(WiredWitness::B);
        let d = s.witness(WiredWitness::D);

        let a = self[a];
        let b = self[b];
        let d = self[d];

        let qm = s.coeff(Selector::Multiplication);
        let ql = s.coeff(Selector::Left);
        let qr = s.coeff(Selector::Right);
        let qf = s.coeff(Selector::Fourth);
        let qc = s.coeff(Selector::Constant);
        let pi = s.coeff(Selector::PublicInput);

        let x = qm * a * b + ql * a + qr * b + qf * d + qc + pi;

        let y = s.coeff(Selector::Output);

        // Invert is an expensive operation; in most cases, `q_O` is going to be
        // either 1 or -1, so we can optimize these
        let c = {
            const ONE: BlsScalar = BlsScalar::one();
            const MINUS_ONE: BlsScalar = BlsScalar([
                0xfffffffd00000003,
                0xfb38ec08fffb13fc,
                0x99ad88181ce5880f,
                0x5bc8f5f97cd877d8,
            ]);

            // Can't use a match pattern here since `BlsScalar` doesn't derive
            // `PartialEq`
            if y == &ONE {
                Some(-x)
            } else if y == &MINUS_ONE {
                Some(x)
            } else {
                y.invert().map(|y| x * (-y))
            }
        };

        let output = c.map(|c| self.append_witness(c));
        if let Some(output) = output {
            s = s.c(output);
        }
        self.append_gate(s);

        output
    }

    /// Constrain a scalar into the circuit description and return an allocated
    /// [`Witness`] with its value
    pub fn append_constant<C: Into<BlsScalar>>(
        &mut self,
        constant: C,
    ) -> Witness {
        let constant = constant.into();
        let witness = self.append_witness(constant);

        self.assert_equal_constant(witness, constant, None);

        witness
    }

    /// Allocate a witness value into the composer and return its index.
    ///
    /// Create a public input with the scalar
    pub fn append_public<P: Into<BlsScalar>>(&mut self, public: P) -> Witness {
        let public = public.into();
        let witness = self.append_witness(public);

        let constraint = Constraint::new()
            .left(-BlsScalar::one())
            .a(witness)
            .public(public);
        self.append_gate(constraint);

        witness
    }

    /// Asserts `a == b` by appending a gate
    pub fn assert_equal(&mut self, a: Witness, b: Witness) {
        let constraint =
            Constraint::new().left(1).right(-BlsScalar::one()).a(a).b(b);

        self.append_gate(constraint);
    }

    /// Constrain `a` to be equal to `constant + pi`.
    ///
    /// `constant` will be defined as part of the public circuit description.
    pub fn assert_equal_constant<C: Into<BlsScalar>>(
        &mut self,
        a: Witness,
        constant: C,
        public: Option<BlsScalar>,
    ) {
        let constant = constant.into();
        let constraint = Constraint::new()
            .left(-BlsScalar::one())
            .a(a)
            .constant(constant);
        let constraint =
            public.map(|p| constraint.public(p)).unwrap_or(constraint);

        self.append_gate(constraint);
    }

    /// Evaluate and return `o` by appending a new constraint into the circuit.
    ///
    /// Set `q_O = (-1)` and override the output of the constraint with:
    /// `c := q_L · a + q_R · b + q_F · d + q_C + PI`
    pub fn gate_add(&mut self, s: Constraint) -> Witness {
        let s = Constraint::arithmetic(&s).output(-BlsScalar::one());

        self.append_evaluated_output(s)
            .expect("output selector is -1")
    }

    /// Evaluate and return `c` by appending a new constraint into the circuit.
    ///
    /// Set `q_O = (-1)` and override the output of the constraint with:
    /// `c := q_M · a · b + q_F · d + q_C + PI`
    pub fn gate_mul(&mut self, s: Constraint) -> Witness {
        let s = Constraint::arithmetic(&s).output(-BlsScalar::one());

        self.append_evaluated_output(s)
            .expect("output selector is -1")
    }

    /// Build a circuit with a composer initialized with dummy gates.
    pub fn build<C>(constraints: usize, circuit: &C) -> Result<Self, Error>
    where
        C: Circuit,
    {
        let mut composer = Self::initialized();

        circuit.circuit(&mut composer)?;

        // assert that the circuit has the same amount of constraints as the
        // circuit description
        let description_size = composer.constraints();
        if description_size != constraints {
            return Err(Error::InvalidCircuitSize(
                description_size,
                constraints,
            ));
        }

        composer.runtime().event(RuntimeEvent::CircuitFinished);

        Ok(composer)
    }

    /// Public-input values in circuit emission order.
    pub fn public_inputs(&self) -> Vec<BlsScalar> {
        self.public_inputs.clone()
    }
}

#[cfg(feature = "plonkish")]
impl Composer<Plonkish> {
    /// Sparse public-input row indexes in ascending order.
    pub fn public_input_indexes(&self) -> Vec<usize> {
        let mut public_input_indexes: Vec<_> =
            self.backend.public_inputs.keys().copied().collect();

        public_input_indexes.as_mut_slice().sort();

        public_input_indexes
    }

    /// Expand sparse public inputs into a row-aligned vector of `size`.
    ///
    /// The indexes and values are paired in iteration order. Callers must
    /// provide indexes smaller than `size`; PLONKish-produced indexes satisfy
    /// that requirement when `size` covers all circuit rows.
    pub fn dense_public_inputs(
        public_input_indexes: &[usize],
        public_inputs: &[BlsScalar],
        size: usize,
    ) -> Vec<BlsScalar> {
        let mut dense_public_inputs = vec![BlsScalar::zero(); size];

        public_input_indexes
            .iter()
            .zip(public_inputs.iter())
            .for_each(|(idx, pi)| dense_public_inputs[*idx] = *pi);

        dense_public_inputs
    }

    /// Return PLONKish gates in row order.
    pub fn gates(&self) -> &[Gate] {
        &self.backend.constraints
    }

    /// Return the PLONKish circuit shape without assignment values.
    pub fn shape(&self) -> CircuitShape {
        CircuitShape {
            gates: self.backend.constraints.clone(),
            public_input_indexes: self.public_input_indexes(),
            witnesses: self.witnesses.len(),
        }
    }

    fn gate_wire_values(&self, gate: &Gate) -> Option<WireValues<BlsScalar>> {
        let [a, b, c, d] = gate.wires();

        Some(WireValues::new(
            *self.witnesses.get(a.index())?,
            *self.witnesses.get(b.index())?,
            *self.witnesses.get(c.index())?,
            *self.witnesses.get(d.index())?,
        ))
    }

    /// Evaluate every independent PLONKish gate identity at `row`.
    pub fn identity_evaluations(
        &self,
        row: usize,
    ) -> Option<[BlsScalar; IDENTITY_COUNT]> {
        let gate = self.backend.constraints.get(row)?;
        let current = self.gate_wire_values(gate)?;
        let padded_size = self.backend.constraints.len().next_power_of_two();
        let shifted = (row + 1) % padded_size;
        let next = match self.backend.constraints.get(shifted) {
            Some(next) => self.gate_wire_values(next)?,
            None => WireValues::new(
                BlsScalar::zero(),
                BlsScalar::zero(),
                BlsScalar::zero(),
                BlsScalar::zero(),
            ),
        };
        let public_input = self
            .backend
            .public_inputs
            .get(&row)
            .copied()
            .unwrap_or_default();

        Some(evaluate(gate, current, next, public_input))
    }

    /// Return whether the current assignment satisfies every PLONKish row.
    pub fn is_satisfied(&self) -> bool {
        (0..self.backend.constraints.len()).all(|row| {
            self.identity_evaluations(row).is_some_and(|identities| {
                identities
                    .into_iter()
                    .all(|identity| identity == BlsScalar::zero())
            })
        })
    }

    /// Create a PLONKish Composer from a compressed circuit description.
    pub fn from_bytes(compressed: &[u8]) -> Result<Self, Error> {
        compress::CompressedCircuit::from_bytes(compressed)
    }
}
