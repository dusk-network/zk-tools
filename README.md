<h1 align="center">ZK Tools</h1>

<p align="center"><em>One API. Many proof systems.</em></p>

<p align="center">
  <a href="https://github.com/dusk-network/zk-tools/actions/workflows/ci.yml"><img alt="Build Status" src="https://github.com/dusk-network/zk-tools/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/dusk-network/zk-tools"><img alt="Repository" src="https://img.shields.io/badge/github-zk--tools-blueviolet?logo=github"></a>
</p>

This repository contains cryptographic tools used to develop zero-knowledge applications. These are the included tools:

- 🧩 **ZK composer:** an arithmetic circuit writer supporting both R1CS and Plonkish circuits.
- 🔐 **Groth16:** the Groth'16 proving scheme, used with circuits written by the ZK composer.
- 🔐 **Plonk:** the Plonk proving scheme with custom gates, used with circuits written by the ZK composer.
- 🧰 **Gadgets:** several gadgets written using the ZK composer. In particular:
  - ✍️ Schnorr signatures over the Jubjub elliptic curve.
  - #️⃣ Poseidon hashing function.
  - 🌳 Poseidon Merkle trees.

Some code comes from external repositories, which were imported with their relevant Git histories and relocated under `crates/`. Original revisions and provenance are recorded in [`UPSTREAMS.md`](UPSTREAMS.md), while [`docs/upstream-imports.md`](docs/upstream-imports.md) documents the reproducible transformations required for future synchronization.

Notice that the dependencies of the imported crates have been updated to use local paths. Furthermore, take into account that changes are continuously applied to introduce new experimental features and improvements.

> **⚠️ DISCLAIMER:** this workspace is currently experimental and intended for coordinated development. It is not published as a combined package, and the imported crates should not be released from this repository.

## Getting started

Circuits implement the shared `Circuit` trait, allowing the same definition to
be used with different proving systems. This example proves that two private
values multiply to a public result:

```rust,no_run
use dusk_groth16::Compiler as Groth16Compiler;
use dusk_plonk::prelude::{Compiler as PlonkCompiler, PublicParameters};
use dusk_zk_composer::prelude::*;
use rand_core::OsRng;

#[derive(Default)]
struct ProductCircuit {
    left: BlsScalar,
    right: BlsScalar,
    result: BlsScalar,
}

impl Circuit for ProductCircuit {
    fn circuit<B: ComposerBackend>(
        &self,
        composer: &mut Composer<B>,
    ) -> Result<(), CircuitError> {
        let left = composer.append_witness(self.left);
        let right = composer.append_witness(self.right);
        let product = composer.gate_mul(
            Constraint::new().mult(1).a(left).b(right),
        );
        let result = composer.append_public(self.result);
        composer.assert_equal(product, result);
        Ok(())
    }
}

fn main() {
    let circuit = ProductCircuit {
        left: BlsScalar::from(6u64),
        right: BlsScalar::from(7u64),
        result: BlsScalar::from(42u64),
    };

    // GROTH'16
    // Compute trusted setup and compile circuit. Groth16 uses a circuit-specific
    // trusted setup. Generate its randomness securely and destroy it after setup.
    let (prover, verifier) =
        Groth16Compiler::trusted_setup::<ProductCircuit, _>(&mut OsRng)
            .expect("setup should succeed");
    
    // Compute proof
    let (proof, public_inputs) = prover
        .prove(&mut OsRng, &circuit)
        .expect("proof generation should succeed");
      
    // Verify proof
    verifier
        .verify(&proof, &public_inputs)
        .expect("proof should verify");

    // PLONK
    // Generate trusted setup and compile circuit. Plonk compiles the circuit 
    // against reusable KZG public parameters.
    let pp = PublicParameters::setup(1 << 8, &mut OsRng)
        .expect("public-parameter setup should succeed"); 
    let (prover, verifier) =
        PlonkCompiler::compile::<ProductCircuit>(&pp, b"product-circuit")
            .expect("circuit compilation should succeed");
    
    // Compute proof
    let (proof, public_inputs) = prover
        .prove(&mut OsRng, &circuit)
        .expect("proof generation should succeed");
    
    // Verify proof
    verifier
        .verify(&proof, &public_inputs)
        .expect("proof should verify");
}
```

Run the complete, executable examples in release mode with `make examples`.
See [`crates/groth16/examples/circuit.rs`](crates/groth16/examples/circuit.rs)
and [`crates/plonk/examples/circuit.rs`](crates/plonk/examples/circuit.rs) for
more detail.

## License

This project is licensed under the [Mozilla Public License 2.0](LICENSE).
