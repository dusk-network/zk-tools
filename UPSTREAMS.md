# Imported upstream histories

`zk-tools` incorporates Git history from several independent Dusk
repositories. The revisions below identify the original upstream commits
before filtering. Historical paths were rewritten when the repositories were
imported, so the corresponding commit IDs in this repository are different.

## plonk

Local path: `crates/plonk/`

Upstream repository: `https://github.com/dusk-network/plonk`

Upstream branch: `master`

Imported upstream revision: `e9f4901aaa7438fb97e50bc378e073c053372446`

Import scope: complete repository

The upstream Git history is preserved in `zk-tools`. Commit IDs differ from
the original repository because historical paths were rewritten under
`crates/plonk/`.

## Poseidon252

Local path: `crates/poseidon/`

Upstream repository: `https://github.com/dusk-network/Poseidon252`

Upstream branch: `master`

Imported upstream revision: `cef0d2aa139b61e2ab69d42e359e099a551b4b79`

Import scope: complete repository

The upstream Git history is preserved in `zk-tools`. Commit IDs differ from
the original repository because historical paths were rewritten under
`crates/poseidon/`.

## merkle

Local path: `crates/merkle/`

Upstream repository: `https://github.com/dusk-network/merkle`

Upstream branch: `main`

Imported upstream revision: `dd33092b6e8e8c46f353b3cebdcfff4e522b2e29`

Import scope: complete repository

The complete repository, including both `dusk-merkle/` and
`poseidon-merkle/` and its repository-level files, was imported as one shared
history. Commit IDs differ from the original repository because historical
paths were rewritten under `crates/merkle/`.

## jubjub-schnorr

Local path: `crates/jubjub-schnorr/`

Upstream repository: `https://github.com/dusk-network/jubjub-schnorr`

Upstream branch: `main`

Imported upstream revision: `8bac114bca6255e49276378c6e686bb6841f8e25`

Import scope: complete repository

The upstream Git history is preserved in `zk-tools`. Commit IDs differ from
the original repository because historical paths were rewritten under
`crates/jubjub-schnorr/`.

## plonkwasm

Local path: `crates/plonkwasm/`

Upstream repository: `https://github.com/dusk-network/plonkweb`

Upstream branch: `main`

Imported upstream revision: `e269204f17d28159a4ace4aa2c0bff4a2df5ad85`

Upstream path: `plonkwasm/`

Import scope: `plonkwasm/` subtree only

Only commits and files relevant to the upstream `plonkwasm/` subtree are
preserved. Commit IDs differ from the original repository because unrelated
paths were removed and `plonkwasm/` was rewritten as `crates/plonkwasm/`.

## Monorepo integration changes

After import, repository integration was adjusted so the imported crates use
one another directly from this repository. Integration-specific dependency and
feature changes are recorded below and in the crate manifests.

The following dependency edges use local paths:

- `dusk-poseidon` -> `dusk-plonk`
- `poseidon-merkle` -> `dusk-merkle`
- `poseidon-merkle` -> `dusk-poseidon`
- `poseidon-merkle` -> `dusk-plonk`
- `jubjub-schnorr` -> `dusk-poseidon`
- `jubjub-schnorr` -> `dusk-plonk`
- `plonkwasm` -> `dusk-plonk`

The upstream merkle repository's nested workspace manifest is omitted so its
two crates can be direct members of the root workspace. Consequently,
`poseidon-merkle`'s upstream `dusk-merkle.workspace = true` declaration is
expanded to the equivalent version-and-path dependency.

`plonkwasm` is a root workspace member and uses the local `dusk-plonk` crate.
Its upstream `=0.23.0` version pin is omitted from the path dependency because
the imported local `dusk-plonk` crate remains at version `0.22.1`.

The local dependency graph exposed API and circuit-soundness differences
between the imported revisions. The following post-import compatibility
changes are intentionally part of the monorepo history:

- `dusk-poseidon` uses `dusk-plonk`'s canonically bound 250-bit truncation
  component. Native and circuit hash outputs are unchanged, but the circuit
  layout changes.
- `jubjub-schnorr` adapts to the fallible point-allocation API, constrains
  prover-controlled public keys and variable generators to the prime-order
  subgroup, rejects identity points where the native verifier does, and
  constrains variable-generator responses to canonical JubJub scalars. These
  are security-sensitive circuit changes.
- Circuit tests and benchmarks are adapted to the current point, Poseidon, and
  Merkle APIs. The Poseidon-Merkle benchmark data generation now uses the
  public domain-separated hash API.
- The PLONK debugger test preserves its caller's `CDF_OUTPUT` environment and
  explicitly cleans up its temporary directory.

These circuit-layout changes require regeneration of circuit-specific proving
and verifier keys and cached circuit descriptions. They do not require a new
universal SRS, provided the existing parameters have sufficient capacity.
Crate versions and feature definitions remain unchanged.

Nested upstream `.github/workflows/` files are omitted because GitHub Actions
only discovers workflows in the repository-root `.github/workflows/`
directory. They are replaced by the root `ci.yml`, which checks formatting,
builds and tests the workspace, runs supported clippy feature matrices, checks
no-std and WASM targets, compiles available benchmarks, builds documentation,
and runs the PLONK example.

The crate-local Cargo configurations that only supplied relative rustdoc
header paths are also omitted. Cargo does not discover them when invoked from
the workspace root, so the root Makefile supplies the correct workspace-relative
paths for the affected documentation builds.

Package profile sections from the imported PLONK and Poseidon manifests are
defined at the workspace root because Cargo ignores member profiles. The root
manifest documents the unified release and benchmark policy and retains
Poseidon's optimized development build through a package override.
