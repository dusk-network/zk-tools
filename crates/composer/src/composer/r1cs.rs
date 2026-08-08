// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Rank-1 constraint-system backend and symbolic Plonkish-identity lowering.

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::ops::{Add, Mul, Sub};

use dusk_curves::bls12_381::BlsScalar;
use sha2::{Digest, Sha256};
#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

use super::{Composer, ComposerBackend, Constraint, Gate, Witness};
use crate::identities::{self, IdentityElement, WireValues};

/// A variable in a finalized rank-1 constraint system.
///
/// Index zero is the implicit constant-one variable. Public inputs follow,
/// then Composer witnesses and lowering auxiliaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variable(usize);

impl Variable {
    /// Return the variable's canonical assignment index.
    pub const fn index(self) -> usize {
        self.0
    }
}

/// A normalized sparse linear combination.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LinearCombination {
    terms: Vec<(Variable, BlsScalar)>,
}

impl LinearCombination {
    fn zero() -> Self {
        Self::default()
    }

    fn one() -> Self {
        Self::from_variable(Variable(0))
    }

    fn from_variable(variable: Variable) -> Self {
        Self {
            terms: vec![(variable, BlsScalar::one())],
        }
    }

    fn from_constant(value: BlsScalar) -> Self {
        if value == BlsScalar::zero() {
            Self::zero()
        } else {
            Self {
                terms: vec![(Variable(0), value)],
            }
        }
    }

    fn add_term(&mut self, variable: Variable, coefficient: BlsScalar) {
        if coefficient == BlsScalar::zero() {
            return;
        }

        match self
            .terms
            .binary_search_by_key(&variable, |(variable, _)| *variable)
        {
            Ok(index) => {
                self.terms[index].1 += coefficient;
                if self.terms[index].1 == BlsScalar::zero() {
                    self.terms.remove(index);
                }
            }
            Err(index) => self.terms.insert(index, (variable, coefficient)),
        }
    }

    fn add_assign_scaled(&mut self, other: &Self, scale: BlsScalar) {
        for (variable, coefficient) in &other.terms {
            self.add_term(*variable, *coefficient * scale);
        }
    }

    fn scaled(mut self, scale: BlsScalar) -> Self {
        if scale == BlsScalar::zero() {
            return Self::zero();
        }
        for (_, coefficient) in &mut self.terms {
            *coefficient *= scale;
        }
        self
    }

    fn negated(self) -> Self {
        self.scaled(-BlsScalar::one())
    }

    fn evaluate(&self, assignment: &[BlsScalar]) -> Option<BlsScalar> {
        self.terms.iter().try_fold(
            BlsScalar::zero(),
            |accumulator, (variable, coefficient)| {
                assignment
                    .get(variable.index())
                    .map(|value| accumulator + *value * coefficient)
            },
        )
    }

    /// Iterate over nonzero `(variable, coefficient)` terms in index order.
    pub fn terms(
        &self,
    ) -> impl ExactSizeIterator<Item = &(Variable, BlsScalar)> {
        self.terms.iter()
    }
}

/// One rank-1 equation `A(z) * B(z) = C(z)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct R1csConstraint {
    a: LinearCombination,
    b: LinearCombination,
    c: LinearCombination,
}

impl R1csConstraint {
    /// Left multiplicand.
    pub const fn a(&self) -> &LinearCombination {
        &self.a
    }

    /// Right multiplicand.
    pub const fn b(&self) -> &LinearCombination {
        &self.b
    }

    /// Product result.
    pub const fn c(&self) -> &LinearCombination {
        &self.c
    }

    /// Evaluate this equation against an assignment.
    pub fn is_satisfied(&self, assignment: &[BlsScalar]) -> bool {
        let Some(a) = self.a.evaluate(assignment) else {
            return false;
        };
        let Some(b) = self.b.evaluate(assignment) else {
            return false;
        };
        let Some(c) = self.c.evaluate(assignment) else {
            return false;
        };

        a * b == c
    }
}

