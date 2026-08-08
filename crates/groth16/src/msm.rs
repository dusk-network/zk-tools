// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Backend-portable multi-scalar multiplication helpers.

use alloc::{vec, vec::Vec};

use dusk_curves::bls12_381::{
    BlsScalar, G1Affine, G1Projective, G2Affine, G2Projective, msm_variable_base,
};

use crate::Error;

/// Compute a G1 MSM after checking query and scalar lengths.
pub(crate) fn g1(points: &[G1Affine], scalars: &[BlsScalar]) -> Result<G1Projective, Error> {
    if points.len() != scalars.len() {
        return Err(Error::InvalidQueryLength);
    }
    Ok(msm_variable_base(points, scalars))
}

/// Compute a G2 bucket MSM after checking query and scalar lengths.
pub(crate) fn g2(points: &[G2Affine], scalars: &[BlsScalar]) -> Result<G2Projective, Error> {
    if points.len() != scalars.len() {
        return Err(Error::InvalidQueryLength);
    }

    if points.is_empty() {
        return Ok(G2Projective::identity());
    }

    // A portable bucket MSM is used because dusk-curves currently exposes a
    // backend-independent optimized MSM only for G1.
    let window = match points.len() {
        0..=4 => 2,
        5..=32 => 4,
        33..=256 => 6,
        _ => 8,
    };
    let windows = 255usize.div_ceil(window);
    let bases: Vec<_> = points.iter().copied().map(G2Projective::from).collect();
    let scalar_bytes: Vec<_> = scalars.iter().map(BlsScalar::to_bytes).collect();
    let mut result = G2Projective::identity();

    for window_index in (0..windows).rev() {
        for _ in 0..window {
            result = result.double();
        }

        let mut buckets = vec![G2Projective::identity(); (1usize << window) - 1];
        let bit_offset = window_index * window;
        for (base, bytes) in bases.iter().zip(&scalar_bytes) {
            let value = window_value(bytes, bit_offset, window);
            if value != 0 {
                buckets[value - 1] += base;
            }
        }

        let mut running = G2Projective::identity();
        for bucket in buckets.iter().rev() {
            running += bucket;
            result += running;
        }
    }

    Ok(result)
}

fn window_value(bytes: &[u8; 32], offset: usize, width: usize) -> usize {
    (0..width).fold(0, |value, bit| {
        let index = offset + bit;
        if index >= 255 {
            value
        } else {
            value | (((bytes[index / 8] >> (index % 8)) & 1) as usize) << bit
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_g2_msm(length: usize) {
        let points: Vec<_> = (0..length)
            .map(|index| {
                G2Affine::from(G2Affine::generator() * BlsScalar::from((index % 17 + 1) as u64))
            })
            .collect();
        let scalars: Vec<_> = (0..length)
            .map(|index| {
                if index % 3 == 0 {
                    -BlsScalar::one()
                } else if index % 3 == 1 {
                    BlsScalar::zero()
                } else {
                    BlsScalar::from((index + 20) as u64)
                }
            })
            .collect();
        let expected = points
            .iter()
            .zip(&scalars)
            .fold(G2Projective::identity(), |sum, (point, scalar)| {
                sum + point * scalar
            });
        assert_eq!(g2(&points, &scalars).unwrap(), expected);
    }

    #[test]
    fn g2_bucket_msm_matches_naive_sum_at_every_window_threshold() {
        for length in [0, 1, 4, 5, 32, 33, 256, 257] {
            check_g2_msm(length);
        }
    }

    #[test]
    fn msm_rejects_inconsistent_lengths() {
        assert_eq!(
            g1(&[G1Affine::generator()], &[]),
            Err(Error::InvalidQueryLength)
        );
        assert_eq!(
            g2(&[G2Affine::generator()], &[]),
            Err(Error::InvalidQueryLength)
        );
    }
}
