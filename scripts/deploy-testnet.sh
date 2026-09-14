#!/usr/bin/env bash
# Deploy the escrow contract to Stellar testnet with the Stellar CLI.
#
# Prerequisites:
#   cargo install --locked stellar-cli
#   stellar keys generate --global deployer --network testnet   # funds via friendbot
#
# Usage:
#   STELLAR_DEPLOYER=deployer ./scripts/deploy-testnet.sh
#
# The identity name is read from the environment; no secret keys are handled by this script.
set -euo pipefail

IDENTITY="${STELLAR_DEPLOYER:-deployer}"
WASM="contracts/target/wasm32v1-none/release/stellar_escrow_contract.wasm"

if ! command -v stellar >/dev/null 2>&1; then
  echo "stellar CLI not found. Install with: cargo install --locked stellar-cli" >&2
  exit 1
fi
if [ ! -f "$WASM" ]; then
  echo "wasm not found at $WASM — run 'make contract-build' first" >&2
  exit 1
fi

echo "Deploying $WASM to testnet with identity '$IDENTITY'..."
CONTRACT_ID=$(stellar contract deploy \
  --wasm "$WASM" \
  --source "$IDENTITY" \
  --network testnet)

echo "Contract deployed: $CONTRACT_ID"
mkdir -p .stellar
echo "$CONTRACT_ID" > .stellar/escrow-testnet.contract-id
echo "Saved to .stellar/escrow-testnet.contract-id (git-ignored). Record it in docs/deployments.md if this is a shared deployment."
