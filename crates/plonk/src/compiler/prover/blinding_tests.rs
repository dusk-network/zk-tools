// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_bytes::{DeserializableSlice, Serializable};
use dusk_curves::bls12_381::BlsScalar;
use rand::SeedableRng;
use rand::rngs::StdRng;

use super::Prover;
use crate::error::Error;
use crate::fft::{EvaluationDomain, Polynomial};
use crate::prelude::{
    Circuit, CircuitError, Compiler, Composer, ComposerBackend, Constraint,
    PlonkVersion, Plonkish, Proof, PublicParameters, Verifier,
};
use crate::transcript::TranscriptProtocol;

// Interpolate a linear mask from two evaluations.
fn reconstruct_linear_mask(
    candidate: &[BlsScalar],
    domain: &EvaluationDomain,
    point: BlsScalar,
    evaluation: BlsScalar,
    shifted_evaluation: BlsScalar,
) -> Polynomial {
    let shifted_point = point * domain.group_gen;
    let mut coeffs = domain.ifft(candidate);
    let raw = Polynomial::from_coefficients_vec(coeffs.clone());
    let mask_at_point = (evaluation - raw.evaluate(&point))
        * domain
            .evaluate_vanishing_polynomial(&point)
            .invert()
            .unwrap();
    let mask_at_shift = (shifted_evaluation - raw.evaluate(&shifted_point))
        * domain
            .evaluate_vanishing_polynomial(&shifted_point)
            .invert()
            .unwrap();
    let slope = (mask_at_shift - mask_at_point)
        * (shifted_point - point).invert().unwrap();
    let intercept = mask_at_point - slope * point;
    coeffs[0] -= intercept;
    coeffs[1] -= slope;
    coeffs.push(intercept);
    coeffs.push(slope);
    Polynomial::from_coefficients_vec(coeffs)
}

#[test]
fn quadratic_blinding_preserves_rows_and_prevents_linear_mask_recovery() {
    let mut rng = StdRng::seed_from_u64(39231);
    let pp = PublicParameters::setup(16, &mut rng).unwrap();
    let (commit_key, _) = pp.trim(16).unwrap();

    for size in [4, 16] {
        let domain = EvaluationDomain::new(size).unwrap();
        let witnesses: Vec<_> =
            (0..size).map(|i| BlsScalar::from(i as u64 + 5)).collect();
        for hiding_degree in [1, 2] {
            let blinded = Prover::blind_poly(
                &mut rng,
                &witnesses,
                hiding_degree,
                &domain,
            );
            assert_eq!(blinded.degree(), size + hiding_degree);
            for (row, witness) in domain.elements().zip(&witnesses) {
                assert_eq!(blinded.evaluate(&row), *witness);
            }

            let point = BlsScalar::from(12345);
            let reconstructed = reconstruct_linear_mask(
                &witnesses,
                &domain,
                point,
                blinded.evaluate(&point),
                blinded.evaluate(&(point * domain.group_gen)),
            );
            let matches = commit_key.commit(&reconstructed).unwrap()
                == commit_key.commit(&blinded).unwrap();
            // A quadratic mask retains randomness after two evaluations.
            assert_eq!(matches, hiding_degree == 1);
        }
    }
}

#[derive(Default)]
struct SecretCircuit(BlsScalar);

impl Circuit for SecretCircuit {
    fn circuit<B: ComposerBackend>(
        &self,
        composer: &mut Composer<B>,
    ) -> Result<(), CircuitError> {
        let secret = composer.append_witness(self.0);
        // Twenty total rows round to n = 32, and the compiler also trims
        // for 32. This exercises the exact n + 9 SRS boundary.
        for _ in 0..16 {
            composer.assert_equal(secret, secret);
        }
        Ok(())
    }
}

