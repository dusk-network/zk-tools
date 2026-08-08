# ZK Composer

[![Build Status](https://github.com/dusk-network/zk-tools/actions/workflows/ci.yml/badge.svg)](https://github.com/dusk-network/zk-tools/actions/workflows/ci.yml)
[![Repository](https://img.shields.io/badge/github-zk--tools-blueviolet?logo=github)](https://github.com/dusk-network/zk-tools)

`dusk-zk-composer` provides the shared circuit-construction language used by
Dusk zero-knowledge proving systems. It does not depend on proving-system
crates. Instead, proving systems depend on this crate and select a backend when
constructing a circuit.

Reusable circuits implement `Circuit` with a generic `ComposerBackend`:

```rust
use dusk_zk_composer::prelude::*;

#[derive(Default)]
struct ProductCircuit {
    left: BlsScalar,
    right: BlsScalar,
}

impl Circuit for ProductCircuit {
    fn circuit<B: ComposerBackend>(
        &self,
        composer: &mut Composer<B>,
    ) -> Result<(), CircuitError> {
        let left = composer.append_witness(self.left);
        let right = composer.append_witness(self.right);
        composer.gate_mul(Constraint::new().mult(1).a(left).b(right));
        Ok(())
    }
}
```

The `plonkish` feature provides the current width-four PLONKish backend. The
`r1cs` feature provides a rank-1 backend that lowers the same arithmetic, range,
logic, and elliptic-curve identities into sparse rank-1 equations. Additional
backends can implement the same `ComposerBackend` contract without introducing
a dependency from the composer to a proving-system crate.

Finalized R1CS assignments use the order `(1, public inputs, Composer
witnesses, lowering auxiliaries)`. Sparse linear-combination terms are
normalized by variable index, and the value-independent shape digest commits
to variable counts, public-input counts, constraint order, variable indexes,
and coefficients. Shifted custom-gate identities retain the PLONKish padded
domain rule: the final row wraps only when the emitted row count is already a
power of two; otherwise it reads zero-valued padding.

The crate is `no_std` with `alloc`. Consumers must select exactly one BLS12-381
backend through `bls-backend-dusk` or `bls-backend-blst`.

## Features

- `alloc` enables circuit construction.
- `plonkish` enables `Plonkish`, PLONKish gates, circuit shapes, and compressed
  circuit descriptions. It implies `alloc` and is enabled by default.
- `r1cs` enables the `R1cs` backend, finalized rank-1 shapes, and assignments.
  It implies `alloc`.
- `std` enables standard-library support and implies `alloc`.
- `debug` enables CDF circuit debugging and implies `std` and `plonkish`.
- `test-api` exposes unstable raw hooks for adversarial testing. It is not a
  supported production API.
- `zeroize` enables best-effort clearing of finalized R1CS assignments and
  forwards scalar zeroization support to `dusk-curves`.
- `parallel` forwards Rayon support to the portable Dusk backend and cannot be
  combined with `bls-backend-blst`.
- `bls-backend-dusk` and `bls-backend-blst` select the BLS12-381 backend;
  consumers must enable exactly one.

## Licensing

This code is licensed under the Mozilla Public License Version 2.0 (MPL-2.0).
Please see the [license](https://github.com/dusk-network/zk-tools/blob/main/crates/composer/LICENSE)
for more information.