/// Value-independent rank-1 circuit description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct R1csShape {
    constraints: Vec<R1csConstraint>,
    variables: usize,
    public_inputs: usize,
}

impl R1csShape {
    /// Rank-1 constraints in emission order.
    pub fn constraints(&self) -> &[R1csConstraint] {
        &self.constraints
    }

    /// Total assignment length, including the constant-one variable.
    pub const fn variables(&self) -> usize {
        self.variables
    }

    /// Number of public statement variables.
    pub const fn public_inputs(&self) -> usize {
        self.public_inputs
    }

    /// Canonical SHA-256 digest of the shape.
    pub fn digest(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"dusk-r1cs-shape-v1");
        hasher.update((self.variables as u64).to_be_bytes());
        hasher.update((self.public_inputs as u64).to_be_bytes());
        hasher.update((self.constraints.len() as u64).to_be_bytes());

        for constraint in &self.constraints {
            for combination in [&constraint.a, &constraint.b, &constraint.c] {
                hasher.update((combination.terms.len() as u64).to_be_bytes());
                for (variable, coefficient) in &combination.terms {
                    hasher.update((variable.index() as u64).to_be_bytes());
                    hasher.update(coefficient.to_bytes());
                }
            }
        }

        hasher.finalize().into()
    }
}

/// Canonically ordered values for a finalized rank-1 circuit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct R1csAssignment {
    values: Vec<BlsScalar>,
}

impl R1csAssignment {
    /// All values in canonical variable order.
    pub fn values(&self) -> &[BlsScalar] {
        &self.values
    }
}

#[cfg(feature = "zeroize")]
impl Drop for R1csAssignment {
    fn drop(&mut self) {
        self.values.zeroize();
    }
}

/// A finalized rank-1 shape and its assignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct R1csCircuit {
    shape: R1csShape,
    assignment: R1csAssignment,
}

impl R1csCircuit {
    /// Value-independent circuit shape.
    pub const fn shape(&self) -> &R1csShape {
        &self.shape
    }

    /// Assignment corresponding to the shape.
    pub const fn assignment(&self) -> &R1csAssignment {
        &self.assignment
    }

    /// Return whether every rank-1 equation is satisfied.
    pub fn is_satisfied(&self) -> bool {
        self.shape
            .constraints
            .iter()
            .all(|constraint| constraint.is_satisfied(self.assignment.values()))
    }
}

#[derive(Debug, Clone)]
struct Row {
    gate: Gate,
    has_public_input: bool,
}

/// Backend that lowers Composer rows into rank-1 equations.
#[derive(Debug, Clone, Default)]
pub struct R1cs {
    rows: Vec<Row>,
}

impl ComposerBackend for R1cs {
    fn new() -> Self {
        Self::default()
    }

    fn constraints(&self) -> usize {
        if self.rows.is_empty() {
            return 0;
        }

        let witness_count = self
            .rows
            .iter()
            .flat_map(|row| row.gate.wires())
            .map(|witness| witness.index() + 1)
            .max()
            .unwrap_or_default();
        let public_inputs =
            self.rows.iter().filter(|row| row.has_public_input).count();
        let public_values = vec![BlsScalar::zero(); public_inputs];
        let witnesses = vec![BlsScalar::zero(); witness_count];

        lower_rows(&self.rows, &public_values, &witnesses)
            .shape
            .constraints
            .len()
    }

    fn append_constraint(&mut self, constraint: &Constraint) {
        self.rows.push(Row {
            gate: Gate::from_constraint(constraint),
            has_public_input: constraint.public_input().is_some(),
        });
    }
}

