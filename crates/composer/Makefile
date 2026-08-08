.PHONY: help test fmt clippy no-std doc clean

help: ## Display this help screen
	@grep -h -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

test: ## Run release tests with the BLST backend
	cargo test -p dusk-zk-composer --release --no-default-features \
		--features=bls-backend-blst,std,plonkish,debug,rkyv-impl,zeroize

fmt: ## Format workspace code; use CHECK=1 to check only
	cargo +nightly fmt --all $(if $(CHECK),-- --check,)

clippy: ## Run the supported lint matrix
	cargo clippy -p dusk-zk-composer --no-default-features \
		--features=bls-backend-blst,std,plonkish,debug,rkyv-impl,zeroize \
		--no-deps -- -D warnings

no-std: ## Verify bare-metal builds with the portable Dusk backend
	rustup target add thumbv6m-none-eabi
	cargo build --release -p dusk-zk-composer --target thumbv6m-none-eabi \
		--no-default-features --features=bls-backend-dusk,plonkish,zeroize
	cargo build --release -p dusk-zk-composer --target thumbv6m-none-eabi \
		--no-default-features --features=bls-backend-dusk

doc: ## Generate warning-free public documentation
	RUSTDOCFLAGS="-D warnings" cargo doc -p dusk-zk-composer --no-deps \
		--no-default-features --features=bls-backend-blst,std,plonkish

clean: ## Remove workspace build artifacts
	cargo clean

