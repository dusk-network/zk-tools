# dusk-zk-composer

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

let mut composer = Composer::<Plonkish>::initialized();
ProductCircuit::default().circuit(&mut composer)?;
# Ok::<(), CircuitError>(())
```

The `plonkish` feature provides the current width-four PLONKish backend.
Additional backends can implement the same `ComposerBackend` contract without
introducing a dependency from the composer to a proving-system crate.

The crate is `no_std` with `alloc`. Consumers must select exactly one BLS12-381
backend through `bls-backend-dusk` or `bls-backend-blst`.

## Features

- `alloc` enables circuit construction.
- `plonkish` enables `Plonkish`, PLONKish gates, circuit shapes, and compressed
  circuit descriptions. It implies `alloc` and is enabled by default.
- `std` enables standard-library support and implies `alloc`.
- `debug` enables CDF circuit debugging and implies `std` and `plonkish`.
- `test-api` exposes unstable raw hooks for adversarial testing. It is not a
  supported production API.
- `bls-backend-dusk` and `bls-backend-blst` select the BLS12-381 backend;
  consumers must enable exactly one.