impl Composer<R1cs> {
    /// Finalize the current Composer into a rank-1 shape and assignment.
    ///
    /// Assignment order is constant one, public inputs in emission order,
    /// Composer witnesses, and symbolic-lowering auxiliary products.
    pub fn to_r1cs(&self) -> R1csCircuit {
        lower_rows(&self.backend.rows, &self.public_inputs, &self.witnesses)
    }

    /// Consume the Composer and finalize it into rank-1 form.
    ///
    /// This produces the same canonical result as [`Self::to_r1cs`] without
    /// retaining the Composer.
    pub fn into_r1cs(self) -> R1csCircuit {
        lower_rows(&self.backend.rows, &self.public_inputs, &self.witnesses)
    }
}

#[derive(Debug, Clone)]
enum Expression {
    Constant(BlsScalar),
    Variable(Variable),
    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Mul(Box<Self>, Box<Self>),
}

impl Expression {
    fn variable(variable: Variable) -> Self {
        Self::Variable(variable)
    }

    fn is_constant(&self, expected: BlsScalar) -> bool {
        matches!(self, Self::Constant(value) if *value == expected)
    }

    fn as_constant(&self) -> Option<BlsScalar> {
        match self {
            Self::Constant(value) => Some(*value),
            _ => None,
        }
    }
}

impl IdentityElement for Expression {
    fn constant(value: BlsScalar) -> Self {
        Self::Constant(value)
    }
}

impl Add for Expression {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.is_constant(BlsScalar::zero()) {
            return rhs;
        }
        if rhs.is_constant(BlsScalar::zero()) {
            return self;
        }
        if let (Some(left), Some(right)) =
            (self.as_constant(), rhs.as_constant())
        {
            return Self::Constant(left + right);
        }
        Self::Add(Box::new(self), Box::new(rhs))
    }
}

impl Sub for Expression {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        if rhs.is_constant(BlsScalar::zero()) {
            return self;
        }
        if let (Some(left), Some(right)) =
            (self.as_constant(), rhs.as_constant())
        {
            return Self::Constant(left - right);
        }
        Self::Sub(Box::new(self), Box::new(rhs))
    }
}

impl Mul for Expression {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.is_constant(BlsScalar::zero())
            || rhs.is_constant(BlsScalar::zero())
        {
            return Self::Constant(BlsScalar::zero());
        }
        if self.is_constant(BlsScalar::one()) {
            return rhs;
        }
        if rhs.is_constant(BlsScalar::one()) {
            return self;
        }
        if let (Some(left), Some(right)) =
            (self.as_constant(), rhs.as_constant())
        {
            return Self::Constant(left * right);
        }
        Self::Mul(Box::new(self), Box::new(rhs))
    }
}

struct Lowerer {
    constraints: Vec<R1csConstraint>,
    assignment: Vec<BlsScalar>,
}

impl Lowerer {
    fn lower(&mut self, expression: Expression) -> LinearCombination {
        match expression {
            Expression::Constant(value) => {
                LinearCombination::from_constant(value)
            }
            Expression::Variable(variable) => {
                LinearCombination::from_variable(variable)
            }
            Expression::Add(left, right) => {
                let mut left = self.lower(*left);
                let right = self.lower(*right);
                left.add_assign_scaled(&right, BlsScalar::one());
                left
            }
            Expression::Sub(left, right) => {
                let mut left = self.lower(*left);
                let right = self.lower(*right);
                left.add_assign_scaled(&right, -BlsScalar::one());
                left
            }
            Expression::Mul(left, right) => {
                if let Expression::Constant(scale) = *left {
                    return self.lower(*right).scaled(scale);
                }
                if let Expression::Constant(scale) = *right {
                    return self.lower(*left).scaled(scale);
                }

                let left = self.lower(*left);
                let right = self.lower(*right);
                self.allocate_product(left, right)
            }
        }
    }

