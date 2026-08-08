// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Canonical algebraic identities enforced by Turbo Plonkish gates.
//!
//! The evaluator is generic over
//! [`IdentityElement`](crate::identities::IdentityElement). A proof backend can
//! use scalar values for satisfiability checks or a symbolic expression type
//! for lowering the exact same identities into its native constraint system.

use core::ops::{Add, Mul, Sub};

use dusk_curves::bls12_381::BlsScalar;
use dusk_jubjub::EDWARDS_D;

use crate::composer::Gate;

/// Number of independently enforced Turbo Plonkish identities.
pub const IDENTITY_COUNT: usize = 17;

/// Human-readable names, index-aligned with [`evaluate`].
#[cfg(feature = "plonkish")]
pub const IDENTITY_NAMES: [&str; IDENTITY_COUNT] = [
    "arithmetic",
    "range delta c/d",
    "range delta b/c",
    "range delta a/b",
    "range accumulator",
    "logic left quad",
    "logic right quad",
    "logic output quad",
    "logic product",
    "logic relation",
    "fixed-base bit consistency",
    "fixed-base xy consistency",
    "fixed-base x accumulator",
    "fixed-base y accumulator",
    "variable-base xy consistency",
    "variable-base x accumulator",
    "variable-base y accumulator",
];

/// Value or symbolic expression accepted by the identity evaluator.
pub trait IdentityElement:
    Clone
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self, Output = Self>
{
    /// Embed a scalar-field constant.
    fn constant(value: BlsScalar) -> Self;
}

impl IdentityElement for BlsScalar {
    fn constant(value: BlsScalar) -> Self {
        value
    }
}

/// Values or expressions carried by a Plonkish row's four wires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireValues<E> {
    /// Left wire.
    pub a: E,
    /// Right wire.
    pub b: E,
    /// Output wire.
    pub c: E,
    /// Fourth wire.
    pub d: E,
}

impl<E> WireValues<E> {
    /// Construct a wire row in `(a, b, c, d)` order.
    pub const fn new(a: E, b: E, c: E, d: E) -> Self {
        Self { a, b, c, d }
    }
}

fn scalar<E: IdentityElement>(value: BlsScalar) -> E {
    E::constant(value)
}

fn small<E: IdentityElement>(value: u64) -> E {
    scalar(BlsScalar::from(value))
}

fn square<E: IdentityElement>(value: &E) -> E {
    value.clone() * value.clone()
}

fn delta<E: IdentityElement>(value: E) -> E {
    value.clone()
        * (value.clone() - small::<E>(1))
        * (value.clone() - small::<E>(2))
        * (value - small::<E>(3))
}

fn logic_relation<E: IdentityElement>(
    a: &E,
    b: &E,
    product: &E,
    output: &E,
    q_c: &E,
) -> E {
    let sum = a.clone() + b.clone();
    let f = product.clone()
        * (product.clone()
            * (small::<E>(4) * product.clone() - small::<E>(18) * sum.clone()
                + small::<E>(81))
            + small::<E>(18) * (square(a) + square(b))
            - small::<E>(81) * sum.clone()
            + small::<E>(83));
    let e = small::<E>(3) * (sum.clone() + output.clone()) - small::<E>(2) * f;
    let b_term =
        q_c.clone() * (small::<E>(9) * output.clone() - small::<E>(3) * sum);

    b_term + e
}

