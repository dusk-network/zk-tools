# Reproducing upstream history transformations

Upstream histories must be filtered with exactly the same path transformations
before a future update is merged into `zk-tools`. Reusing the transformations
keeps already imported upstream ancestry identical and gives Git the correct
merge base, so local monorepo changes remain intact except for ordinary merge
conflicts.

The initial imports used `git filter-repo` version
`a40bce548d2c` and the two metadata-preservation flags shown in every command:

- `--preserve-commit-hashes` leaves literal references to old commit hashes in
  commit messages unchanged.
- `--preserve-commit-encoding` leaves original commit-message encodings
  unchanged.

These flags are part of the reproducible filtering recipe. Run each command at
the root of a separate, disposable, full (non-shallow) clone of its upstream
repository. Never filter an upstream working repository or `zk-tools` itself.

## plonk

- Upstream: `https://github.com/dusk-network/plonk`
- Branch: `master`
- Scope: complete repository
- Transformation: prefix every historical path with `crates/plonk/`

```sh
git filter-repo \
    --preserve-commit-hashes \
    --preserve-commit-encoding \
    --to-subdirectory-filter crates/plonk
```

## Poseidon252

- Upstream: `https://github.com/dusk-network/Poseidon252`
- Branch: `master`
- Scope: complete repository
- Transformation: prefix every historical path with `crates/poseidon/`

```sh
git filter-repo \
    --preserve-commit-hashes \
    --preserve-commit-encoding \
    --to-subdirectory-filter crates/poseidon
```

## merkle

- Upstream: `https://github.com/dusk-network/merkle`
- Branch: `main`
- Scope: complete repository, including both crates and repository-root files
- Transformation: prefix every historical path with `crates/merkle/`

```sh
git filter-repo \
    --preserve-commit-hashes \
    --preserve-commit-encoding \
    --to-subdirectory-filter crates/merkle
```

The `dusk-merkle/` and `poseidon-merkle/` directories share this one filtered
history and must not be split into separate imports.

## jubjub-schnorr

- Upstream: `https://github.com/dusk-network/jubjub-schnorr`
- Branch: `main`
- Scope: complete repository
- Transformation: prefix every historical path with `crates/jubjub-schnorr/`

```sh
git filter-repo \
    --preserve-commit-hashes \
    --preserve-commit-encoding \
    --to-subdirectory-filter crates/jubjub-schnorr
```

## plonkweb / plonkwasm

- Upstream: `https://github.com/dusk-network/plonkweb`
- Branch: `main`
- Scope: `plonkwasm/` subtree only
- Transformation: retain `plonkwasm/` and rename it to
  `crates/plonkwasm/`

```sh
git filter-repo \
    --preserve-commit-hashes \
    --preserve-commit-encoding \
    --path plonkwasm/ \
    --path-rename plonkwasm/:crates/plonkwasm/
```

The path selection is independent from the rename: both options are required.
Commits that do not affect `plonkwasm/` are intentionally removed. Therefore,
if the latest upstream commit has no `plonkwasm/` change, the rewritten branch
tip can correspond to an older original commit while its `crates/plonkwasm/`
tree still matches the subtree at the recorded upstream revision.

## Future update procedure

For each update:

1. Make a new full clone of the original upstream and check out the intended
   canonical branch and revision.
2. Apply the exact command above for that upstream, including the same path,
   trailing slashes, and metadata-preservation flags.
3. Verify the rewritten subtree against the original tree, accounting only for
   the documented path transformation.
4. Fetch the rewritten branch into `zk-tools` and merge it without squashing.
5. Resolve only ordinary conflicts between newer upstream changes and local
   monorepo changes, then update `UPSTREAMS.md` with the new original upstream
   revision.

Do not use a different prefix, split a previously combined history, retain
additional `plonkweb` paths, or introduce a different filtering recipe for a
later update.