    fn allocate_product(
        &mut self,
        left: LinearCombination,
        right: LinearCombination,
    ) -> LinearCombination {
        let left_value = left
            .evaluate(&self.assignment)
            .expect("lowered variables have assignments");
        let right_value = right
            .evaluate(&self.assignment)
            .expect("lowered variables have assignments");
        let variable = Variable(self.assignment.len());
        self.assignment.push(left_value * right_value);
        let output = LinearCombination::from_variable(variable);
        self.constraints.push(R1csConstraint {
            a: left,
            b: right,
            c: output.clone(),
        });
        output
    }

    fn enforce_zero(&mut self, expression: Expression) {
        if expression.is_constant(BlsScalar::zero()) {
            return;
        }

        let mut terms = Vec::new();
        flatten_terms(expression, BlsScalar::one(), &mut terms);
        if let Some(index) = terms.iter().position(|(_, expression)| {
            matches!(expression, Expression::Mul(_, _))
        }) {
            let (sign, product) = terms.remove(index);
            let Expression::Mul(left, right) = product else {
                unreachable!("selected multiplication expression")
            };
            let left = self.lower(*left);
            let right = self.lower(*right);
            let mut remainder = LinearCombination::zero();
            for (coefficient, expression) in terms {
                let lowered = self.lower(expression);
                remainder.add_assign_scaled(&lowered, coefficient);
            }
            let output = if sign == BlsScalar::one() {
                remainder.negated()
            } else {
                remainder
            };
            self.constraints.push(R1csConstraint {
                a: left,
                b: right,
                c: output,
            });
            return;
        }

        let linear = self.lower(terms.into_iter().fold(
            Expression::Constant(BlsScalar::zero()),
            |sum, (sign, term)| sum + Expression::Constant(sign) * term,
        ));
        self.constraints.push(R1csConstraint {
            a: linear,
            b: LinearCombination::one(),
            c: LinearCombination::zero(),
        });
    }
}

fn flatten_terms(
    expression: Expression,
    sign: BlsScalar,
    terms: &mut Vec<(BlsScalar, Expression)>,
) {
    match expression {
        Expression::Add(left, right) => {
            flatten_terms(*left, sign, terms);
            flatten_terms(*right, sign, terms);
        }
        Expression::Sub(left, right) => {
            flatten_terms(*left, sign, terms);
            flatten_terms(*right, -sign, terms);
        }
        expression => terms.push((sign, expression)),
    }
}

fn wire_values(gate: &Gate, public_inputs: usize) -> WireValues<Expression> {
    let [a, b, c, d] = gate.wires();
    let variable = |witness: Witness| {
        Expression::variable(Variable(1 + public_inputs + witness.index()))
    };

    WireValues::new(variable(a), variable(b), variable(c), variable(d))
}

fn zero_wire_values() -> WireValues<Expression> {
    let zero = Expression::Constant(BlsScalar::zero());
    WireValues::new(zero.clone(), zero.clone(), zero.clone(), zero)
}

fn lower_rows(
    rows: &[Row],
    public_values: &[BlsScalar],
    witnesses: &[BlsScalar],
) -> R1csCircuit {
    let public_inputs = rows.iter().filter(|row| row.has_public_input).count();
    assert_eq!(
        public_values.len(),
        public_inputs,
        "Composer public inputs match public rows"
    );

    let mut assignment =
        Vec::with_capacity(1 + public_values.len() + witnesses.len());
    assignment.push(BlsScalar::one());
    assignment.extend_from_slice(public_values);
    assignment.extend_from_slice(witnesses);
    let mut lowerer = Lowerer {
        constraints: Vec::new(),
        assignment,
    };

    let padded_size = rows.len().next_power_of_two();
    let mut public_index = 0;
    for (row_index, row) in rows.iter().enumerate() {
        let current = wire_values(&row.gate, public_inputs);
        let next = if let Some(next) = rows.get(row_index + 1) {
            wire_values(&next.gate, public_inputs)
        } else if rows.len() == padded_size {
            rows.first()
                .map(|first| wire_values(&first.gate, public_inputs))
                .unwrap_or_else(zero_wire_values)
        } else {
            zero_wire_values()
        };
        let public = if row.has_public_input {
            let expression = Expression::variable(Variable(1 + public_index));
            public_index += 1;
            expression
        } else {
            Expression::Constant(BlsScalar::zero())
        };

        for identity in identities::evaluate(&row.gate, current, next, public) {
            lowerer.enforce_zero(identity);
        }
    }

    let shape = R1csShape {
        variables: lowerer.assignment.len(),
        public_inputs,
        constraints: lowerer.constraints,
    };
    let assignment = R1csAssignment {
        values: lowerer.assignment,
    };

    R1csCircuit { shape, assignment }
}

