# zk-tools

## Purpose

This is a history-preserving monorepo for Dusk's core zero-knowledge tooling.
The imported Git histories remain available for `git log` and `git blame`; they
were rewritten only to relocate files under `crates/`.

## Structure

- `crates/composer/` — `dusk-zk-composer`, the shared circuit-construction API
- `crates/groth16/` — `dusk-groth16`
- `crates/plonk/` — `dusk-plonk`
- `crates/poseidon/` — `dusk-poseidon`
- `crates/merkle/` — the shared history of `dusk-merkle` and
  `poseidon-merkle`
- `crates/jubjub-schnorr/` — `jubjub-schnorr`
- `crates/plonkwasm/` — the filtered `plonkweb/plonkwasm/` subtree

The root Cargo workspace contains every crate above. Workspace crates use local
path dependencies where they depend on one another.

## Working in the repository

- Use the root `Makefile` as the source of truth for build, test, formatting,
  lint, no-std, benchmark, documentation, and example checks.
- Run cryptographic tests in release mode; the Makefile supplies the supported
  feature matrices.
- Keep changes focused and follow the more specific `AGENTS.md` in a crate when
  working below it.
- Treat cryptographic and serialization changes as compatibility-sensitive.
- Use the existing MPL-2.0 and Dusk Network copyright header on new Rust source
  files.

## Adding proof systems

- Keep circuit construction and reusable constraint semantics in
  `dusk-zk-composer`. Add a feature-gated Composer backend when a scheme needs a
  new constraint representation; do not introduce a proving-system dependency
  into Composer.
- Put each proving system in its own workspace crate and add it to the root
  Makefile's release-test, lint, no-std, benchmark-build, documentation, and
  example matrices as applicable.
- Reuse the same circuit definitions and test cases across proving systems.
  Tests should cover valid proofs, invalid witnesses and public inputs, stable
  circuit shapes, and malformed or incompatible serialized data.
- Document the setup trust model, public-input ordering, circuit-shape binding,
  serialization format, compatibility boundaries, and audit status. Do not
  imply MPC or ceremony support until contribution and verification flows are
  implemented and tested.
- Treat changes to shared identities, constraint lowering, public-input order,
  transcripts, or evaluation domains as affecting every proving system that
  consumes them, and run all affected release-mode test matrices.

## Upstream history

`UPSTREAMS.md` records imported repositories and original revisions.
`docs/upstream-imports.md` records the exact deterministic `git filter-repo`
transformations. Future updates must use those same transformations in fresh,
full clones and merge the rewritten history without squashing. Do not replace
an upstream update with copied snapshots or submodules.
