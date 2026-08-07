# Plonk — AI Agent Instructions

## Project Overview

**dusk-plonk** is a pure Rust implementation of the PLONK ZK proving system over BLS12-381 with a KZG10 polynomial commitment scheme and custom gates. It is a member of the `zk-tools` workspace.

### Key Directories

| Directory | Purpose |
|-----------|---------|
| `src/` | Library source |
| `src/composer/` | Circuit building, constraint system, gates |
| `src/proof_system/` | Proving and verification, widgets |
| `src/commitment_scheme/` | KZG10 polynomial commitments |
| `src/fft/` | Fast Fourier Transform, polynomial arithmetic |
| `tests/` | Integration tests |
| `benches/` | Benchmarks |
| `examples/` | Usage example (`circuit.rs`) |
| `docs/` | PLONK specs PDF |

## Commands

Run these commands from the `zk-tools` repository root.

### Build

    cargo build --no-default-features --features bls-backend-blst,std
    make clippy                                  # Preferred — checks and lints in one step

### Test

    make test                                    # Full suite (release mode)
    cargo test --release --no-default-features --features bls-backend-blst,std
    cargo run --release --example circuit --no-default-features --features bls-backend-blst,std

**Tests MUST use `--release`** — debug mode takes up to an hour for proof tests.

### Lint

    make clippy
    make fmt                                     # Requires nightly toolchain

### no_std Verification

    make no-std

### Docs

    make doc                                     # Build docs with KaTeX

### PR Minimum

    make test
    make fmt
    make clippy

## Architecture

Widget-based prover with a Turbo composer. Circuits implement the `Circuit` trait, which defines `circuit(&self, composer: &mut Composer)` to build constraints. The `Compiler` takes a circuit + public parameters and produces prover/verifier pairs.

**Core flow**: Circuit -> Composer (constraint system) -> Compiler -> ProverKey/VerifierKey -> Proof -> Verify.

**Widget types** (in `proof_system/widget/`): arithmetic, logic, range, ECC, permutation — each handles a category of constraints during proving and verification.

**Commitment scheme**: KZG10 with a structured reference string (SRS). The `PublicParameters` type holds the SRS and is needed for compilation.

## Feature Flags

| Feature | Purpose | Default |
|---------|---------|---------|
| `bls-backend-dusk` | Pure-Rust BLS12-381 backend | No |
| `bls-backend-blst` | BLST BLS12-381 backend | No |
| `std` | Enables rayon parallelism | Yes |
| `alloc` | Core feature for proof construction/verification | No |
| `debug` | Runtime debugger with CDF output | No |
| `rkyv-impl` | rkyv serialization support | No |
| `parallel` | Dusk-backend curve parallelism | No |

## Elevated Care Zone

This is a cryptographic crate — soundness bugs break consensus and privacy. Work with extra diligence.

- **Verify**: `make test` (covers the supported BLST feature matrix) and
  `make no-std`
- **Watch**: polynomial arithmetic, commitment opening proofs, transcript (Fiat-Shamir) binding, gate constraint enforcement

## Conventions

- **`no_std` compatible**: the crate supports `no_std` with `alloc`. Don't add `std` imports outside feature gates.
- **Release mode testing**: always `--release` for proof tests
- **Feature gating**: don't pull gated dependencies into default features
- **Clippy**: don't suppress warnings — fix the underlying issue

## Change Propagation

Changes to plonk ripple to downstream crates — `phoenix/circuits`, `poseidon-merkle`, `rusk` to name a few. Keep this in mind when making code changes, but no need to verify downstream for non-code changes.

## Git

**Branches**: branch from `main`. Don't push to `main` directly.
**Commits**: follow the style of recent commits in the repo.
