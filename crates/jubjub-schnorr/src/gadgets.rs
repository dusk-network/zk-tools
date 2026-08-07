// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! # Schnorr Signature Gadgets
//!
//! This module provides Plonk gadgets for verification of Schnorr signatures.

use dusk_jubjub::{GENERATOR_EXTENDED, GENERATOR_NUMS_EXTENDED};
use dusk_plonk::prelude::*;
use dusk_poseidon::{Domain, HashGadget};

fn assert_not_identity(composer: &mut Composer, y: Witness) {
    // For an on-curve JubJub point, y != 1 is equivalent to requiring a
    // non-identity point. Enforce it with an inverse witness:
    // (y - 1) * inverse = 1.
    let inverse = (composer[y] - BlsScalar::one())
        .invert()
        .unwrap_or(BlsScalar::zero());
    let inverse = composer.append_witness(inverse);
    composer.append_gate(
        Constraint::new()
            .mult(1)
            .a(y)
            .right(-BlsScalar::one())
            .b(inverse)
            .constant(-BlsScalar::one()),
    );
}

fn assert_valid_point(
    composer: &mut Composer,
    point: WitnessPoint,
) -> TorsionFreeWitnessPoint {
    let point = composer.assert_torsion_free_point(point);

    // A torsion-free point may still be the identity, which is not a valid
    // Schnorr public key or variable generator.
    assert_not_identity(composer, *point.y());

    point
}

/// Verifies a single-key Schnorr signature [`Signature`]within a Plonk circuit
/// without requiring the secret key as a witness.
///
/// The function performs Schnorr verification by calculating the challenge and
/// confirming the signature equation.
///
/// # Feature
///
/// Only available with the "zk" feature enabled.
///
/// ### Parameters
///
/// - `composer`: A mutable reference to the Plonk [`Composer`]`.
/// - `u`: Witness for the random nonce used during signature generation.
/// - `r`: Witness Point representing the nonce point `r = u*G`.
/// - `pk`: Witness Point representing the public key `pk = sk*G`.
/// - `msg`: Witness for the message.
///
/// ### Returns
///
/// - `Result<(), Error>`: Returns an empty `Result` on successful gadget
///   creation or an `Error` if the witness `u` is not a valid [`JubJubScalar`].
///
/// ### Errors
///
/// This function will return an `Error` if the witness `u` is not a valid
/// [`JubJubScalar`].
///
/// [`Signature`]: [`crate::Signature`]
pub fn verify_signature(
    composer: &mut Composer,
    u: Witness,
    r: WitnessPoint,
    pk: WitnessPoint,
    msg: Witness,
) -> Result<(), Error> {
    let pk = assert_valid_point(composer, pk);
    assert_not_identity(composer, *r.y());

    let r_x = *r.x();
    let r_y = *r.y();

    let pk_x = *pk.x();
    let pk_y = *pk.y();

    let challenge = [r_x, r_y, pk_x, pk_y, msg];
    let challenge_hash =
        HashGadget::digest_truncated(composer, Domain::Other, &challenge)[0];

    let s_a = composer.component_mul_generator(u, GENERATOR_EXTENDED)?;
    let s_b = composer.component_mul_point(challenge_hash, pk);
    let point = composer.component_add_point(s_a, s_b);

    composer.assert_equal_point(r, point.into());

    Ok(())
}

