// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Circuit-specific setup, proving, and verification.

use alloc::vec::Vec;

use dusk_curves::bls12_381::{
    BlsScalar, G1Affine, G1Projective, G2Affine, G2Prepared, G2Projective, Gt,
    multi_miller_loop_result,
};
use dusk_zk_composer::{Circuit, Composer, R1cs, R1csCircuit};
use ff::Field;
use rand_core::{CryptoRng, RngCore};
#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

use crate::{
    Error, PreparedVerifyingKey, Proof, ProvingKey, VerifyingKey, fft::EvaluationDomain, msm, qap,
};

/// Compile Composer circuits into circuit-specific Groth16 keys.
pub struct Compiler;

impl Compiler {
    /// Perform a single-party circuit-specific trusted setup.
    ///
    /// The R1CS shape is synthesized from `C::default()`. Its placeholder
    /// assignment need not satisfy the circuit; setup uses only the emitted
    /// topology. Every later circuit passed to [`Prover::prove`] must emit
    /// exactly that shape, so topology must not depend on witness values.
    ///
    /// # Security
    ///
    /// The caller must ensure that the process and RNG are trustworthy and
    /// that no setup randomness is retained. This is not an MPC ceremony.
    /// Enabling `zeroize` provides best-effort clearing of internal trapdoors,
    /// but the caller remains responsible for the RNG and its state.
    ///
    /// # Errors
    ///
    /// Returns an error if default-circuit synthesis fails or the constraint
    /// count exceeds the scalar field's supported evaluation domains.
    pub fn trusted_setup<C, R>(rng: &mut R) -> Result<(Prover, Verifier), Error>
    where
        C: Circuit,
        R: RngCore + CryptoRng,
    {
        let circuit = compose(&C::default())?;
        let (proving_key, verifying_key) = setup(rng, circuit.shape())?;
        Ok((
            Prover { key: proving_key },
            Verifier {
                key: verifying_key.prepare(),
            },
        ))
    }
}

/// Groth16 prover bound to one circuit shape.
#[derive(Clone, Debug)]
pub struct Prover {
    key: ProvingKey,
}

impl Prover {
    /// Construct a prover from an existing circuit-specific key.
    ///
    /// Keys decoded with [`ProvingKey::try_from_bytes`] have already received
    /// structural validation. This constructor does not prove that an
    /// independently supplied key was honestly generated.
    pub fn from_key(key: ProvingKey) -> Self {
        Self { key }
    }

    /// Access the circuit-specific proving key.
    pub fn key(&self) -> &ProvingKey {
        &self.key
    }

    /// Return an owned copy of the underlying proving key.
    pub fn into_key(self) -> ProvingKey {
        self.key
    }

