help: ## Display this help screen
	@grep -h -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

test: ## Run tests with the BLST backend (release mode)
	@cargo test -p jubjub-schnorr --release --no-default-features \
		--features=bls-backend-blst,alloc,zk,rkyv-impl,serde

clippy: ## Run clippy
	@cargo clippy -p jubjub-schnorr --no-default-features \
		--features=bls-backend-blst,rkyv-impl,rkyv/size_32,zk,serde -- -D warnings

cq: ## Run code quality checks (formatting + clippy)
	@$(MAKE) fmt CHECK=1
	@$(MAKE) clippy

fmt: ## Format code
	@rustup component add --toolchain nightly rustfmt 2>/dev/null || true
	@cargo +nightly fmt --all $(if $(CHECK),-- --check,)

check: ## Type-check
	@cargo check -p jubjub-schnorr --no-default-features \
		--features=bls-backend-blst,zk,alloc,serde

doc: ## Generate docs
	@cargo doc --no-deps -p jubjub-schnorr --no-default-features \
		--features=bls-backend-blst,zk,alloc,serde

build-benches: ## Build benchmarks
	@cargo bench --no-run -p jubjub-schnorr --no-default-features \
		--features=bls-backend-blst,zk

no-std: ## Verify no_std + WASM compatibility
	@rustup target add wasm32-unknown-unknown 2>/dev/null || true
	@cargo build --release --no-default-features \
		--features=bls-backend-dusk,serde --target wasm32-unknown-unknown

clean: ## Clean build artifacts
	@cargo clean

.PHONY: help test clippy cq fmt check doc build-benches no-std clean
