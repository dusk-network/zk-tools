<h1 align="center">ZK Tools</h1>

<p align="center"><em>One API. One circuit. Multiple proof systems.</em></p>

<p align="center">
  <a href="https://github.com/dusk-network/zk-tools/actions/workflows/ci.yml"><img alt="Build Status" src="https://github.com/dusk-network/zk-tools/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/dusk-network/zk-tools"><img alt="Repository" src="https://img.shields.io/badge/github-zk--tools-blueviolet?logo=github"></a>
</p>

This repository brings together the cryptographic tools used to build zero-knowledge applications. It includes:

- 🧩 **ZK Composer:** an arithmetic circuit API supporting both R1CS and PLONKish circuits.
- 🔐 **Groth16:** the Groth16 proving system for Composer circuits, including
  circuit-specific BLS12-381 Solidity verifier generation through EIP-2537.
- 🔐 **PLONK:** the PLONK proving system with custom gates, also using Composer circuits.
- 🧰 **Gadgets:** reusable Composer components, including:
  - ✍️ Schnorr signatures over the Jubjub elliptic curve.
  - #️⃣ Poseidon hashing.
  - 🌳 Poseidon Merkle trees.

Some code comes from external repositories, which were imported with their relevant Git histories and relocated under `crates/`. Original revisions and provenance are recorded in [`UPSTREAMS.md`](UPSTREAMS.md), while [`docs/upstream-imports.md`](docs/upstream-imports.md) documents the reproducible transformations required for future synchronization.

The imported crates use local path dependencies so they can be developed and tested together. This workspace also contains ongoing experimental changes and improvements.

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
    let expected_public_inputs = [BlsScalar::from(42u64)];
    assert_eq!(public_inputs.as_slice(), expected_public_inputs.as_slice());
      
    // Verify proof
    verifier
        .verify(&proof, &expected_public_inputs)
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
    assert_eq!(public_inputs.as_slice(), expected_public_inputs.as_slice());
    
    // Verify proof
    verifier
        .verify(&proof, &expected_public_inputs)
        .expect("proof should verify");
}
```

Run the complete, executable examples in release mode with `make examples`.
See [`crates/groth16/examples/circuit.rs`](crates/groth16/examples/circuit.rs)
and [`crates/plonk/examples/circuit.rs`](crates/plonk/examples/circuit.rs) for
more detail.

## Solidity verification

Groth16 verification keys can generate circuit-specific Solidity contracts for
EVM networks with the EIP-2537 BLS12-381 precompiles. Proofs are converted from
the crate's canonical 192-byte compressed format into a separate 512-byte
EIP-2537 transport before they are submitted to the contract.

See the [Solidity verification guide](https://dusk-network.github.io/zk-tools/solidity/)
for the generation workflow, verifier ABI, Foundry tests, compatibility limits,
and security boundaries.

## License

This project is licensed under the [Mozilla Public License 2.0](LICENSE).