    /// Generate a randomized proof and return its public inputs.
    ///
    /// Public inputs are returned in Composer emission order. The method
    /// checks the complete R1CS shape, assignment satisfaction, canonical
    /// evaluation domain, and QAP quotient degree before constructing a proof.
    ///
    /// # Errors
    ///
    /// Returns an error for circuit synthesis failure, a setup/proving shape
    /// mismatch, an unsatisfied assignment, malformed key query lengths, or an
    /// unsupported evaluation domain.
    pub fn prove<C, R>(&self, rng: &mut R, circuit: &C) -> Result<(Proof, Vec<BlsScalar>), Error>
    where
        C: Circuit,
        R: RngCore + CryptoRng,
    {
        let circuit = compose(circuit)?;
        if circuit.shape().digest() != self.key.shape_digest
            || circuit.shape().variables() != self.key.variables
            || circuit.shape().public_inputs() != self.key.public_inputs
        {
            return Err(Error::InvalidCircuitShape);
        }
        if !circuit.is_satisfied() {
            return Err(Error::CircuitUnsatisfied);
        }

        let assignment = circuit.assignment().values();
        let domain = EvaluationDomain::new(circuit.shape().constraints().len())?;
        if domain.size() != self.key.domain_size {
            return Err(Error::InvalidCircuitShape);
        }
        let quotient = qap::quotient(&circuit, domain)?;
        if quotient.len() != self.key.h_query.len() + 1
            || quotient.last().copied() != Some(BlsScalar::zero())
        {
            return Err(Error::InvalidCircuitShape);
        }
        let private_start = 1 + self.key.public_inputs;
        let private = assignment
            .get(private_start..)
            .ok_or(Error::InvalidCircuitShape)?;

        let r = BlsScalar::random(&mut *rng);
        let s = BlsScalar::random(&mut *rng);
        let a = G1Projective::from(self.key.alpha_g1)
            + msm::g1(&self.key.a_query, assignment)?
            + self.key.delta_g1 * r;
        let b = G2Projective::from(self.key.beta_g2)
            + msm::g2(&self.key.b_g2_query, assignment)?
            + self.key.delta_g2 * s;
        let b_g1 = G1Projective::from(self.key.beta_g1)
            + msm::g1(&self.key.b_g1_query, assignment)?
            + self.key.delta_g1 * s;
        let h = &quotient[..self.key.h_query.len()];
        let c = msm::g1(&self.key.private_query, private)?
            + msm::g1(&self.key.h_query, h)?
            + a * s
            + b_g1 * r
            - self.key.delta_g1 * (r * s);

        let public_inputs = assignment[1..private_start].to_vec();
        discard_scalars(quotient);
        Ok((
            Proof {
                a: G1Affine::from(a),
                b: G2Affine::from(b),
                c: G1Affine::from(c),
            },
            public_inputs,
        ))
    }
}

/// Groth16 verifier bound to one circuit shape.
#[derive(Clone, Debug)]
pub struct Verifier {
    key: PreparedVerifyingKey,
}

impl Verifier {
    /// Construct a verifier and precompute its fixed pairing terms.
    ///
    /// This constructor assumes `key` has already received structural
    /// validation or came directly from [`Compiler::trusted_setup`].
    pub fn from_key(key: VerifyingKey) -> Self {
        Self { key: key.prepare() }
    }

    /// Access the circuit-specific verification key.
    pub const fn key(&self) -> &VerifyingKey {
        self.key.key()
    }

    /// Return an owned copy of the underlying verification key.
    pub fn into_key(self) -> VerifyingKey {
        self.key.key
    }

    /// Verify a proof against public inputs.
    ///
    /// # Errors
    ///
    /// Returns an error if the public-input count is wrong, a proof element is
    /// the identity, an MSM query length is inconsistent, or the Groth16
    /// pairing equation does not hold.
    pub fn verify(&self, proof: &Proof, public_inputs: &[BlsScalar]) -> Result<(), Error> {
        self.key.verify(proof, public_inputs)
    }
}

impl PreparedVerifyingKey {
    /// Verify a proof using the precomputed fixed pairing terms.
    ///
    /// This performs the same checks as [`Verifier::verify`] while reusing the
    /// prepared `gamma` and `delta` elements and the `alpha`/`beta` pairing.
    pub fn verify(&self, proof: &Proof, public_inputs: &[BlsScalar]) -> Result<(), Error> {
        let expected = self.key.public_inputs();
        if public_inputs.len() != expected {
            return Err(Error::InvalidPublicInputCount {
                expected,
                provided: public_inputs.len(),
            });
        }
        if bool::from(proof.a.is_identity())
            || bool::from(proof.b.is_identity())
            || bool::from(proof.c.is_identity())
        {
            return Err(Error::ProofVerification);
        }

        let public = G1Projective::from(self.key.public_query[0])
            + msm::g1(&self.key.public_query[1..], public_inputs)?;
        let public = G1Affine::from(public);
        let neg_a = -proof.a;
        let proof_b = G2Prepared::from(proof.b);
        let result = multi_miller_loop_result(&[
            (&neg_a, &proof_b),
            (&public, &self.gamma_g2),
            (&proof.c, &self.delta_g2),
        ]) + self.alpha_beta;
        if result == Gt::identity() {
            Ok(())
        } else {
            Err(Error::ProofVerification)
        }
    }
}