// Replay the public transcript only as far as the opening point.
fn opening_point(
    prover: &Prover,
    proof: &Proof,
    public_inputs: &[BlsScalar],
    version: PlonkVersion,
) -> BlsScalar {
    let mut transcript = prover.transcript_for_version(version);
    for input in public_inputs {
        transcript.append_scalar(b"pi", input);
    }
    transcript.append_commitment(b"a_comm", &proof.a_comm);
    transcript.append_commitment(b"b_comm", &proof.b_comm);
    transcript.append_commitment(b"c_comm", &proof.c_comm);
    transcript.append_commitment(b"d_comm", &proof.d_comm);
    let beta = transcript.challenge_scalar(b"beta");
    transcript.append_scalar(b"beta", &beta);
    transcript.challenge_scalar(b"gamma");
    transcript.append_commitment(b"z_comm", &proof.z_comm);
    for label in [
        b"alpha".as_slice(),
        b"range separation challenge",
        b"logic separation challenge",
        b"fixed base separation challenge",
        b"variable base separation challenge",
    ] {
        transcript.challenge_scalar(label);
    }
    transcript.append_commitment(b"t_low_comm", &proof.t_low_comm);
    transcript.append_commitment(b"t_mid_comm", &proof.t_mid_comm);
    transcript.append_commitment(b"t_high_comm", &proof.t_high_comm);
    transcript.append_commitment(b"t_fourth_comm", &proof.t_fourth_comm);
    transcript.challenge_scalar(b"z_challenge")
}

#[test]
fn shifted_wire_blinding_preserves_verifier_compatibility() {
    let mut rng = StdRng::seed_from_u64(8124);
    let pp = PublicParameters::setup(32, &mut rng).unwrap();
    let label = b"shifted-wire-blinding";
    let (prover, verifier) =
        Compiler::compile::<SecretCircuit>(&pp, label).unwrap();
    assert_eq!(prover.size, 32);
    assert_eq!(pp.max_degree(), prover.size + 9);
    assert_eq!(prover.commit_key.max_degree(), prover.size + 9);

    // Verifier serialization must be independent of commitment-key capacity.
    let composer = Composer::<Plonkish>::build(
        prover.constraints,
        &SecretCircuit::default(),
    )
    .unwrap();
    let old_commit_key = pp.commit_key.truncate(prover.size + 6).unwrap();
    let (old_prover, old_verifier) = Compiler::preprocess(
        label,
        old_commit_key,
        pp.opening_key.clone(),
        &composer,
    )
    .unwrap();
    assert_eq!(old_verifier.to_bytes(), verifier.to_bytes());
    let old_verifier =
        Verifier::try_from_bytes(old_verifier.to_bytes()).unwrap();
    assert!(matches!(
        old_prover.prove(&mut rng, &SecretCircuit(BlsScalar::from(7))),
        Err(Error::PolynomialDegreeTooLarge)
    ));

    let versions = [
        PlonkVersion::V3,
        #[cfg(feature = "legacy-proving")]
        PlonkVersion::V2,
    ];
    for version in versions {
        for secret in [7u64, 9] {
            let circuit = SecretCircuit(BlsScalar::from(secret));
            let (proof, public_inputs) = prover
                .prove_with_version(&mut rng, &circuit, version)
                .unwrap();
            let proof = Proof::from_slice(&proof.to_bytes()).unwrap();
            old_verifier
                .verify_with_version(&proof, &public_inputs, version)
                .unwrap();
            let point = opening_point(&prover, &proof, &public_inputs, version);
            let domain = EvaluationDomain::new(prover.size).unwrap();

            // Both candidates are valid witnesses for the same statement.
            // Neither candidate should reproduce any shifted-wire commitment
            // by solving for a linear mask, even when the guess is correct.
            for guess in [7u64, 9] {
                let candidate = Composer::<Plonkish>::build(
                    prover.constraints,
                    &SecretCircuit(BlsScalar::from(guess)),
                )
                .unwrap();
                let mut wires = vec![vec![BlsScalar::zero(); prover.size]; 3];
                for (i, gate) in candidate.gates().iter().enumerate() {
                    wires[0][i] = candidate[gate.a()];
                    wires[1][i] = candidate[gate.b()];
                    wires[2][i] = candidate[gate.d()];
                }
                let evaluations = &proof.evaluations;
                let openings = [
                    (evaluations.a_eval, evaluations.a_w_eval, proof.a_comm),
                    (evaluations.b_eval, evaluations.b_w_eval, proof.b_comm),
                    (evaluations.d_eval, evaluations.d_w_eval, proof.d_comm),
                ];
                for (wire, (evaluation, shifted_evaluation, commitment)) in
                    wires.iter().zip(openings)
                {
                    let reconstructed = reconstruct_linear_mask(
                        wire,
                        &domain,
                        point,
                        evaluation,
                        shifted_evaluation,
                    );
                    assert_ne!(
                        prover.commit_key.commit(&reconstructed).unwrap(),
                        commitment
                    );
                }
            }
        }
    }
}

