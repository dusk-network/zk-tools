# PLONK 
[![Build Status](https://github.com/dusk-network/zk-tools/actions/workflows/ci.yml/badge.svg)](https://github.com/dusk-network/zk-tools/actions/workflows/ci.yml)
[![Repository](https://img.shields.io/badge/github-plonk-blueviolet?logo=github)](https://github.com/dusk-network/plonk)

_This is a pure Rust implementation of the PLONK proving system over BLS12-381._

This library contains a modular implementation of KZG10 as the default polynomial commitment scheme. Moreover, it includes custom gates for efficiency purposes. The details on our specific implementation can be found [here](docs/dusk-plonk-specs.pdf). An audit can be found [here](https://github.com/dusk-network/audits).

**DISCLAIMER**: This library is currently unstable. A security audit has been completed, though further in-depth analysis and testing are encouraged. Use at your own risk.

## Usage

Select exactly one BLS12-381 backend when adding this crate. For example, to
use BLST:

```toml
dusk-plonk = { version = "0.22", default-features = false, features = ["std", "bls-backend-blst"] }
```

Use `bls-backend-dusk` instead for the pure-Rust Dusk backend. To see how to
use this library, check the `examples` directory.

## Features

This crate includes a variety of features which are briefly explained below:
- `bls-backend-blst`: Selects the optimized BLST BLS12-381 backend.
- `bls-backend-dusk`: Selects the pure-Rust Dusk BLS12-381 backend. It is
  mutually exclusive with `bls-backend-blst`.
- `alloc`: Enables the usage of an allocator, allowing for `Proof` constructions and verifications. Without this feature it **IS NOT** possible to prove or verify anything. 
  Its absence only makes `dusk-plonk` export certain fixed-size data structures such as `Proof`. This is useful in no_std environments that also do not make use of an allocator.
- `std`: Enables `std` usage as well as `rayon` parallelization in some proving and verifying operations. 
  This feature is enabled by default, but does not select a BLS backend.
- `parallel`: Enables the Dusk curve backend's parallel implementation. This
  feature is not compatible with `bls-backend-blst`.
- `rkyv-impl`: Enables rkyv serialization. Archived curve representations are
  backend-specific and must not be read after switching backends without an
  explicit migration.
- `debug`: Enables the runtime debugger backend, outputting [CDF](https://crates.io/crates/dusk-cdf) files to the path defined in the `CDF_OUTPUT` environment variable. When used, the binary must be compiled with `debug = true`. For more info, check the [cargo book](https://doc.rust-lang.org/cargo/reference/profiles.html#debug).
  __It is recommended to derive the std output and std error and then place them in a text file for efficient gate analysis.__
- `legacy-proving`: Enables creation of legacy V2 proofs through
  `Prover::prove_with_version`. Legacy proving is disabled by default; current
  V3 proving and legacy verification do not require this feature.

## Documentation

The crate documentation provides information about all the functions that the library provides, as well
as the documentation regarding the data structures that it exports. To check this, visit the [documentation page](https://docs.rs/dusk-plonk/) or run `make doc` from the repository root.

## Performance

Benchmarks taken on `Apple M1`, for a circuit-size of `2^16` constraints:

- Proving time: `7.871s`
- Verification time: `2.821ms` **(This time does not vary depending on the circuit-size.)**

For more results, run
`cargo bench --no-default-features --features=bls-backend-blst,std` to get a
full report of benchmarks in respect of constraint numbers.

## Acknowledgements

- Reference implementation by Aztec Protocol/Barretenberg.
- FFT Module and KZG10 Module were adapted from ZEXE/Zcash and SCIPR Lab, respectively.

## Licensing

This code is licensed under the Mozilla Public License Version 2.0 (MPL-2.0). Please see the [workspace license](https://github.com/dusk-network/zk-tools/blob/main/LICENSE) for more information.

## About

This implementation is designed by the [Dusk](https://dusk.network) team.

## Contributing

- If you want to contribute to this repository/project, please check our [CONTRIBUTING.md](CONTRIBUTING.md).
- If you want to report a bug or request a new feature addition, please open an issue on this repository.