/// Synthesize a Composer circuit using the canonical R1CS backend.
fn compose<C: Circuit>(circuit: &C) -> Result<R1csCircuit, Error> {
    let mut composer = Composer::<R1cs>::initialized();
    circuit.circuit(&mut composer)?;
    Ok(composer.into_r1cs())
}

/// Sample a nonzero scalar without introducing modulo bias.
fn random_nonzero<R: RngCore + CryptoRng>(rng: &mut R) -> BlsScalar {
    loop {
        let scalar = BlsScalar::random(&mut *rng);
        if scalar != BlsScalar::zero() {
            return scalar;
        }
    }
}

/// Scalar trapdoors retained only while constructing setup queries.
struct SetupTrapdoor {
    alpha: BlsScalar,
    beta: BlsScalar,
    gamma: BlsScalar,
    delta: BlsScalar,
    tau: BlsScalar,
    gamma_inverse: BlsScalar,
    delta_inverse: BlsScalar,
}

#[cfg(feature = "zeroize")]
impl Zeroize for SetupTrapdoor {
    fn zeroize(&mut self) {
        self.alpha.zeroize();
        self.beta.zeroize();
        self.gamma.zeroize();
        self.delta.zeroize();
        self.tau.zeroize();
        self.gamma_inverse.zeroize();
        self.delta_inverse.zeroize();
    }
}