#[derive(Default)]
struct PaddedCircuit {
    extra_rows: usize,
}

impl Circuit for PaddedCircuit {
    fn circuit<B: ComposerBackend>(
        &self,
        composer: &mut Composer<B>,
    ) -> Result<(), CircuitError> {
        for _ in 0..self.extra_rows {
            composer.assert_equal(Composer::<B>::ONE, Composer::<B>::ONE);
        }
        Ok(())
    }
}

#[test]
fn blinded_proofs_respect_exact_key_capacity_after_serialization() {
    let mut rng = StdRng::seed_from_u64(0xb11d);
    let versions = [
        PlonkVersion::V3,
        #[cfg(feature = "legacy-proving")]
        PlonkVersion::V2,
    ];

    // Include the minimum domain n = 4, padded circuits, and exact fills.
    // At n = 4 the honest degree bound 4n + 9 is closest to the quotient
    // rejection threshold 7n; a satisfied circuit must still be accepted.
    for (extra_rows, size) in [
        (0, 4),
        (1, 8),
        (4, 8),
        (5, 16),
        (12, 16),
        (16, 32),
        (28, 32),
    ] {
        let pp = PublicParameters::setup(size, &mut rng).unwrap();
        let pp = PublicParameters::from_slice(&pp.to_var_bytes()).unwrap();
        let circuit = PaddedCircuit { extra_rows };
        let (prover, verifier) =
            Compiler::compile_with_circuit(&pp, b"blinding-capacity", &circuit)
                .unwrap();
        assert_eq!(prover.size, size);

        // Setup and compilation must agree on the actual n + 9 boundary,
        // including after decoding, without requiring a larger setup.
        let prover = Prover::try_from_bytes(prover.to_bytes()).unwrap();
        let verifier = Verifier::try_from_bytes(verifier.to_bytes()).unwrap();
        assert_eq!(prover.commit_key.max_degree(), size + 9);

        for version in versions {
            let (proof, inputs) = prover
                .prove_with_version(&mut rng, &circuit, version)
                .unwrap();
            let proof = Proof::from_slice(&proof.to_bytes()).unwrap();
            verifier
                .verify_with_version(&proof, &inputs, version)
                .unwrap();

            // Decoding a key does not establish sufficient capacity.
            // Undersized keys must fail when used for proving.
            for allowance in [6, 8] {
                let mut short_prover = prover.clone();
                short_prover.commit_key =
                    pp.commit_key.truncate(size + allowance).unwrap();
                let short_prover =
                    Prover::try_from_bytes(short_prover.to_bytes()).unwrap();
                assert!(matches!(
                    short_prover
                        .prove_with_version(&mut rng, &circuit, version),
                    Err(Error::PolynomialDegreeTooLarge)
                ));
            }
        }
    }
}

#[test]
fn sufficiently_large_legacy_setup_and_prover_key_remain_usable() {
    let mut rng = StdRng::seed_from_u64(0x1e6ac7);
    let mut pp = PublicParameters::setup(32, &mut rng).unwrap();
    // Use a setup whose capacity exceeds this circuit's requirements.
    pp.commit_key = pp.commit_key.truncate(32 + 6).unwrap();
    let pp = PublicParameters::from_slice(&pp.to_var_bytes()).unwrap();
    let circuit = PaddedCircuit { extra_rows: 12 };
    let label = b"legacy-blinding-capacity";
    let (prover, verifier) =
        Compiler::compile_with_circuit(&pp, label, &circuit).unwrap();
    assert_eq!(prover.size, 16);
    assert_eq!(prover.commit_key.max_degree(), 16 + 9);

    // A serialized prover with extra capacity must remain usable.
    let composer =
        Composer::<Plonkish>::build(prover.constraints, &circuit).unwrap();
    let (old_prover, old_verifier) = Compiler::preprocess(
        label,
        pp.commit_key.clone(),
        pp.opening_key.clone(),
        &composer,
    )
    .unwrap();
    let old_prover = Prover::try_from_bytes(old_prover.to_bytes()).unwrap();
    assert_eq!(old_verifier.to_bytes(), verifier.to_bytes());
    let versions = [
        PlonkVersion::V3,
        #[cfg(feature = "legacy-proving")]
        PlonkVersion::V2,
    ];
    for version in versions {
        for prover in [&prover, &old_prover] {
            let (proof, inputs) = prover
                .prove_with_version(&mut rng, &circuit, version)
                .unwrap();
            old_verifier
                .verify_with_version(&proof, &inputs, version)
                .unwrap();
        }
    }
}

