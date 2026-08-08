// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! R1CS-to-QAP instance and witness reduction.

use alloc::{vec, vec::Vec};

use dusk_curves::bls12_381::BlsScalar;
use dusk_zk_composer::{LinearCombination, R1csCircuit, R1csShape};
#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

use crate::{Error, fft::EvaluationDomain};

/// QAP variable-polynomial evaluations and vanishing value at setup point tau.
pub(crate) struct QapAtTau {
    /// Evaluations of the left R1CS matrix polynomials.
    pub(crate) u: Vec<BlsScalar>,
    /// Evaluations of the right R1CS matrix polynomials.
    pub(crate) v: Vec<BlsScalar>,
    /// Evaluations of the output R1CS matrix polynomials.
    pub(crate) w: Vec<BlsScalar>,
    /// Evaluation of the domain vanishing polynomial.
    pub(crate) vanishing: BlsScalar,
}

#[cfg(feature = "zeroize")]
impl Zeroize for QapAtTau {
    fn zeroize(&mut self) {
        self.u.zeroize();
        self.v.zeroize();
        self.w.zeroize();
        self.vanishing.zeroize();
    }
}

#[cfg(feature = "zeroize")]
impl Drop for QapAtTau {
    fn drop(&mut self) {
        self.zeroize();
    }
}

#[cfg(feature = "zeroize")]
fn discard_scalars(mut scalars: Vec<BlsScalar>) {
    scalars.zeroize();
}

#[cfg(not(feature = "zeroize"))]
fn discard_scalars(_: Vec<BlsScalar>) {}

#[cfg(feature = "zeroize")]
fn discard_scalar(scalar: &mut BlsScalar) {
    scalar.zeroize();
}

#[cfg(not(feature = "zeroize"))]
fn discard_scalar(_: &mut BlsScalar) {}

/// Evaluate the QAP instance polynomials at a point outside the domain.
pub(crate) fn evaluate_shape_at_tau(
    shape: &R1csShape,
    domain: EvaluationDomain,
    mut tau: BlsScalar,
) -> Result<QapAtTau, Error> {
    if shape.constraints().len() > domain.size() || shape.public_inputs() >= shape.variables() {
        return Err(Error::InvalidCircuitShape);
    }

    let lagrange = domain.lagrange_at(tau)?;
    let mut u = vec![BlsScalar::zero(); shape.variables()];
    let mut v = vec![BlsScalar::zero(); shape.variables()];
    let mut w = vec![BlsScalar::zero(); shape.variables()];

    for (row, constraint) in shape.constraints().iter().enumerate() {
        accumulate(&mut u, constraint.a(), lagrange[row]);
        accumulate(&mut v, constraint.b(), lagrange[row]);
        accumulate(&mut w, constraint.c(), lagrange[row]);
    }

    let qap = QapAtTau {
        u,
        v,
        w,
        vanishing: domain.vanishing(tau),
    };
    discard_scalars(lagrange);
    discard_scalar(&mut tau);
    Ok(qap)
}

fn accumulate(destination: &mut [BlsScalar], combination: &LinearCombination, factor: BlsScalar) {
    for (variable, coefficient) in combination.terms() {
        destination[variable.index()] += *coefficient * factor;
    }
}

fn evaluate(combination: &LinearCombination, assignment: &[BlsScalar]) -> Result<BlsScalar, Error> {
    combination
        .terms()
        .try_fold(BlsScalar::zero(), |sum, (variable, coefficient)| {
            assignment
                .get(variable.index())
                .map(|value| sum + *value * coefficient)
                .ok_or(Error::InvalidCircuitShape)
        })
}

