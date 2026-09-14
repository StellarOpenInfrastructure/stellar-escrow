# Quickstart

## Build and test the contract

```bash
make setup
make contract-test
make contract-build     # contracts/target/wasm32v1-none/release/stellar_escrow_contract.wasm
```

## Deploy to testnet

```bash
cargo install --locked stellar-cli
stellar keys generate --global deployer --network testnet
STELLAR_DEPLOYER=deployer make contract-deploy-testnet
```

## Create an escrow with the CLI

```bash
CONTRACT=$(cat .stellar/escrow-testnet.contract-id)
XLM=$(stellar contract id asset --asset native --network testnet)
stellar contract invoke --id "$CONTRACT" --source buyer --network testnet -- create \
  --buyer buyer --seller "$SELLER" --arbitrator "$ARB" --asset "$XLM" \
  --milestones '[{"amount":"4000000000","description":"design"},{"amount":"6000000000","description":"build"}]' \
  --expiration $(( $(date +%s) + 30*86400 ))
stellar contract invoke --id "$CONTRACT" --source buyer --network testnet -- fund --escrow_id 0
```

## Use the SDK

```ts
import { EscrowClient } from "@stellar-escrow/sdk";
const client = new EscrowClient({ contractId, networkPassphrase, rpcUrl });
const op = client.approveMilestoneOp(0, 0);
// add `op` to a transaction, prepareTransaction, sign as buyer, send
const escrow = await client.readEscrow(0);
```