#[cfg(test)]
mod tests {
    use dusk_jubjub::{GENERATOR_EXTENDED, JubJubScalar};

    use super::*;
    use crate::prelude::{Circuit, CircuitError};

    #[derive(Default)]
    struct ProductCircuit {
        left: BlsScalar,
        right: BlsScalar,
        result: BlsScalar,
    }

    impl Circuit for ProductCircuit {
        fn circuit<B: ComposerBackend>(
            &self,
            composer: &mut Composer<B>,
        ) -> Result<(), CircuitError> {
            let left = composer.append_witness(self.left);
            let right = composer.append_witness(self.right);
            let product =
                composer.gate_mul(Constraint::new().mult(1).a(left).b(right));
            let result = composer.append_public(self.result);
            composer.assert_equal(product, result);
            Ok(())
        }
    }

    fn compose(circuit: &ProductCircuit) -> R1csCircuit {
        let mut composer = Composer::<R1cs>::initialized();
        circuit.circuit(&mut composer).unwrap();
        composer.into_r1cs()
    }

    #[test]
    fn arithmetic_assignment_is_satisfied() {
        let r1cs = compose(&ProductCircuit {
            left: BlsScalar::from(3u64),
            right: BlsScalar::from(4u64),
            result: BlsScalar::from(12u64),
        });
        assert!(r1cs.is_satisfied());
        assert_eq!(r1cs.shape().public_inputs(), 1);
        assert_eq!(r1cs.assignment().values()[0], BlsScalar::one());
        assert_eq!(r1cs.assignment().values()[1], BlsScalar::from(12u64));
    }

    #[test]
    fn invalid_public_result_is_rejected() {
        let r1cs = compose(&ProductCircuit {
            left: BlsScalar::from(3u64),
            right: BlsScalar::from(4u64),
            result: BlsScalar::from(13u64),
        });
        assert!(!r1cs.is_satisfied());
    }

    #[test]
    fn public_values_do_not_change_shape() {
        let zero = compose(&ProductCircuit::default());
        let populated = compose(&ProductCircuit {
            left: BlsScalar::from(3u64),
            right: BlsScalar::from(4u64),
            result: BlsScalar::from(12u64),
        });
        assert_eq!(zero.shape(), populated.shape());
        assert_eq!(zero.shape().digest(), populated.shape().digest());
        assert_eq!(
            zero.shape().digest(),
            [
                8, 185, 1, 1, 104, 46, 204, 122, 67, 53, 84, 68, 96, 54, 120,
                244, 229, 78, 129, 71, 254, 75, 207, 141, 151, 249, 245, 179,
                255, 172, 207, 112,
            ]
        );
        assert_ne!(zero.assignment(), populated.assignment());
    }

