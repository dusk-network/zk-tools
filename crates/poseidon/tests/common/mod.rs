// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_groth16::{
    Compiler as GrothCompiler, Prover as GrothProver, Verifier as GrothVerifier,
};
use dusk_plonk::prelude::{
    BlsScalar, Circuit, Compiler as PlonkCompiler, Prover as PlonkProver,
    PublicParameters, Verifier as PlonkVerifier,
};
use rand::rngs::StdRng;

pub(crate) struct DualProvers {
    pub(crate) plonk_prover: PlonkProver,
    pub(crate) plonk_verifier: PlonkVerifier,
    pub(crate) groth_prover: GrothProver,
    pub(crate) groth_verifier: GrothVerifier,
}

impl DualProvers {
    pub(crate) fn compile<C>(
        rng: &mut StdRng,
        public_parameters: &PublicParameters,
        label: &'static [u8],
    ) -> Self
    where
        C: Circuit,
    {
        let (plonk_prover, plonk_verifier) =
            PlonkCompiler::compile::<C>(public_parameters, label)
                .expect("PLONK circuit compilation should succeed");
        let (groth_prover, groth_verifier) =
            GrothCompiler::trusted_setup::<C, _>(rng)
                .expect("Groth16 setup should succeed");

        Self {
            plonk_prover,
            plonk_verifier,
            groth_prover,
            groth_verifier,
        }
    }

    pub(crate) fn prove_and_verify<C>(
        &self,
        rng: &mut StdRng,
        circuit: &C,
        public_inputs: &[BlsScalar],
    ) where
        C: Circuit,
    {
        let (plonk_proof, plonk_inputs) = self
            .plonk_prover
            .prove(rng, circuit)
            .expect("PLONK proof generation should succeed");
        assert_eq!(plonk_inputs.as_slice(), public_inputs);
        self.plonk_verifier
            .verify(&plonk_proof, public_inputs)
            .expect("PLONK proof verification should succeed");

        let (groth_proof, groth_inputs) = self
            .groth_prover
            .prove(rng, circuit)
            .expect("Groth16 proof generation should succeed");
        assert_eq!(groth_inputs.as_slice(), public_inputs);
        self.groth_verifier
            .verify(&groth_proof, public_inputs)
            .expect("Groth16 proof verification should succeed");
    }
}
