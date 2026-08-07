[![Build Status](https://github.com/dusk-network/zk-tools/actions/workflows/ci.yml/badge.svg)](https://github.com/dusk-network/zk-tools/actions/workflows/ci.yml)
[![Repository](https://img.shields.io/badge/github-poseidon252-blueviolet)](https://github.com/dusk-network/Poseidon252)

# Dusk-Poseidon

Reference implementation for the Poseidon Hashing algorithm.

Reference:
[Starkad and Poseidon: New Hash Functions for Zero Knowledge Proof Systems](https://eprint.iacr.org/2019/458.pdf)

This repository has been created so there's a unique library that holds the tools & functions required to perform Poseidon Hashes on field elements of the bls12-381 elliptic curve.

The hash uses the Hades design for its inner permutation and the [SAFE](https://eprint.iacr.org/2023/522.pdf) framework for contstructing the sponge.

The library provides the two hashing techniques of Poseidon:
- The 'normal' hashing functionalities operating on `BlsScalar`.
- The 'gadget' hashing functionalities that build a circuit which outputs the hash.

Consumers must select exactly one curve backend. For example:

```toml
dusk-poseidon = { version = "0.42", default-features = false, features = ["bls-backend-blst"] }
dusk-curves = { version = "0.2", default-features = false, features = ["bls-backend-blst"] }
```

Use `bls-backend-dusk` on both dependencies to select the pure-Rust backend.

## Example

```rust
use rand::rngs::StdRng;
use rand::SeedableRng;

use dusk_poseidon::{Domain, Hash};
use dusk_curves::bls12_381::BlsScalar;
use ff::Field;

// generate random input
let mut rng = StdRng::seed_from_u64(0xbeef);
let mut input = [BlsScalar::zero(); 42];
for scalar in input.iter_mut() {
    *scalar = BlsScalar::random(&mut rng);
}

// digest the input all at once
let hash = Hash::digest(Domain::Other, &input);

// update the input gradually
let mut hasher = Hash::new(Domain::Other);
hasher.update(&input[..3]);
hasher.update(&input[3..]);
assert_eq!(hash, hasher.finalize());

// create a hash used for merkle tree hashing with arity = 4
let merkle_hash = Hash::digest(Domain::Merkle4, &input[..4]);

// which is different when another domain is used
assert_ne!(merkle_hash, Hash::digest(Domain::Other, &input[..4]));
```

## Benchmarks

There are benchmarks for hashing, encrypting and decrypting in their native form, operating on `Scalar`, and for a zero-knowledge circuit proof generation and verification.

To run all benchmarks on your machine, run
```shell
cargo bench --no-default-features --features=bls-backend-blst,zk,encryption
```
in the repository.

## Licensing

This code is licensed under Mozilla Public License Version 2.0 (MPL-2.0). Please see the [workspace license](https://github.com/dusk-network/zk-tools/blob/main/LICENSE) for further info.

## About

Implementation designed by the [dusk](https://dusk.network) team.

## Contributing

- If you want to contribute to this repository/project please, check [CONTRIBUTING.md](CONTRIBUTING.md)
- If you want to report a bug or request a new feature addition, please open an issue on this repository.
