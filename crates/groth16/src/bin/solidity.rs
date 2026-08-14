// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::{env, error::Error, fs, io, process::ExitCode};

use dusk_bytes::DeserializableSlice;
use dusk_groth16::{PROOF_SIZE, Proof, VerifyingKey};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("dusk-groth16-solidity: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args();
    let program = arguments
        .next()
        .unwrap_or_else(|| "dusk-groth16-solidity".into());
    let Some(command) = arguments.next() else {
        return Err(usage(&program).into());
    };

    match command.as_str() {
        "generate-verifier" => {
            let key_path = required_argument(&mut arguments, &program)?;
            let contract_name = required_argument(&mut arguments, &program)?;
            let output_path = required_argument(&mut arguments, &program)?;
            reject_extra(arguments, &program)?;

            let key = VerifyingKey::try_from_bytes(&fs::read(key_path)?)?;
            fs::write(output_path, key.solidity_verifier(&contract_name)?)?;
        }
        "encode-proof" => {
            let proof_path = required_argument(&mut arguments, &program)?;
            let output_path = required_argument(&mut arguments, &program)?;
            reject_extra(arguments, &program)?;

            let bytes = fs::read(proof_path)?;
            let proof_bytes: [u8; PROOF_SIZE] = bytes.try_into().map_err(|bytes: Vec<u8>| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "invalid proof length: expected {PROOF_SIZE}, provided {}",
                        bytes.len()
                    ),
                )
            })?;
            let proof = Proof::from_slice(&proof_bytes).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid canonical proof encoding",
                )
            })?;
            fs::write(output_path, proof.to_eip2537().as_bytes())?;
        }
        _ => return Err(usage(&program).into()),
    }

    Ok(())
}

fn required_argument(
    arguments: &mut impl Iterator<Item = String>,
    program: &str,
) -> Result<String, Box<dyn Error>> {
    arguments.next().ok_or_else(|| usage(program).into())
}

fn reject_extra(
    mut arguments: impl Iterator<Item = String>,
    program: &str,
) -> Result<(), Box<dyn Error>> {
    if arguments.next().is_some() {
        Err(usage(program).into())
    } else {
        Ok(())
    }
}

fn usage(program: &str) -> String {
    format!(
        "usage:\n  {program} generate-verifier <verifying-key> <contract-name> <output.sol>\n  \
         {program} encode-proof <canonical-proof> <output.bin>"
    )
}
