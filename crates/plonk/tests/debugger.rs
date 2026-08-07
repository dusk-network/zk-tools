// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::{env, io};

use dusk_cdf::CircuitDescription;
use dusk_plonk::prelude::*;

#[derive(Debug, Default)]
struct EmptyCircuit;

impl Circuit for EmptyCircuit {
    fn circuit<B: ComposerBackend>(
        &self,
        _composer: &mut Composer<B>,
    ) -> Result<(), CircuitError> {
        Ok(())
    }
}

#[test]
fn generate_cdf_works() -> io::Result<()> {
    let rng = &mut rand::thread_rng();

    let dir = tempdir::TempDir::new("plonk-cdf")?;
    let path = dir.path().canonicalize()?.join("test.cdf");

    let label = b"transcript-arguments";
    let pp = PublicParameters::setup(1 << 5, rng)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let (prover, _verifier) = Compiler::compile::<EmptyCircuit>(&pp, label)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let previous_output = env::var_os("CDF_OUTPUT");
    unsafe {
        env::set_var("CDF_OUTPUT", &path);
    }

    let prove_result = prover
        .prove(rng, &EmptyCircuit)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e));

    unsafe {
        match previous_output {
            Some(value) => env::set_var("CDF_OUTPUT", value),
            None => env::remove_var("CDF_OUTPUT"),
        }
    }

    prove_result?;

    let description = path.canonicalize().and_then(CircuitDescription::open);
    let cleanup = dir.close();

    description?;
    cleanup
}
