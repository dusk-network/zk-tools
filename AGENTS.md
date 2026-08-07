# zk-tools

## Purpose

This is a history-preserving monorepo for Dusk's core zero-knowledge tooling.
The imported Git histories remain available for `git log` and `git blame`; they
were rewritten only to relocate files under `crates/`.

## Structure

- `crates/plonk/` — `dusk-plonk`
- `crates/poseidon/` — `dusk-poseidon`
- `crates/merkle/` — the shared history of `dusk-merkle` and
  `poseidon-merkle`
- `crates/jubjub-schnorr/` — `jubjub-schnorr`
- `crates/plonkwasm/` — the filtered `plonkweb/plonkwasm/` subtree

The root Cargo workspace contains every crate above except `plonkwasm`, which
is intentionally excluded for now. Workspace crates use local path dependencies
where they depend on one another.

## Working in the repository

- Use the root `Makefile` as the source of truth for build, test, formatting,
  lint, no-std, benchmark, documentation, and example checks.
- Run cryptographic tests in release mode; the Makefile supplies the supported
  feature matrices.
- Keep changes focused and follow the more specific `AGENTS.md` in a crate when
  working below it.
- Treat cryptographic and serialization changes as compatibility-sensitive.

## Upstream history

`UPSTREAMS.md` records imported repositories and original revisions.
`docs/upstream-imports.md` records the exact deterministic `git filter-repo`
transformations. Future updates must use those same transformations in fresh,
full clones and merge the rewritten history without squashing. Do not replace
an upstream update with copied snapshots or submodules.
