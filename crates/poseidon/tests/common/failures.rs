// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_plonk::prelude::{BlsScalar, Circuit};
use rand::rngs::StdRng;

use super::DualProvers;

impl DualProvers {
    pub(crate) fn assert_proving_fails<C>(&self, rng: &mut StdRng, circuit: &C)
    where
        C: Circuit,
    {
        assert!(
            self.plonk_prover.prove(rng, circuit).is_err(),
            "PLONK proving must reject the invalid circuit"
        );
        assert!(
            self.groth_prover.prove(rng, circuit).is_err(),
            "Groth16 proving must reject the invalid circuit"
        );
    }

    pub(crate) fn assert_verification_fails<C>(
        &self,
        rng: &mut StdRng,
        circuit: &C,
        public_inputs: &[BlsScalar],
        invalid_public_inputs: &[BlsScalar],
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
        assert!(
            self.plonk_verifier
                .verify(&plonk_proof, invalid_public_inputs)
                .is_err(),
            "PLONK verification must reject invalid public inputs"
        );

        let (groth_proof, groth_inputs) = self
            .groth_prover
            .prove(rng, circuit)
            .expect("Groth16 proof generation should succeed");
        assert_eq!(groth_inputs.as_slice(), public_inputs);
        self.groth_verifier
            .verify(&groth_proof, public_inputs)
            .expect("Groth16 proof verification should succeed");
        assert!(
            self.groth_verifier
                .verify(&groth_proof, invalid_public_inputs)
                .is_err(),
            "Groth16 verification must reject invalid public inputs"
        );
    }
}
