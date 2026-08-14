.PHONY: help test fmt clippy no-std build-benches doc examples solidity-test solidity-fmt clean

help: ## Display this help screen
	@grep -h -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

test: ## Run release tests with the BLST backend
	cargo test -p dusk-groth16 --release --no-default-features \
		--features=bls-backend-blst,std,zeroize

fmt: ## Format workspace code; use CHECK=1 to check only
	cargo +nightly fmt --all $(if $(CHECK),-- --check,)

clippy: ## Run the supported lint matrix
	cargo clippy -p dusk-groth16 --no-default-features \
		--features=bls-backend-blst,std,zeroize --no-deps -- -D warnings

no-std: ## Verify a bare-metal build with the portable Dusk backend
	rustup target add thumbv6m-none-eabi
	cargo build --release -p dusk-groth16 --target thumbv6m-none-eabi \
		--no-default-features --features=bls-backend-dusk,alloc,zeroize

build-benches: ## Compile benchmark targets without running them
	cargo bench -p dusk-groth16 --no-default-features \
		--features=bls-backend-blst,std --no-run

doc: ## Generate warning-free public documentation
	RUSTDOCFLAGS="-D warnings" cargo doc -p dusk-groth16 --no-deps \
		--no-default-features --features=bls-backend-blst,std

examples: ## Run the circuit example and build the Solidity exporter
	cargo run --release -p dusk-groth16 --example circuit \
		--no-default-features --features=bls-backend-blst,std
	cargo build --release -p dusk-groth16 --example solidity \
		--no-default-features --features=bls-backend-blst,std

solidity-test: ## Run the EIP-2537 verifier tests with Foundry
	forge test --root tests/solidity

solidity-fmt: ## Format Solidity code; use CHECK=1 to check only
	forge fmt --root tests/solidity $(if $(CHECK),--check,)

clean: ## Remove workspace build artifacts
	cargo clean