#[derive(Default)]
struct PublicSumCircuit {
    left: BlsScalar,
    right: BlsScalar,
    sum: BlsScalar,
}

impl Circuit for PublicSumCircuit {
    fn circuit<B: ComposerBackend>(
        &self,
        composer: &mut Composer<B>,
    ) -> Result<(), CircuitError> {
        let left = composer.append_witness(self.left);
        let right = composer.append_witness(self.right);
        let sum = composer.append_witness(self.sum);
        composer.append_gate(
            Constraint::new()
                .left(1)
                .right(1)
                .output(-BlsScalar::one())
                .a(left)
                .b(right)
                .c(sum),
        );
        composer.assert_equal_constant(sum, BlsScalar::zero(), Some(self.sum));
        Ok(())
    }
}

#[test]
fn fresh_blinding_preserves_statement_and_rejects_invalid_assignments() {
    let mut rng = StdRng::seed_from_u64(0xf2e5);
    let pp = PublicParameters::setup(32, &mut rng).unwrap();
    let (prover, verifier) =
        Compiler::compile::<PublicSumCircuit>(&pp, b"fresh-blinding").unwrap();
    assert_eq!(prover.size, 8);
    let versions = [
        PlonkVersion::V3,
        #[cfg(feature = "legacy-proving")]
        PlonkVersion::V2,
    ];

    for version in versions {
        for sum in [BlsScalar::zero(), BlsScalar::from(17)] {
            let circuit = PublicSumCircuit {
                left: BlsScalar::from(7),
                right: sum - BlsScalar::from(7),
                sum,
            };
            let (first, inputs) = prover
                .prove_with_version(&mut rng, &circuit, version)
                .unwrap();
            let (second, repeated_inputs) = prover
                .prove_with_version(&mut rng, &circuit, version)
                .unwrap();
            assert_eq!(inputs, vec![sum]);
            assert_eq!(inputs, repeated_inputs);
            verifier
                .verify_with_version(&first, &inputs, version)
                .unwrap();
            verifier
                .verify_with_version(&second, &inputs, version)
                .unwrap();

            // Fresh masks must affect every witness-bearing commitment,
            // including the once-opened c wire and the permutation product.
            for (first, second) in [
                (first.a_comm, second.a_comm),
                (first.b_comm, second.b_comm),
                (first.c_comm, second.c_comm),
                (first.d_comm, second.d_comm),
                (first.z_comm, second.z_comm),
                (first.t_low_comm, second.t_low_comm),
                (first.t_mid_comm, second.t_mid_comm),
                (first.t_high_comm, second.t_high_comm),
                (first.t_fourth_comm, second.t_fourth_comm),
            ] {
                assert_ne!(first, second);
            }

            let wrong_inputs = [sum + BlsScalar::one()];
            assert!(
                verifier
                    .verify_with_version(&first, &wrong_inputs, version)
                    .is_err()
            );

            let invalid = PublicSumCircuit {
                left: circuit.left + BlsScalar::one(),
                ..circuit
            };
            // Also exercise the degree-based satisfaction check with a key
            // large enough to commit to an invalid quotient's chunks.
            for oversized in [false, true] {
                let mut prover = prover.clone();
                if oversized {
                    prover.commit_key = pp.commit_key.clone();
                }
                assert!(matches!(
                    prover.prove_with_version(&mut rng, &invalid, version),
                    Err(Error::CircuitUnsatisfied)
                ));
            }
        }
    }
}
