# ZK Tools

[![Build Status](https://github.com/dusk-network/zk-tools/actions/workflows/ci.yml/badge.svg)](https://github.com/dusk-network/zk-tools/actions/workflows/ci.yml)
[![Repository](https://img.shields.io/badge/github-zk--tools-blueviolet?logo=github)](https://github.com/dusk-network/zk-tools)

This repository contains cryptographic tools used to develop zero-knowledge applications. It is a history-preserving monorepo containing:

- `dusk-plonk` in `crates/plonk/`
- `dusk-poseidon` in `crates/poseidon/`
- `dusk-merkle` in `crates/merkle/dusk-merkle/`
- `poseidon-merkle` in `crates/merkle/poseidon-merkle/`
- `jubjub-schnorr` in `crates/jubjub-schnorr/`
- `plonkwasm` in `crates/plonkwasm/`

The original repositories were imported with their relevant Git histories and relocated under `crates/`. Original revisions and provenance are recorded in [`UPSTREAMS.md`](UPSTREAMS.md), while [`docs/upstream-imports.md`](docs/upstream-imports.md) documents the reproducible transformations required for future synchronization.

Notice that the dependencies of the imported crates have been updated to use local paths. Furthermore, take into account that changes can be applied to introduce new experimental features or improvements.

> **⚠️ DISCLAIMER:** this workspace is currently experimental and intended for coordinated development. It is not published as a combined package, and the imported crates should not be released from this repository.

## License

This project is licensed under the [Mozilla Public License 2.0](LICENSE).
