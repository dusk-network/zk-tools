// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Serial radix-two FFT domains used by the R1CS-to-QAP reduction.

use alloc::vec::Vec;
use core::ops::MulAssign;

use dusk_curves::bls12_381::{BlsScalar, GENERATOR, ROOT_OF_UNITY, TWO_ADACITY};

use crate::Error;

#[derive(Debug, Clone, Copy)]
/// Power-of-two multiplicative subgroup and its coset parameters.
pub(crate) struct EvaluationDomain {
    size: usize,
    log_size: u32,
    omega: BlsScalar,
    omega_inverse: BlsScalar,
    size_inverse: BlsScalar,
    coset_inverse: BlsScalar,
}

impl EvaluationDomain {
    /// Construct the smallest supported power-of-two domain at least as large
    /// as `minimum_size`.
    pub(crate) fn new(minimum_size: usize) -> Result<Self, Error> {
        let size = minimum_size.max(1).next_power_of_two();
        let log_size = size.trailing_zeros();
        if log_size >= TWO_ADACITY {
            return Err(Error::InvalidEvaluationDomain);
        }

        let mut omega = ROOT_OF_UNITY;
        for _ in log_size..TWO_ADACITY {
            omega = omega.square();
        }

        Ok(Self {
            size,
            log_size,
            omega,
            omega_inverse: omega.invert().ok_or(Error::InvalidEvaluationDomain)?,
            size_inverse: BlsScalar::from(size as u64)
                .invert()
                .ok_or(Error::InvalidEvaluationDomain)?,
            coset_inverse: GENERATOR.invert().ok_or(Error::InvalidEvaluationDomain)?,
        })
    }

    /// Return the number of elements in the domain.
    pub(crate) const fn size(self) -> usize {
        self.size
    }

    /// Evaluate coefficients over the subgroup, padding with zeros as needed.
    pub(crate) fn fft(self, coefficients: &[BlsScalar]) -> Vec<BlsScalar> {
        let mut values = coefficients.to_vec();
        values.resize(self.size, BlsScalar::zero());
        serial_fft(&mut values, self.omega, self.log_size);
        values
    }

    /// Interpolate subgroup evaluations into coefficient form.
    pub(crate) fn ifft(self, evaluations: &[BlsScalar]) -> Vec<BlsScalar> {
        let mut coefficients = evaluations.to_vec();
        coefficients.resize(self.size, BlsScalar::zero());
        serial_fft(&mut coefficients, self.omega_inverse, self.log_size);
        for coefficient in &mut coefficients {
            *coefficient *= self.size_inverse;
        }
        coefficients
    }

    /// Evaluate coefficients over the multiplicative-generator coset.
    pub(crate) fn coset_fft(self, coefficients: &[BlsScalar]) -> Vec<BlsScalar> {
        let mut values = coefficients.to_vec();
        distribute_powers(&mut values, GENERATOR);
        self.fft(&values)
    }

    /// Interpolate multiplicative-generator coset evaluations.
    pub(crate) fn coset_ifft(self, evaluations: &[BlsScalar]) -> Vec<BlsScalar> {
        let mut coefficients = self.ifft(evaluations);
        distribute_powers(&mut coefficients, self.coset_inverse);
        coefficients
    }

    /// Evaluate the domain vanishing polynomial `X^n - 1`.
    pub(crate) fn vanishing(self, point: BlsScalar) -> BlsScalar {
        point.pow(&[self.size as u64, 0, 0, 0]) - BlsScalar::one()
    }

    /// Evaluate every domain Lagrange basis polynomial at `point`.
    pub(crate) fn lagrange_at(self, point: BlsScalar) -> Result<Vec<BlsScalar>, Error> {
        let vanishing = self.vanishing(point);
        if vanishing == BlsScalar::zero() {
            return Err(Error::InvalidEvaluationDomain);
        }

        let common = vanishing * self.size_inverse;
        let mut omega = BlsScalar::one();
        let mut coefficients = Vec::with_capacity(self.size);
        for _ in 0..self.size {
            let inverse = (point - omega)
                .invert()
                .ok_or(Error::InvalidEvaluationDomain)?;
            coefficients.push(common * omega * inverse);
            omega *= self.omega;
        }
        Ok(coefficients)
    }
}

fn distribute_powers(values: &mut [BlsScalar], generator: BlsScalar) {
    let mut power = BlsScalar::one();
    for value in values {
        *value *= power;
        power *= generator;
    }
}

fn bitreverse(mut value: u32, bits: u32) -> u32 {
    let mut reversed = 0;
    for _ in 0..bits {
        reversed = (reversed << 1) | (value & 1);
        value >>= 1;
    }
    reversed
}

fn serial_fft(values: &mut [BlsScalar], omega: BlsScalar, log_size: u32) {
    let size = values.len() as u32;
    assert_eq!(size, 1 << log_size);

    for index in 0..size {
        let reversed = bitreverse(index, log_size);
        if index < reversed {
            values.swap(index as usize, reversed as usize);
        }
    }

    let mut width = 1;
    for _ in 0..log_size {
        let step = omega.pow(&[(size / (2 * width)) as u64, 0, 0, 0]);
        let mut block = 0;
        while block < size {
            let mut twiddle = BlsScalar::one();
            for offset in 0..width {
                let upper = (block + offset) as usize;
                let lower = (block + offset + width) as usize;
                let product = values[lower] * twiddle;
                values[lower] = values[upper] - product;
                values[upper] += product;
                twiddle.mul_assign(step);
            }
            block += 2 * width;
        }
        width *= 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evaluate(coefficients: &[BlsScalar], point: BlsScalar) -> BlsScalar {
        coefficients
            .iter()
            .rev()
            .fold(BlsScalar::zero(), |value, coefficient| {
                value * point + coefficient
            })
    }

    #[test]
    fn fft_round_trip() {
        let domain = EvaluationDomain::new(4).unwrap();
        let polynomial = vec![
            BlsScalar::from(1u64),
            BlsScalar::from(2u64),
            BlsScalar::from(3u64),
            BlsScalar::from(4u64),
        ];
        assert_eq!(domain.ifft(&domain.fft(&polynomial)), polynomial);
        assert_eq!(
            domain.coset_ifft(&domain.coset_fft(&polynomial)),
            polynomial
        );
    }

    #[test]
    fn fft_matches_direct_polynomial_evaluation() {
        for size in [1usize, 2, 4, 8, 16] {
            let domain = EvaluationDomain::new(size).unwrap();
            let coefficients: Vec<_> = (0..size)
                .map(|index| BlsScalar::from((index * index + 3) as u64))
                .collect();
            let evaluations = domain.fft(&coefficients);
            let mut point = BlsScalar::one();
            for evaluation in evaluations {
                assert_eq!(evaluation, evaluate(&coefficients, point));
                point *= domain.omega;
            }
        }
    }

    #[test]
    fn lagrange_coefficients_reconstruct_an_out_of_domain_evaluation() {
        let domain = EvaluationDomain::new(8).unwrap();
        let coefficients: Vec<_> = (1..=8).map(BlsScalar::from).collect();
        let evaluations = domain.fft(&coefficients);
        let point = GENERATOR;
        assert_ne!(domain.vanishing(point), BlsScalar::zero());
        let lagrange = domain.lagrange_at(point).unwrap();
        let reconstructed = evaluations
            .iter()
            .zip(lagrange)
            .fold(BlsScalar::zero(), |sum, (evaluation, basis)| {
                sum + evaluation * basis
            });
        assert_eq!(reconstructed, evaluate(&coefficients, point));
    }
}