    #[test]
    fn specialized_gadget_identities_lower_to_r1cs() {
        let mut composer = Composer::<R1cs>::initialized();

        let left = composer.append_witness(BlsScalar::from(3u64));
        let right = composer.append_witness(BlsScalar::from(5u64));
        composer.gate_mul(Constraint::new().mult(1).a(left).b(right));

        let range = composer.append_witness(BlsScalar::from(0xabu64));
        composer.component_range_bits::<8>(range);
        composer.append_logic_xor::<4>(left, right);

        let point_a = composer.append_point(GENERATOR_EXTENDED).unwrap();
        let point_b = composer.append_point(GENERATOR_EXTENDED).unwrap();
        composer.component_add_point(
            super::super::TorsionFreeWitnessPoint::new_unchecked(point_a),
            super::super::TorsionFreeWitnessPoint::new_unchecked(point_b),
        );

        let scalar = composer.append_witness(JubJubScalar::from(17u64));
        composer
            .component_mul_generator(scalar, GENERATOR_EXTENDED)
            .unwrap();

        let r1cs = composer.into_r1cs();
        assert!(r1cs.is_satisfied());
        assert!(r1cs.shape().constraints().len() > 100);
    }

    #[test]
    fn range_and_logic_constraints_reject_invalid_assignments() {
        let mut range = Composer::<R1cs>::initialized();
        let out_of_range = range.append_witness(BlsScalar::from(0x100u64));
        range.component_range_bits::<8>(out_of_range);
        assert!(!range.into_r1cs().is_satisfied());

        let mut logic = Composer::<R1cs>::initialized();
        let left = logic.append_witness(BlsScalar::from(0xabu64));
        let right = logic.append_witness(BlsScalar::from(0x3cu64));
        let output = logic.append_logic_xor::<4>(left, right);
        logic.witnesses[output.index()] += BlsScalar::one();
        assert!(!logic.into_r1cs().is_satisfied());
    }

    #[test]
    fn curve_constraints_reject_corrupted_outputs() {
        let mut variable_base = Composer::<R1cs>::initialized();
        let point_a = variable_base.append_point(GENERATOR_EXTENDED).unwrap();
        let point_b = variable_base.append_point(GENERATOR_EXTENDED).unwrap();
        let sum = variable_base.component_add_point(
            super::super::TorsionFreeWitnessPoint::new_unchecked(point_a),
            super::super::TorsionFreeWitnessPoint::new_unchecked(point_b),
        );
        variable_base.witnesses[sum.x().index()] += BlsScalar::one();
        assert!(!variable_base.into_r1cs().is_satisfied());

        let mut fixed_base = Composer::<R1cs>::initialized();
        let scalar = fixed_base.append_witness(JubJubScalar::from(17u64));
        let product = fixed_base
            .component_mul_generator(scalar, GENERATOR_EXTENDED)
            .unwrap();
        fixed_base.witnesses[product.y().index()] += BlsScalar::one();
        assert!(!fixed_base.into_r1cs().is_satisfied());
    }

    #[cfg(feature = "plonkish")]
    fn logic_composer<B: ComposerBackend>() -> (Composer<B>, Witness) {
        let mut composer = Composer::<B>::initialized();
        let left = composer.append_witness(BlsScalar::from(0xabu64));
        let right = composer.append_witness(BlsScalar::from(0x3cu64));
        let output = composer.append_logic_xor::<4>(left, right);
        (composer, output)
    }

    #[cfg(feature = "plonkish")]
    #[test]
    fn r1cs_and_plonkish_backends_agree_on_corrupted_logic_witnesses() {
        use super::super::Plonkish;

        let (plonkish, _) = logic_composer::<Plonkish>();
        let (r1cs, _) = logic_composer::<R1cs>();
        assert!(plonkish.is_satisfied());
        assert!(r1cs.to_r1cs().is_satisfied());

        let (mut plonkish, plonkish_output) = logic_composer::<Plonkish>();
        let (mut r1cs, r1cs_output) = logic_composer::<R1cs>();
        plonkish.witnesses[plonkish_output.index()] += BlsScalar::one();
        r1cs.witnesses[r1cs_output.index()] += BlsScalar::one();
        assert!(!plonkish.is_satisfied());
        assert!(!r1cs.to_r1cs().is_satisfied());
    }
}
