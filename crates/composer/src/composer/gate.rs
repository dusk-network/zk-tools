// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_curves::bls12_381::BlsScalar;

use super::{Constraint, Selector, WiredWitness, Witness};

/// Represents a gate with its associated wire data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gate {
    // Selectors
    /// Multiplier selector
    pub(crate) q_m: BlsScalar,
    /// Left wire selector
    pub(crate) q_l: BlsScalar,
    /// Right wire selector
    pub(crate) q_r: BlsScalar,
    /// Output wire selector
    pub(crate) q_o: BlsScalar,
    /// Fourth wire selector
    pub(crate) q_f: BlsScalar,
    /// Constant wire selector
    pub(crate) q_c: BlsScalar,
    /// Arithmetic wire selector
    pub(crate) q_arith: BlsScalar,
    /// Range selector
    pub(crate) q_range: BlsScalar,
    /// Logic selector
    pub(crate) q_logic: BlsScalar,
    /// Fixed base group addition selector
    pub(crate) q_fixed_group_add: BlsScalar,
    /// Variable base group addition selector
    pub(crate) q_variable_group_add: BlsScalar,

    /// Left wire witness.
    pub(crate) a: Witness,
    /// Right wire witness.
    pub(crate) b: Witness,
    /// Output wire witness.
    pub(crate) c: Witness,
    /// Fourth wire witness.
    pub(crate) d: Witness,
}

impl Gate {
    pub(crate) fn from_constraint(constraint: &Constraint) -> Self {
        Self {
            q_m: *constraint.coeff(Selector::Multiplication),
            q_l: *constraint.coeff(Selector::Left),
            q_r: *constraint.coeff(Selector::Right),
            q_o: *constraint.coeff(Selector::Output),
            q_f: *constraint.coeff(Selector::Fourth),
            q_c: *constraint.coeff(Selector::Constant),
            q_arith: *constraint.coeff(Selector::Arithmetic),
            q_range: *constraint.coeff(Selector::Range),
            q_logic: *constraint.coeff(Selector::Logic),
            q_fixed_group_add: *constraint.coeff(Selector::GroupAddFixedBase),
            q_variable_group_add: *constraint
                .coeff(Selector::GroupAddVariableBase),
            a: constraint.witness(WiredWitness::A),
            b: constraint.witness(WiredWitness::B),
            c: constraint.witness(WiredWitness::C),
            d: constraint.witness(WiredWitness::D),
        }
    }

    /// Multiplication selector.
    pub const fn q_m(&self) -> &BlsScalar {
        &self.q_m
    }

    /// Left-wire selector.
    pub const fn q_l(&self) -> &BlsScalar {
        &self.q_l
    }

    /// Right-wire selector.
    pub const fn q_r(&self) -> &BlsScalar {
        &self.q_r
    }

    /// Output-wire selector.
    pub const fn q_o(&self) -> &BlsScalar {
        &self.q_o
    }

    /// Fourth-wire selector.
    pub const fn q_f(&self) -> &BlsScalar {
        &self.q_f
    }

    /// Constant selector.
    pub const fn q_c(&self) -> &BlsScalar {
        &self.q_c
    }

    /// Arithmetic-gate selector.
    pub const fn q_arith(&self) -> &BlsScalar {
        &self.q_arith
    }

    /// Range-gate selector.
    pub const fn q_range(&self) -> &BlsScalar {
        &self.q_range
    }

    /// Logic-gate selector.
    pub const fn q_logic(&self) -> &BlsScalar {
        &self.q_logic
    }

    /// Fixed-base group-addition selector.
    pub const fn q_fixed_group_add(&self) -> &BlsScalar {
        &self.q_fixed_group_add
    }

    /// Variable-base group-addition selector.
    pub const fn q_variable_group_add(&self) -> &BlsScalar {
        &self.q_variable_group_add
    }

    /// Left-wire witness.
    pub const fn a(&self) -> Witness {
        self.a
    }

    /// Right-wire witness.
    pub const fn b(&self) -> Witness {
        self.b
    }

    /// Output-wire witness.
    pub const fn c(&self) -> Witness {
        self.c
    }

    /// Fourth-wire witness.
    pub const fn d(&self) -> Witness {
        self.d
    }

    /// All four wire witnesses in `(a, b, c, d)` order.
    pub const fn wires(&self) -> [Witness; 4] {
        [self.a, self.b, self.c, self.d]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_is_copy_clone_and_eq() {
        let gate = Gate {
            q_m: BlsScalar::from(1u64),
            q_l: BlsScalar::from(2u64),
            q_r: BlsScalar::from(3u64),
            q_o: BlsScalar::from(4u64),
            q_f: BlsScalar::from(5u64),
            q_c: BlsScalar::from(6u64),
            q_arith: BlsScalar::one(),
            q_range: BlsScalar::zero(),
            q_logic: BlsScalar::zero(),
            q_fixed_group_add: BlsScalar::zero(),
            q_variable_group_add: BlsScalar::zero(),
            a: Witness::ZERO,
            b: Witness::ONE,
            c: Witness::new(2),
            d: Witness::new(3),
        };

        // Copy
        let gate_copy = gate;
        assert_eq!(gate, gate_copy);

        // Clone
        let gate_clone = gate_copy.clone();
        assert_eq!(gate_copy, gate_clone);

        // Debug fmt should not panic
        let _ = format!("{gate_clone:?}");
    }

    #[test]
    fn gate_partial_eq_compares_fields() {
        let a = Gate {
            q_m: BlsScalar::from(1u64),
            q_l: BlsScalar::from(2u64),
            q_r: BlsScalar::from(3u64),
            q_o: BlsScalar::from(4u64),
            q_f: BlsScalar::from(5u64),
            q_c: BlsScalar::from(6u64),
            q_arith: BlsScalar::one(),
            q_range: BlsScalar::zero(),
            q_logic: BlsScalar::zero(),
            q_fixed_group_add: BlsScalar::zero(),
            q_variable_group_add: BlsScalar::zero(),
            a: Witness::ZERO,
            b: Witness::ONE,
            c: Witness::new(2),
            d: Witness::new(3),
        };

        let mut b = a;
        assert_eq!(a, b);

        // Flip one field and ensure inequality.
        b.q_c = BlsScalar::from(7u64);
        assert_ne!(a, b);
    }
}
