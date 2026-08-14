.PHONY: help build test fmt clippy no-std build-benches doc examples solidity-test solidity-fmt website-check clean

help: ## Display this help screen
	@grep -h -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

build: ## Build all workspace crates
	# Consumers select a backend explicitly; repository automation uses BLST.
	cargo build --workspace --no-default-features --features=bls-backend-blst

test: ## Run the supported test matrix for every workspace crate
	# Keep repository tests on BLST while preserving both public backend features.
	cargo test -p dusk-zk-composer --release \
		--no-default-features \
		--features=bls-backend-blst,std,plonkish,r1cs,debug,rkyv-impl,zeroize
	cargo test -p dusk-groth16 --release --no-default-features \
		--features=bls-backend-blst,std,zeroize
	cargo test -p dusk-plonk --release \
		--no-default-features \
		--features=bls-backend-blst,std,debug,rkyv-impl,zeroize,legacy-proving
	cargo test -p dusk-poseidon --release --no-default-features \
		--features=bls-backend-blst,zk,encryption
	$(MAKE) -C crates/merkle test
	cargo test -p jubjub-schnorr --release --no-default-features \
		--features=bls-backend-blst,zk,alloc,rkyv-impl,serde
	cargo test -p plonkwasm --release --no-default-features \
		--features=bls-backend-blst,wasm-rayon

fmt: ## Format all workspace crates; use CHECK=1 to check only
	cargo +nightly fmt --all $(if $(CHECK),-- --check,)

clippy: ## Run the supported clippy matrix for every workspace crate
	cargo clippy -p dusk-zk-composer --no-default-features \
		--features=bls-backend-blst,std,plonkish,r1cs,debug,rkyv-impl,zeroize \
		--no-deps -- -D warnings
	cargo clippy -p dusk-groth16 --no-default-features \
		--features=bls-backend-blst,std,zeroize --no-deps -- -D warnings
	cargo clippy -p dusk-plonk --no-default-features \
		--features=bls-backend-blst,std,rkyv-impl,zeroize,legacy-proving \
		--no-deps -- -D warnings
	cargo clippy -p dusk-poseidon --no-default-features \
		--features=bls-backend-blst,zk,encryption --no-deps -- -D warnings
	cargo clippy -p dusk-merkle --features=rkyv-impl,size_32 --no-deps -- -D warnings
	cargo clippy -p poseidon-merkle --no-default-features \
		--features=bls-backend-blst,zk,rkyv-impl,size_32 --no-deps -- -D warnings
	cargo clippy -p jubjub-schnorr \
		--no-default-features \
		--features=bls-backend-blst,rkyv/size_32,zk,alloc,rkyv-impl,serde \
		--no-deps -- -D warnings
	cargo clippy -p plonkwasm --no-default-features \
		--features=bls-backend-blst,wasm-rayon --no-deps -- -D warnings

no-std: ## Check bare-metal and WASM with the portable Dusk backend
	$(MAKE) -C crates/composer no-std
	$(MAKE) -C crates/groth16 no-std
	$(MAKE) -C crates/plonk no-std
	$(MAKE) -C crates/poseidon no-std
	$(MAKE) -C crates/merkle no-std
	$(MAKE) -C crates/jubjub-schnorr no-std

build-benches: ## Compile benchmark targets without running them
	cargo bench -p dusk-groth16 --no-default-features \
		--features=bls-backend-blst,std --no-run
	cargo bench -p dusk-plonk --no-default-features \
		--features=bls-backend-blst,std --no-run
	cargo bench -p dusk-poseidon --no-default-features \
		--features=bls-backend-blst,zk,encryption --no-run
	cargo bench -p dusk-merkle --features=rkyv-impl,size_32 --no-run
	cargo bench -p poseidon-merkle --no-default-features \
		--features=bls-backend-blst,zk,rkyv-impl,size_32 --no-run
	cargo bench -p jubjub-schnorr --no-default-features \
		--features=bls-backend-blst,zk --no-run
	cargo bench -p plonkwasm --no-default-features \
		--features=bls-backend-blst,wasm-rayon --no-run

doc: ## Build documentation for every workspace crate
	RUSTDOCFLAGS="-D warnings" cargo doc -p dusk-zk-composer --no-deps --no-default-features \
		--features=bls-backend-blst,std,plonkish,r1cs
	RUSTDOCFLAGS="-D warnings" cargo doc -p dusk-groth16 --no-deps --no-default-features \
		--features=bls-backend-blst,std
	cargo rustdoc -p dusk-plonk --lib --no-default-features \
		--features=bls-backend-blst,std -- \
		--html-in-header crates/plonk/katex-header.html -D warnings
	cargo doc -p dusk-poseidon --no-deps --no-default-features \
		--features=bls-backend-blst,zk,encryption
	cargo doc -p dusk-merkle --no-deps
	cargo doc -p poseidon-merkle --no-deps --no-default-features \
		--features=bls-backend-blst
	RUSTDOCFLAGS="--html-in-header crates/jubjub-schnorr/katex-header.html" \
		cargo doc -p jubjub-schnorr --no-deps --no-default-features \
		--features=bls-backend-blst,zk,alloc,serde
	cargo doc -p plonkwasm --no-deps --no-default-features \
		--features=bls-backend-blst

examples: ## Build and run the proof-system examples
	cargo run --release -p dusk-groth16 --example circuit \
		--no-default-features --features=bls-backend-blst,std
	cargo build --release -p dusk-groth16 --example solidity \
		--no-default-features --features=bls-backend-blst,std
	cargo run --release -p dusk-plonk --example circuit \
		--no-default-features --features=bls-backend-blst,std

solidity-test: ## Run the BLS12-381 Groth16 verifier tests with Foundry
	forge test --root crates/groth16/tests/solidity

solidity-fmt: ## Format Solidity code; use CHECK=1 to check only
	forge fmt --root crates/groth16/tests/solidity $(if $(CHECK),--check,)

website-check: ## Validate the static documentation website
	python3 website/scripts/check_site.py

clean: ## Remove workspace build artifacts
	cargo clean