#[cfg(feature = "zeroize")]
impl Drop for SetupTrapdoor {
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

/// Generate Groth16 proving and verification keys for a finalized R1CS shape.
fn setup<R: RngCore + CryptoRng>(
    rng: &mut R,
    shape: &dusk_zk_composer::R1csShape,
) -> Result<(ProvingKey, VerifyingKey), Error> {
    let domain = EvaluationDomain::new(shape.constraints().len())?;
    let mut alpha = random_nonzero(rng);
    let mut beta = random_nonzero(rng);
    let mut gamma = random_nonzero(rng);
    let mut delta = random_nonzero(rng);
    let mut tau = loop {
        let candidate = random_nonzero(rng);
        if domain.vanishing(candidate) != BlsScalar::zero() {
            break candidate;
        }
    };
    let trapdoor = SetupTrapdoor {
        alpha,
        beta,
        gamma,
        delta,
        tau,
        gamma_inverse: gamma.invert().ok_or(Error::InvalidEvaluationDomain)?,
        delta_inverse: delta.invert().ok_or(Error::InvalidEvaluationDomain)?,
    };
    let qap = qap::evaluate_shape_at_tau(shape, domain, trapdoor.tau)?;

    let g1 = G1Affine::generator();
    let g2 = G2Affine::generator();
    let alpha_g1 = G1Affine::from(g1 * trapdoor.alpha);
    let beta_g1 = G1Affine::from(g1 * trapdoor.beta);
    let beta_g2 = G2Affine::from(g2 * trapdoor.beta);
    let gamma_g2 = G2Affine::from(g2 * trapdoor.gamma);
    let delta_g1 = G1Affine::from(g1 * trapdoor.delta);
    let delta_g2 = G2Affine::from(g2 * trapdoor.delta);

    let a_query = qap
        .u
        .iter()
        .map(|scalar| G1Affine::from(g1 * scalar))
        .collect();
    let b_g1_query = qap
        .v
        .iter()
        .map(|scalar| G1Affine::from(g1 * scalar))
        .collect();
    let b_g2_query = qap
        .v
        .iter()
        .map(|scalar| G2Affine::from(g2 * scalar))
        .collect();
    let combined: Vec<_> = qap
        .u
        .iter()
        .zip(&qap.v)
        .zip(&qap.w)
        .map(|((u, v), w)| trapdoor.beta * u + trapdoor.alpha * v + w)
        .collect();
    let public_end = 1 + shape.public_inputs();
    let public_query = combined[..public_end]
        .iter()
        .map(|scalar| G1Affine::from(g1 * (*scalar * trapdoor.gamma_inverse)))
        .collect();
    let private_query = combined[public_end..]
        .iter()
        .map(|scalar| G1Affine::from(g1 * (*scalar * trapdoor.delta_inverse)))
        .collect();
    let mut power = BlsScalar::one();
    let h_query = (0..domain.size().saturating_sub(1))
        .map(|_| {
            let point = G1Affine::from(g1 * (power * qap.vanishing * trapdoor.delta_inverse));
            power *= trapdoor.tau;
            point
        })
        .collect();

    let verifying_key = VerifyingKey {
        alpha_g1,
        beta_g2,
        gamma_g2,
        delta_g2,
        public_query,
    };
    let proving_key = ProvingKey {
        shape_digest: shape.digest(),
        variables: shape.variables(),
        public_inputs: shape.public_inputs(),
        domain_size: domain.size(),
        alpha_g1,
        beta_g1,
        beta_g2,
        delta_g1,
        delta_g2,
        a_query,
        b_g1_query,
        b_g2_query,
        private_query,
        h_query,
    };

    discard_scalars(combined);
    discard_scalar(&mut power);
    discard_scalar(&mut alpha);
    discard_scalar(&mut beta);
    discard_scalar(&mut gamma);
    discard_scalar(&mut delta);
    discard_scalar(&mut tau);
    Ok((proving_key, verifying_key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dusk_bytes::Serializable;
    use dusk_zk_composer::{ComposerBackend, Constraint};
    use rand_chacha::{ChaCha20Rng, rand_core::SeedableRng};

    #[derive(Default)]
    struct ProductCircuit {
        left: BlsScalar,
        right: BlsScalar,
        result: BlsScalar,
    }

    #[derive(Default)]
    struct GadgetCircuit {
        left: BlsScalar,
        right: BlsScalar,
        result: BlsScalar,
    }

    #[derive(Default)]
    struct NoPublicCircuit {
        secret: BlsScalar,
    }

    impl Circuit for NoPublicCircuit {
        fn circuit<B: ComposerBackend>(
            &self,
            composer: &mut Composer<B>,
        ) -> Result<(), dusk_zk_composer::Error> {
            let secret = composer.append_witness(self.secret);
            let square = composer.gate_mul(Constraint::new().mult(1).a(secret).b(secret));
            composer.assert_equal_constant(square, BlsScalar::from(49u64), None);
            Ok(())
        }
    }

    #[derive(Default)]
    struct PublicOrderCircuit {
        first: BlsScalar,
        second: BlsScalar,
    }

    impl Circuit for PublicOrderCircuit {
        fn circuit<B: ComposerBackend>(
            &self,
            composer: &mut Composer<B>,
        ) -> Result<(), dusk_zk_composer::Error> {
            composer.append_public(self.first);
            composer.append_public(self.second);
            Ok(())
        }
    }

    impl Circuit for GadgetCircuit {
        fn circuit<B: ComposerBackend>(
            &self,
            composer: &mut Composer<B>,
        ) -> Result<(), dusk_zk_composer::Error> {
            let left = composer.append_witness(self.left);
            let right = composer.append_witness(self.right);
            composer.component_range_bits::<8>(left);
            composer.component_range_bits::<8>(right);
            let xor = composer.append_logic_xor::<4>(left, right);
            let result = composer.append_public(self.result);
            composer.assert_equal(xor, result);
            Ok(())
        }
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

    #[derive(Default)]
    struct SumCircuit {
        left: BlsScalar,
        right: BlsScalar,
        result: BlsScalar,
    }

    impl Circuit for SumCircuit {
        fn circuit<B: ComposerBackend>(
            &self,
            composer: &mut Composer<B>,
        ) -> Result<(), dusk_zk_composer::Error> {
            let left = composer.append_witness(self.left);
            let right = composer.append_witness(self.right);
            let sum = composer.gate_add(Constraint::new().left(1).right(1).a(left).b(right));
            let result = composer.append_public(self.result);
            composer.assert_equal(sum, result);
            Ok(())
        }
    }

    #[test]
    fn setup_prove_verify_round_trip() {
        let mut rng = ChaCha20Rng::from_seed([7u8; 32]);
        let (prover, verifier) = Compiler::trusted_setup::<ProductCircuit, _>(&mut rng).unwrap();
        let circuit = ProductCircuit {
            left: BlsScalar::from(3u64),
            right: BlsScalar::from(4u64),
            result: BlsScalar::from(12u64),
        };
        let (proof, public_inputs) = prover.prove(&mut rng, &circuit).unwrap();
        verifier.verify(&proof, &public_inputs).unwrap();
        let (second_proof, second_inputs) = prover.prove(&mut rng, &circuit).unwrap();
        assert_ne!(proof, second_proof);
        assert_eq!(public_inputs, second_inputs);
        verifier.verify(&second_proof, &second_inputs).unwrap();
        assert_eq!(public_inputs, vec![BlsScalar::from(12u64)]);
        assert_eq!(
            verifier.verify(&proof, &[BlsScalar::from(13u64)]),
            Err(Error::ProofVerification)
        );
        assert_eq!(
            verifier.verify(&proof, &[]),
            Err(Error::InvalidPublicInputCount {
                expected: 1,
                provided: 0,
            })
        );

        let mut tampered = proof;
        tampered.a = G1Affine::from(G1Projective::from(tampered.a) + G1Affine::generator());
        assert_eq!(
            verifier.verify(&tampered, &public_inputs),
            Err(Error::ProofVerification)
        );
    }

    #[test]
    fn zero_and_multiple_public_input_edges_verify() {
        let mut rng = ChaCha20Rng::from_seed([8u8; 32]);
        let (prover, verifier) = Compiler::trusted_setup::<NoPublicCircuit, _>(&mut rng).unwrap();
        let (proof, public_inputs) = prover
            .prove(
                &mut rng,
                &NoPublicCircuit {
                    secret: BlsScalar::from(7u64),
                },
            )
            .unwrap();
        assert!(public_inputs.is_empty());
        verifier.verify(&proof, &public_inputs).unwrap();

        let (prover, verifier) =
            Compiler::trusted_setup::<PublicOrderCircuit, _>(&mut rng).unwrap();
        let circuit = PublicOrderCircuit {
            first: BlsScalar::from(17u64),
            second: BlsScalar::from(23u64),
        };
        let (proof, public_inputs) = prover.prove(&mut rng, &circuit).unwrap();
        assert_eq!(
            public_inputs,
            vec![BlsScalar::from(17u64), BlsScalar::from(23u64)]
        );
        verifier.verify(&proof, &public_inputs).unwrap();
        let mut swapped = public_inputs;
        swapped.swap(0, 1);
        assert_eq!(
            verifier.verify(&proof, &swapped),
            Err(Error::ProofVerification)
        );
    }

    #[test]
    fn unsatisfied_circuit_is_rejected_before_proving() {
        let mut rng = ChaCha20Rng::from_seed([9u8; 32]);
        let (prover, _) = Compiler::trusted_setup::<ProductCircuit, _>(&mut rng).unwrap();
        let circuit = ProductCircuit {
            left: BlsScalar::from(3u64),
            right: BlsScalar::from(4u64),
            result: BlsScalar::from(13u64),
        };
        assert_eq!(
            prover.prove(&mut rng, &circuit),
            Err(Error::CircuitUnsatisfied)
        );
    }

    #[test]
    fn a_different_shape_is_rejected_before_proving() {
        let mut rng = ChaCha20Rng::from_seed([10u8; 32]);
        let (prover, _) = Compiler::trusted_setup::<ProductCircuit, _>(&mut rng).unwrap();
        let circuit = SumCircuit {
            left: BlsScalar::from(3u64),
            right: BlsScalar::from(4u64),
            result: BlsScalar::from(7u64),
        };
        assert_eq!(
            prover.prove(&mut rng, &circuit),
            Err(Error::InvalidCircuitShape)
        );
    }

    #[test]
    fn an_inconsistent_key_domain_is_rejected_without_panicking() {
        let mut rng = ChaCha20Rng::from_seed([12u8; 32]);
        let (prover, _) = Compiler::trusted_setup::<ProductCircuit, _>(&mut rng).unwrap();
        let mut key = prover.into_key();
        key.domain_size *= 2;
        key.h_query
            .resize(key.domain_size - 1, G1Affine::generator());
        let prover = Prover::from_key(key);
        let circuit = ProductCircuit {
            left: BlsScalar::from(3u64),
            right: BlsScalar::from(4u64),
            result: BlsScalar::from(12u64),
        };
        assert_eq!(
            prover.prove(&mut rng, &circuit),
            Err(Error::InvalidCircuitShape)
        );
    }

    #[test]
    fn serialized_keys_can_prove_and_verify() {
        let mut rng = ChaCha20Rng::from_seed([11u8; 32]);
        let (prover, verifier) = Compiler::trusted_setup::<ProductCircuit, _>(&mut rng).unwrap();
        let proving_bytes = prover.key().to_bytes();
        let verifying_bytes = verifier.key().to_bytes();
        let proving_key = ProvingKey::try_from_bytes(&proving_bytes).unwrap();
        let verifying_key = VerifyingKey::try_from_bytes(&verifying_bytes).unwrap();
        assert_eq!(&proving_key, prover.key());
        assert_eq!(&verifying_key, verifier.key());
        let prover = Prover::from_key(proving_key);
        let verifier = Verifier::from_key(verifying_key);
        let circuit = ProductCircuit {
            left: BlsScalar::from(5u64),
            right: BlsScalar::from(6u64),
            result: BlsScalar::from(30u64),
        };
        let (proof, public_inputs) = prover.prove(&mut rng, &circuit).unwrap();
        let proof = Proof::from_bytes(&proof.to_bytes()).unwrap();
        verifier.verify(&proof, &public_inputs).unwrap();

        assert_eq!(
            ProvingKey::try_from_bytes(&proving_bytes[..proving_bytes.len() - 1]),
            Err(Error::InvalidEncoding)
        );
        let mut malformed = verifying_bytes;
        malformed[8] = 2;
        assert_eq!(
            VerifyingKey::try_from_bytes(&malformed),
            Err(Error::InvalidEncoding)
        );
    }

    #[test]
    fn specialized_composer_identities_prove_and_verify() {
        let mut rng = ChaCha20Rng::from_seed([13u8; 32]);
        let (prover, verifier) = Compiler::trusted_setup::<GadgetCircuit, _>(&mut rng).unwrap();
        let circuit = GadgetCircuit {
            left: BlsScalar::from(0xabu64),
            right: BlsScalar::from(0x3cu64),
            result: BlsScalar::from(0x97u64),
        };
        let (proof, public_inputs) = prover.prove(&mut rng, &circuit).unwrap();
        verifier.verify(&proof, &public_inputs).unwrap();

        let invalid = GadgetCircuit {
            result: BlsScalar::from(0x96u64),
            ..circuit
        };
        assert_eq!(
            prover.prove(&mut rng, &invalid),
            Err(Error::CircuitUnsatisfied)
        );
    }

    #[cfg(feature = "zeroize")]
    #[test]
    fn setup_trapdoor_can_be_explicitly_zeroized() {
        let mut trapdoor = SetupTrapdoor {
            alpha: BlsScalar::one(),
            beta: BlsScalar::one(),
            gamma: BlsScalar::one(),
            delta: BlsScalar::one(),
            tau: BlsScalar::one(),
            gamma_inverse: BlsScalar::one(),
            delta_inverse: BlsScalar::one(),
        };
        trapdoor.zeroize();
        assert_eq!(trapdoor.alpha, BlsScalar::zero());
        assert_eq!(trapdoor.beta, BlsScalar::zero());
        assert_eq!(trapdoor.gamma, BlsScalar::zero());
        assert_eq!(trapdoor.delta, BlsScalar::zero());
        assert_eq!(trapdoor.tau, BlsScalar::zero());
        assert_eq!(trapdoor.gamma_inverse, BlsScalar::zero());
        assert_eq!(trapdoor.delta_inverse, BlsScalar::zero());
    }
}
