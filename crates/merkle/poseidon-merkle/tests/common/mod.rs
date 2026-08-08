// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_groth16::Compiler as GrothCompiler;
use dusk_plonk::prelude::{
    BlsScalar, Circuit, Compiler as PlonkCompiler, PublicParameters,
};
use rand::rngs::StdRng;

pub(crate) fn prove_and_verify<C>(
    rng: &mut StdRng,
    public_parameters: &PublicParameters,
    label: &'static [u8],
    circuit: &C,
    public_inputs: &[BlsScalar],
) where
    C: Circuit,
{
    let (plonk_prover, plonk_verifier) =
        PlonkCompiler::compile::<C>(public_parameters, label)
            .expect("PLONK circuit compilation should succeed");
    let (plonk_proof, plonk_inputs) = plonk_prover
        .prove(rng, circuit)
        .expect("PLONK proof generation should succeed");
    assert_eq!(plonk_inputs.as_slice(), public_inputs);
    plonk_verifier
        .verify(&plonk_proof, public_inputs)
        .expect("PLONK proof verification should succeed");

    let (groth_prover, groth_verifier) =
        GrothCompiler::trusted_setup::<C, _>(rng)
            .expect("Groth16 setup should succeed");
    let (groth_proof, groth_inputs) = groth_prover
        .prove(rng, circuit)
        .expect("Groth16 proof generation should succeed");
    assert_eq!(groth_inputs.as_slice(), public_inputs);
    groth_verifier
        .verify(&groth_proof, public_inputs)
        .expect("Groth16 proof verification should succeed");
}
