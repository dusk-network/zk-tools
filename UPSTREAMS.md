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

After import, Cargo dependency declarations were adjusted so the imported
crates use one another directly from this repository. Dependency versions,
features, optionality, and `default-features` settings remain unchanged.

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
