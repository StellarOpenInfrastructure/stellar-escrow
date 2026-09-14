/**
 * Builds (but does not sign or submit) a `create` invocation for the escrow contract
 * and prints the transaction XDR. Simulate/sign it with your wallet or the Stellar CLI.
 *
 * Usage: ESCROW_CONTRACT_ID=C... pnpm exec tsx examples/create-escrow.ts
 */
import { Account, BASE_FEE, Keypair, Networks, TransactionBuilder } from "@stellar/stellar-sdk";
import { EscrowClient } from "@stellar-escrow/sdk";

const contractId = process.env.ESCROW_CONTRACT_ID;
if (!contractId) throw new Error("ESCROW_CONTRACT_ID is required");

const buyer = Keypair.random().publicKey(); // replace with the buyer's public key
const client = new EscrowClient({ contractId, networkPassphrase: Networks.TESTNET });

const op = client.createOp({
  buyer,
  seller: Keypair.random().publicKey(),
  arbitrator: Keypair.random().publicKey(),
  asset: "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC", // native XLM SAC on testnet
  milestones: [
    { amount: 400_0000000n, description: "design approved" },
    { amount: 600_0000000n, description: "delivery accepted" },
  ],
  expiration: Math.floor(Date.now() / 1000) + 30 * 86_400,
});

const tx = new TransactionBuilder(new Account(buyer, "0"), {
  fee: BASE_FEE,
  networkPassphrase: Networks.TESTNET,
})
  .addOperation(op)
  .setTimeout(300)
  .build();

console.log(tx.toXDR());
