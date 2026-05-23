# PlonkWasm

[![Crates.io](https://img.shields.io/crates/v/plonkwasm.svg)](https://crates.io/crates/plonkwasm)
[![docs.rs](https://img.shields.io/docsrs/plonkwasm.svg)](https://docs.rs/plonkwasm)
[![CI](https://github.com/dusk-network/plonkweb/actions/workflows/ci.yml/badge.svg)](https://github.com/dusk-network/plonkweb/actions/workflows/ci.yml)
[![Repository](https://img.shields.io/badge/github-plonkweb-blueviolet?logo=github)](https://github.com/dusk-network/plonkweb)

Reusable proof and verification helpers for WebAssembly frontends built on
[`dusk-plonk`](https://crates.io/crates/dusk-plonk).

> ⚠️ **DISCLAIMER:** this code is experimental, and thus, unstable. Use at
> your own risk.

## ✨ What It Does

- Computes PLONK proofs from deserialized prover keys and concrete circuit values.
- Verifies serialized proofs with deserialized verifier keys and public inputs.
- Encodes public inputs as concatenated 32-byte scalar bytes.
- Provides a compact JSON-oriented wasm ABI for JavaScript callers.
- Exposes an optional `wasm-rayon` feature for threaded wasm builds that need
  `dusk-plonk/std`.

The concrete circuit is still supplied by the crate that links the final wasm
binary. PLONK circuits are compiled into that binary, while `plonkwasm` supplies
the reusable proof plumbing.

## 🚀 Usage

Use `prove` with a deserialized prover, deterministic 32-byte RNG seed, and a
concrete circuit value. Use `verify` with a deserialized verifier:

```rust
let output = plonkwasm::prove(&prover, [9; 32], &circuit)?;

plonkwasm::verify(
    &verifier,
    &output.proof,
    &output.public_inputs,
)?;
# Ok::<(), String>(())
```

`ProofOutput::proof` contains serialized `dusk_plonk::Proof` bytes.
`ProofOutput::public_inputs` contains public inputs encoded as concatenated
32-byte scalar encodings in the same order returned by `dusk-plonk`.

## 🧬 Public Input Helpers

Use `serialize_public_inputs` and `deserialize_public_inputs` when a host needs
to move public inputs across the wasm boundary without carrying Rust types:

```rust
let bytes = plonkwasm::serialize_public_inputs(&public_inputs);
let restored = plonkwasm::deserialize_public_inputs(&bytes)?;
# Ok::<(), String>(())
```

## 🌐 WebAssembly ABI Helpers

The `wasm` module provides helpers for circuit-specific wasm crates that expose
JavaScript-callable functions. A wasm crate typically exports allocation/free
functions and delegates request handling to `respond_from_request`:

```rust
#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    plonkwasm::wasm::alloc(len)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn free(ptr: *mut u8, len: usize) {
    unsafe { plonkwasm::wasm::free(ptr, len) }
}
```

Responses are packed as `(ptr << 32) | len` and point to JSON bytes.

Successful response:

```json
{"ok":true,"result":{},"error":null}
```

Error response:

```json
{"ok":false,"result":null,"error":"message"}
```

## ⚡ Threaded Wasm

The optional `wasm-rayon` feature enables `dusk-plonk/std`, which is required
for the parallel code paths inside `dusk-plonk`.

Building a threaded wasm artifact can still require nightly Rust and
`-Z build-std`, depending on the final wasm crate and target flags. The
`plonkwasm` library itself is compatible with stable Rust 1.94.

## 📚 Documentation

- API docs: <https://docs.rs/plonkwasm>
- Repository: <https://github.com/dusk-network/plonkweb>

## 📜 License

This crate is licensed under the Mozilla Public License Version 2.0
(`MPL-2.0`). See [LICENSE](LICENSE) for the full license text.