/// Verifies a [`SignatureDouble`] within a Plonk circuit without requiring
/// the secret key as a witness.
///
/// # Feature
///
/// Only available with the "zk" feature enabled.
///
/// ### Parameters
///
/// - `composer`: A mutable reference to the Plonk [`Composer`].
/// - `u`: Witness for the random nonce used during signature generation.
/// - `r`: Witness Point representing the nonce points `R = u*G`
/// - `r_p`: Witness Point representing the nonce points `R' = u*G'`.
/// - `pk`: Witness Point public key `PK = sk*G`
/// - `pk_p`: Witness Point public key `PK' = sk*G'`
/// - `msg`: Witness for the message.
///
/// ### Returns
///
/// - `Result<(), Error>`: Returns an empty `Result` on successful gadget
///   creation or an `Error` if the witness `u` is not a valid [`JubJubScalar`].
///
/// ### Errors
///
/// This function will return an `Error` if the witness `u` is not a valid
/// [`JubJubScalar`].
///
/// [`SignatureDouble`]: [`crate::SignatureDouble`]
pub fn verify_signature_double(
    composer: &mut Composer,
    u: Witness,
    r: WitnessPoint,
    r_p: WitnessPoint,
    pk: WitnessPoint,
    pk_p: WitnessPoint,
    msg: Witness,
) -> Result<(), Error> {
    let pk = assert_valid_point(composer, pk);
    let pk_p = assert_valid_point(composer, pk_p);
    assert_not_identity(composer, *r.y());
    assert_not_identity(composer, *r_p.y());

    let r_x = *r.x();
    let r_y = *r.y();

    let r_p_x = *r_p.x();
    let r_p_y = *r_p.y();

    let pk_x = *pk.x();
    let pk_y = *pk.y();

    let challenge = [r_x, r_y, r_p_x, r_p_y, pk_x, pk_y, msg];
    let challenge_hash =
        HashGadget::digest_truncated(composer, Domain::Other, &challenge)[0];

    let s_a = composer.component_mul_generator(u, GENERATOR_EXTENDED)?;
    let s_b = composer.component_mul_point(challenge_hash, pk);
    let point = composer.component_add_point(s_a, s_b);

    let s_p_a = composer.component_mul_generator(u, GENERATOR_NUMS_EXTENDED)?;
    let s_p_b = composer.component_mul_point(challenge_hash, pk_p);
    let point_p = composer.component_add_point(s_p_a, s_p_b);

    composer.assert_equal_point(r, point.into());
    composer.assert_equal_point(r_p, point_p.into());

    Ok(())
}

/// Verifies a Schnorr signature with variable generator [`SignatureVarGen`]
/// within a Plonk circuit without requiring the secret key as a witness.
///
/// The function performs Schnorr verification by calculating the challenge and
/// confirming the signature equation.
///
/// # Feature
///
/// Only available with the "zk" feature enabled.
///
/// ### Parameters
///
/// - `composer`: A mutable reference to the Plonk [`Composer`]`.
/// - `u`: Witness for the random nonce used during signature generation.
/// - `r`: Witness Point representing the nonce point `r = u*G`.
/// - `pk`: Witness Point representing the public key `pk = sk*G`.
/// - `generator`: Witness Point representing the variable generator `G`
/// - `msg`: Witness for the message.
///
/// ### Returns
///
/// - `Result<(), Error>`: Returns an empty `Result` on successful gadget
///   creation. The generated constraints require `u` to be a canonical
///   [`JubJubScalar`].
///
/// ### Errors
///
/// This function currently has no host-side error path. A non-canonical `u`
/// makes the circuit unsatisfiable.
///
/// [`SignatureVarGen`]: [`crate::SignatureVarGen`]
pub fn verify_signature_var_gen(
    composer: &mut Composer,
    u: Witness,
    r: WitnessPoint,
    pk: WitnessPoint,
    generator: WitnessPoint,
    msg: Witness,
) -> Result<(), Error> {
    let pk = assert_valid_point(composer, pk);
    let generator = assert_valid_point(composer, generator);
    assert_not_identity(composer, *r.y());
    composer.assert_canonical_jubjub_scalar(u);

    let r_x = *r.x();
    let r_y = *r.y();

    let pk_x = *pk.x();
    let pk_y = *pk.y();

    let gen_x = *generator.x();
    let gen_y = *generator.y();

    let challenge = [r_x, r_y, pk_x, pk_y, gen_x, gen_y, msg];
    let challenge_hash =
        HashGadget::digest_truncated(composer, Domain::Other, &challenge)[0];

    let s_a = composer.component_mul_point(u, generator);
    let s_b = composer.component_mul_point(challenge_hash, pk);
    let point = composer.component_add_point(s_a, s_b);

    composer.assert_equal_point(r, point.into());

    Ok(())
}
