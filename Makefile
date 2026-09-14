.PHONY: setup dev test lint lint-fix typecheck build clean contract-build contract-test contract-lint contract-fmt contract-deploy-testnet help

# ---- everything -------------------------------------------------------------
setup: ## Install JS dependencies and check the Rust toolchain
	pnpm install
	rustup target add wasm32v1-none
	@command -v stellar >/dev/null 2>&1 || echo "note: install Stellar CLI for deployment: cargo install --locked stellar-cli"

dev: ## Watch SDK tests
	pnpm run dev

test: contract-test ## Run contract and SDK tests
	pnpm run test

lint: contract-lint ## Lint contracts and SDK
	pnpm run lint

lint-fix: contract-fmt ## Fix formatting and lint issues
	pnpm run lint:fix

typecheck: ## Type-check the SDK
	pnpm run typecheck

build: contract-build ## Build contracts (wasm) and SDK
	pnpm run build

clean: ## Remove build output
	pnpm run clean
	cd contracts && cargo clean

# ---- contracts --------------------------------------------------------------
contract-build: ## Build optimized wasm into contracts/target/wasm32v1-none/release
	cd contracts && cargo build --release --target wasm32v1-none

contract-test: ## Run contract unit tests
	cd contracts && cargo test --workspace

contract-lint: ## cargo fmt --check and clippy with warnings as errors
	cd contracts && cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings

contract-fmt: ## Format Rust code
	cd contracts && cargo fmt --all

contract-deploy-testnet: contract-build ## Deploy to testnet (requires Stellar CLI and STELLAR_DEPLOYER identity)
	./scripts/deploy-testnet.sh

help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-24s\033[0m %s\n", $$1, $$2}'
