# Groth'16

[![Build Status](https://github.com/dusk-network/zk-tools/actions/workflows/ci.yml/badge.svg)](https://github.com/dusk-network/zk-tools/actions/workflows/ci.yml)
[![Repository](https://img.shields.io/badge/github-zk--tools-blueviolet?logo=github)](https://github.com/dusk-network/zk-tools)

`dusk-groth16` implements the Groth'16 preprocessing zk-SNARK over BLS12-381
for circuits built with `dusk-zk-composer`.

The current setup is circuit-specific and single-party. The process and random
number generator used for setup must be trusted, and all setup randomness must
be destroyed. Multi-party setup ceremonies are not implemented yet.

Setup compiles the default circuit through Composer's `R1cs` backend, pads the
constraint count to a power-of-two evaluation domain, and constructs the
Groth16 QAP queries over BLS12-381. Proving rebuilds the supplied circuit,
checks its canonical shape digest and assignment, and returns public inputs in
circuit emission order.

Proofs have a fixed 192-byte canonical compressed encoding through
`dusk_bytes::Serializable`. Proving and verification keys use versioned,
length-checked encodings through `to_bytes` and `try_from_bytes`. A verifier
automatically prepares the fixed pairing terms when constructed.

## Solidity verification

`VerifyingKey::solidity_verifier` generates a circuit-specific verifier that
uses the [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537) BLS12-381 G1 MSM
and pairing precompiles. Generate a new contract for every verification key.
The contract accepts public inputs in the same Composer emission order as the
Rust verifier and rejects values that are not canonical BLS scalar-field
elements.

Solidity verification uses a separate, uncompressed 512-byte proof transport
encoding. Convert a Rust `Proof` with `Proof::to_eip2537`, or convert saved
canonical proof bytes with the `dusk-groth16-solidity encode-proof` command.
This transport stores `-A`, `B`, and `C` in EIP-2537 order; it does not replace
the crate's canonical 192-byte proof format.

Generated contracts only work on EVM networks that have activated EIP-2537.
Each public-input query point is embedded in runtime bytecode, so circuits
with many public inputs must also be checked against the target chain's
contract-size limit (for example with `forge build --sizes`). See [`tests/solidity/README.md`](tests/solidity/README.md) for generation
commands, integration details, and the Foundry test fixture.

## Circuit and public-input model

`Compiler::trusted_setup` synthesizes `C::default()` and binds the resulting
canonical R1CS shape into the proving key. The default assignment does not
need to satisfy the circuit because setup consumes only its topology. Circuit
topology must therefore be independent of witness and public-input values:
conditional behavior belongs inside constraints, not in Rust control flow
that changes which constraints are emitted. A later proof is rejected if its
R1CS shape differs from setup.

The R1CS assignment is ordered as constant one, public inputs in circuit
emission order, Composer witnesses, and R1CS lowering auxiliaries. `prove`
returns exactly that public-input prefix without the constant-one entry.

## Security and compatibility

- Setup is circuit-specific and single-party. No phase-two MPC ceremony or
  contribution verification is implemented.
- With the `zeroize` feature, setup trapdoors, derived setup scalars, quotient
  coefficients, and finalized R1CS assignments receive best-effort zeroization.
  Callers remain responsible for destroying RNG state and their original
  circuit witness values.
- Point decoding is canonical and subgroup-checked by `dusk-curves`; proofs
  and fixed verification-key parameters at the identity are rejected.
- Solidity verifiers depend on the target EVM's EIP-2537 implementation for
  point validation and subgroup checks. They fail closed when a precompile is
  unavailable or returns malformed output.

## Features

- `std` enables standard-library support and implies `alloc`.
- `alloc` enables setup, proving, verification, keys, and proofs without
  requiring `std`.
- `bls-backend-dusk` and `bls-backend-blst` select exactly one BLS12-381
  backend.
- `zeroize` enables best-effort clearing of sensitive internal scalar buffers.
- `parallel` forwards Rayon support to the portable Dusk backend; it cannot be
  combined with `bls-backend-blst`. The local FFT and G2 MSM implementations
  are currently serial.

The implementation follows Jens Groth's
[“On the Size of Pairing-based Non-interactive Arguments”](https://eprint.iacr.org/2016/260)
(EUROCRYPT 2016).

## Licensing

This code is licensed under the Mozilla Public License Version 2.0 (MPL-2.0).
Please see the [workspace license](https://github.com/dusk-network/zk-tools/blob/main/LICENSE)
for more information.
