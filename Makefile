.PHONY: help build test fmt clippy no-std build-benches doc examples clean

help: ## Display this help screen
	@grep -h -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

build: ## Build all workspace crates
	cargo build --workspace

test: ## Run the supported test matrix for every workspace crate
	cargo test -p dusk-plonk --release
	cargo test -p dusk-plonk --release \
		--features=debug,rkyv-impl,rkyv/size_32,zeroize,legacy-proving
	cargo test -p dusk-poseidon --release --all-features
	$(MAKE) -C crates/merkle test
	cargo test -p jubjub-schnorr --release --features=zk,alloc,serde
	cargo test -p jubjub-schnorr --no-default-features

fmt: ## Format all workspace crates; use CHECK=1 to check only
	cargo +nightly fmt --all $(if $(CHECK),-- --check,)

clippy: ## Run the supported clippy matrix for every workspace crate
	cargo clippy -p dusk-plonk --features=rkyv/size_32 --no-deps -- -D warnings
	cargo clippy -p dusk-poseidon --all-features --no-deps -- -D warnings
	cargo clippy -p dusk-poseidon --no-default-features --no-deps -- -D warnings
	cargo clippy -p dusk-merkle --features=rkyv-impl,size_32 --no-deps -- -D warnings
	cargo clippy -p dusk-merkle --no-default-features --no-deps -- -D warnings
	cargo clippy -p poseidon-merkle --features=zk,rkyv-impl,size_32 --no-deps -- -D warnings
	cargo clippy -p poseidon-merkle --no-default-features --no-deps -- -D warnings
	cargo clippy -p jubjub-schnorr \
		--features=rkyv/size_32,zk,alloc,serde --no-deps
	cargo clippy -p jubjub-schnorr --no-default-features --no-deps

no-std: ## Check the bare-metal and WASM configurations
	$(MAKE) -C crates/plonk no-std
	$(MAKE) -C crates/poseidon no-std
	$(MAKE) -C crates/merkle no-std
	$(MAKE) -C crates/jubjub-schnorr no-std

build-benches: ## Compile benchmark targets without running them
	cargo bench -p dusk-plonk --no-run
	cargo bench -p dusk-poseidon --all-features --no-run
	cargo bench -p dusk-merkle --features=rkyv-impl,size_32 --no-run
	cargo bench -p poseidon-merkle --features=zk,rkyv-impl,size_32 --no-run
	cargo bench -p jubjub-schnorr --features=zk --no-run

doc: ## Build documentation for every workspace crate
	cargo rustdoc -p dusk-plonk --lib -- \
		--html-in-header crates/plonk/katex-header.html -D warnings
	cargo doc -p dusk-poseidon --no-deps --all-features
	cargo doc -p dusk-merkle -p poseidon-merkle --no-deps
	RUSTDOCFLAGS="--html-in-header crates/jubjub-schnorr/katex-header.html" \
		cargo doc -p jubjub-schnorr --no-deps

examples: ## Build and run the PLONK example
	cargo run --release -p dusk-plonk --example circuit

clean: ## Remove workspace build artifacts
	cargo clean
