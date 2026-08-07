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
one another directly from this repository. Dependency versions, features,
optionality, and `default-features` settings remain unchanged.

The following dependency edges use local paths:

- `dusk-poseidon` -> `dusk-plonk`
- `poseidon-merkle` -> `dusk-merkle`
- `poseidon-merkle` -> `dusk-poseidon`
- `poseidon-merkle` -> `dusk-plonk`
- `jubjub-schnorr` -> `dusk-poseidon`
- `jubjub-schnorr` -> `dusk-plonk`

The upstream merkle repository's nested workspace manifest is omitted so its
two crates can be direct members of the root workspace. Consequently,
`poseidon-merkle`'s upstream `dusk-merkle.workspace = true` declaration is
expanded to the equivalent version-and-path dependency.

`plonkwasm` is intentionally excluded from the root workspace and continues to
use its published `dusk-plonk` dependency for now. No Rust source, test,
benchmark, example, asset, crate version, feature, or cryptographic behavior
was changed by the local-path integration.

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

The `jubjub-schnorr` `zk` feature checks and benchmarks are temporarily
omitted from the unified CI because that imported revision's witness-point API
is not compatible with the imported local `dusk-plonk` revision. Its non-ZK
feature matrices remain covered.

The imported `poseidon-merkle` benchmark is also temporarily omitted because
it still uses superseded Poseidon module paths and Merkle type signatures. Its
library and test suite remain covered by the unified CI.

The `dusk-plonk` `debug` feature is also omitted from the unified test matrix
because its imported CDF test does not preserve its temporary output directory.
Default and other optional-feature tests remain covered.