/// Construct coefficients of `(A(X)B(X) - C(X)) / Z(X)` for an assignment.
pub(crate) fn quotient(
    circuit: &R1csCircuit,
    domain: EvaluationDomain,
) -> Result<Vec<BlsScalar>, Error> {
    let assignment = circuit.assignment().values();
    if assignment.len() != circuit.shape().variables()
        || circuit.shape().constraints().len() > domain.size()
    {
        return Err(Error::InvalidCircuitShape);
    }

    let mut a = vec![BlsScalar::zero(); domain.size()];
    let mut b = vec![BlsScalar::zero(); domain.size()];
    let mut c = vec![BlsScalar::zero(); domain.size()];

    for (row, constraint) in circuit.shape().constraints().iter().enumerate() {
        a[row] = evaluate(constraint.a(), assignment)?;
        b[row] = evaluate(constraint.b(), assignment)?;
        c[row] = evaluate(constraint.c(), assignment)?;
    }

    let a_coefficients = domain.ifft(&a);
    let b_coefficients = domain.ifft(&b);
    let c_coefficients = domain.ifft(&c);
    discard_scalars(a);
    discard_scalars(b);
    discard_scalars(c);

    let a = domain.coset_fft(&a_coefficients);
    let b = domain.coset_fft(&b_coefficients);
    let c = domain.coset_fft(&c_coefficients);
    discard_scalars(a_coefficients);
    discard_scalars(b_coefficients);
    discard_scalars(c_coefficients);
    let inverse_vanishing = domain
        .vanishing(dusk_curves::bls12_381::GENERATOR)
        .invert()
        .ok_or(Error::InvalidEvaluationDomain)?;
    let quotient_evaluations: Vec<_> = a
        .iter()
        .zip(&b)
        .zip(&c)
        .map(|((a, b), c)| (*a * b - c) * inverse_vanishing)
        .collect();
    discard_scalars(a);
    discard_scalars(b);
    discard_scalars(c);

    let quotient = domain.coset_ifft(&quotient_evaluations);
    discard_scalars(quotient_evaluations);
    Ok(quotient)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dusk_zk_composer::{Circuit, Composer, ComposerBackend, Constraint, R1cs};

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
        ) -> Result<(), dusk_zk_composer::Error> {
            let left = composer.append_witness(self.left);
            let right = composer.append_witness(self.right);
            let product = composer.gate_mul(Constraint::new().mult(1).a(left).b(right));
            let result = composer.append_public(self.result);
            composer.assert_equal(product, result);
            Ok(())
        }
    }

    fn evaluate_polynomial(coefficients: &[BlsScalar], point: BlsScalar) -> BlsScalar {
        coefficients
            .iter()
            .rev()
            .fold(BlsScalar::zero(), |value, coefficient| {
                value * point + coefficient
            })
    }

    fn inner_product(values: &[BlsScalar], assignment: &[BlsScalar]) -> BlsScalar {
        values
            .iter()
            .zip(assignment)
            .fold(BlsScalar::zero(), |sum, (value, assigned)| {
                sum + value * assigned
            })
    }

    #[test]
    fn qap_witness_polynomial_satisfies_the_divisibility_identity() {
        let circuit = ProductCircuit {
            left: BlsScalar::from(9u64),
            right: BlsScalar::from(11u64),
            result: BlsScalar::from(99u64),
        };
        let mut composer = Composer::<R1cs>::initialized();
        circuit.circuit(&mut composer).unwrap();
        let circuit = composer.into_r1cs();
        assert!(circuit.is_satisfied());

        let domain = EvaluationDomain::new(circuit.shape().constraints().len()).unwrap();
        let tau = dusk_curves::bls12_381::GENERATOR;
        assert_ne!(domain.vanishing(tau), BlsScalar::zero());
        let instance = evaluate_shape_at_tau(circuit.shape(), domain, tau).unwrap();
        let quotient = quotient(&circuit, domain).unwrap();
        assert_eq!(quotient.len(), domain.size());
        assert_eq!(quotient.last(), Some(&BlsScalar::zero()));

        let assignment = circuit.assignment().values();
        let a = inner_product(&instance.u, assignment);
        let b = inner_product(&instance.v, assignment);
        let c = inner_product(&instance.w, assignment);
        let h = evaluate_polynomial(&quotient, tau);
        assert_eq!(a * b - c, instance.vanishing * h);
    }
}