/// Evaluate all independent identities for one gate row.
///
/// `current` is the row selected by `gate`; `next` supplies the cyclic shifted
/// wire values used by range, logic, and curve gates. A backend is responsible
/// for applying the PLONKish padded-domain rotation rule when constructing
/// `next`. `public_input` is the sparse public value assigned to this row, or
/// zero when the row has no public input.
pub fn evaluate<E: IdentityElement>(
    gate: &Gate,
    current: WireValues<E>,
    next: WireValues<E>,
    public_input: E,
) -> [E; IDENTITY_COUNT] {
    let qm: E = scalar(*gate.q_m());
    let ql: E = scalar(*gate.q_l());
    let qr: E = scalar(*gate.q_r());
    let qo: E = scalar(*gate.q_o());
    let qf: E = scalar(*gate.q_f());
    let qc: E = scalar(*gate.q_c());
    let qarith: E = scalar(*gate.q_arith());
    let qrange: E = scalar(*gate.q_range());
    let qlogic: E = scalar(*gate.q_logic());
    let qfixed: E = scalar(*gate.q_fixed_group_add());
    let qvariable: E = scalar(*gate.q_variable_group_add());

    let a = current.a;
    let b = current.b;
    let c = current.c;
    let d = current.d;
    let a_w = next.a;
    let b_w = next.b;
    let d_w = next.d;

    let arithmetic = (qm * a.clone() * b.clone()
        + ql.clone() * a.clone()
        + qr.clone() * b.clone()
        + qo * c.clone()
        + qf * d.clone()
        + qc.clone())
        * qarith
        + public_input;

    let four: E = small(4);
    let range = [
        delta(c.clone() - four.clone() * d.clone()) * qrange.clone(),
        delta(b.clone() - four.clone() * c.clone()) * qrange.clone(),
        delta(a.clone() - four.clone() * b.clone()) * qrange.clone(),
        delta(d_w.clone() - four.clone() * a.clone()) * qrange,
    ];

    let left_quad = a_w.clone() - four.clone() * a.clone();
    let right_quad = b_w.clone() - four.clone() * b.clone();
    let output_quad = d_w.clone() - four * d.clone();
    let logic = [
        delta(left_quad.clone()) * qlogic.clone(),
        delta(right_quad.clone()) * qlogic.clone(),
        delta(output_quad.clone()) * qlogic.clone(),
        (c.clone() - left_quad.clone() * right_quad.clone()) * qlogic.clone(),
        logic_relation(&left_quad, &right_quad, &c, &output_quad, &qc) * qlogic,
    ];

    let bit = d_w.clone() - d.clone() - d.clone();
    let bit_consistency = bit.clone()
        * (bit.clone() - small::<E>(1))
        * (bit.clone() + small::<E>(1));
    let y_alpha = square(&bit) * (qr - small::<E>(1)) + small::<E>(1);
    let x_alpha = ql * bit.clone();
    let edwards_d: E = scalar(EDWARDS_D);
    let fixed = [
        bit_consistency * qfixed.clone(),
        (bit.clone() * qc - c.clone()) * qfixed.clone(),
        (a_w.clone()
            + a_w.clone()
                * c.clone()
                * a.clone()
                * b.clone()
                * edwards_d.clone()
            - (a.clone() * y_alpha.clone() + b.clone() * x_alpha.clone()))
            * qfixed.clone(),
        (b_w.clone()
            - b_w.clone()
                * c.clone()
                * a.clone()
                * b.clone()
                * edwards_d.clone()
            - (b.clone() * y_alpha + a.clone() * x_alpha))
            * qfixed,
    ];

    let x1_y2 = d_w;
    let y1_x2 = b.clone() * c.clone();
    let variable = [
        (a.clone() * d.clone() - x1_y2.clone()) * qvariable.clone(),
        (x1_y2.clone() + y1_x2.clone()
            - (a_w.clone()
                + a_w * scalar(EDWARDS_D) * x1_y2.clone() * y1_x2.clone()))
            * qvariable.clone(),
        (b.clone() * d + a * c
            - (b_w.clone() - b_w * scalar(EDWARDS_D) * x1_y2 * y1_x2))
            * qvariable,
    ];

    [
        arithmetic,
        range[0].clone(),
        range[1].clone(),
        range[2].clone(),
        range[3].clone(),
        logic[0].clone(),
        logic[1].clone(),
        logic[2].clone(),
        logic[3].clone(),
        logic[4].clone(),
        fixed[0].clone(),
        fixed[1].clone(),
        fixed[2].clone(),
        fixed[3].clone(),
        variable[0].clone(),
        variable[1].clone(),
        variable[2].clone(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composer::{Constraint, Gate, Witness};

    #[test]
    fn arithmetic_identity_uses_public_input_outside_selector() {
        let constraint = Constraint::arithmetic(
            &Constraint::new()
                .mult(1)
                .left(2)
                .right(3)
                .output(-BlsScalar::one())
                .a(Witness::ZERO)
                .b(Witness::ONE),
        );
        let gate = Gate::from_constraint(&constraint);
        let current = WireValues::new(
            BlsScalar::from(2u64),
            BlsScalar::from(3u64),
            BlsScalar::from(19u64),
            BlsScalar::zero(),
        );
        let next = WireValues::new(
            BlsScalar::zero(),
            BlsScalar::zero(),
            BlsScalar::zero(),
            BlsScalar::zero(),
        );
        let identities = evaluate(&gate, current, next, BlsScalar::zero());

        assert!(
            identities
                .into_iter()
                .all(|value| value == BlsScalar::zero())
        );
    }
}
