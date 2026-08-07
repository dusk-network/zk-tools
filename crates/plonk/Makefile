help: ## Display this help screen
	@grep -h -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

test: ## Run tests with the BLST backend
	@cargo test -p dusk-plonk --release --no-default-features \
		--features=bls-backend-blst,std,legacy-proving,debug,rkyv-impl,zeroize

clippy: ## Run clippy
	@cargo clippy -p dusk-plonk --no-default-features \
		--features=bls-backend-blst,std,rkyv-impl,zeroize,legacy-proving -- -D warnings

fmt: ## Format code (requires nightly)
	@cargo +nightly fmt --all

bench: ## Run benchmarks
	@cargo bench -p dusk-plonk --no-default-features --features=bls-backend-blst,std

no-std: ## Verify no_std compatibility
	@rustup target add thumbv6m-none-eabi
	@cargo build --release --no-default-features --features bls-backend-dusk,alloc --target thumbv6m-none-eabi
	@cargo build --release --no-default-features --features bls-backend-dusk --target thumbv6m-none-eabi

doc: ## Generate documentation
	@cargo rustdoc -p dusk-plonk --lib --no-default-features \
		--features=bls-backend-blst,std -- --html-in-header katex-header.html -D warnings

doc-internal: ## Generate documentation with private items
	@cargo rustdoc -p dusk-plonk --lib --no-default-features \
		--features=bls-backend-blst,std -- --document-private-items -D warnings

doc-local: ## Open local documentation
	@RUSTDOCFLAGS="--html-in-header katex-header.html" \
		cargo doc -p dusk-plonk --no-deps --no-default-features \
		--features=bls-backend-blst,std --open

clean: ## Clean build artifacts
	@cargo clean

.PHONY: help test clippy fmt bench no-std doc doc-internal doc-local clean
