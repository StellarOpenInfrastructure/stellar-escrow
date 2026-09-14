import { Contract, rpc, TransactionBuilder, type xdr } from "@stellar/stellar-sdk";
import {
  addressVal,
  decodeEscrow,
  decodeMilestone,
  milestoneInputsVal,
  stringVal,
  u32Val,
  u64Val,
} from "./encoding.js";
import {
  BPS_DENOMINATOR,
  ESCROW_ERRORS,
  MAX_MILESTONES,
  type Escrow,
  type Milestone,
  type MilestoneInput,
} from "./types.js";

export interface EscrowClientOptions {
  contractId: string;
  networkPassphrase: string;
  /** Required for `read*` helpers; invocation builders work offline. */
  rpcUrl?: string;
  allowHttp?: boolean;
}

export class EscrowContractError extends Error {
  constructor(
    readonly code: number,
    readonly name_: string,
  ) {
    super(`escrow contract error ${code} (${name_})`);
    this.name = "EscrowContractError";
  }
}

/**
 * Builds contract invocations for the escrow contract and reads its state via RPC.
 *
 * The client never signs. Callers add the returned operation to a transaction,
 * simulate it (`rpc.Server.prepareTransaction`), sign with the party's wallet and submit.
 */
export class EscrowClient {
  readonly contract: Contract;
  private readonly server?: rpc.Server;

  constructor(private readonly opts: EscrowClientOptions) {
    this.contract = new Contract(opts.contractId);
    if (opts.rpcUrl)
      this.server = new rpc.Server(opts.rpcUrl, { allowHttp: opts.allowHttp ?? false });
  }

  // ------------------------------------------------------------ invocations

  createOp(params: {
    buyer: string;
    seller: string;
    arbitrator: string;
    asset: string;
    milestones: MilestoneInput[];
    /** Unix seconds. */
    expiration: bigint | number;
  }): xdr.Operation {
    if (params.milestones.length === 0 || params.milestones.length > MAX_MILESTONES) {
      throw new RangeError(`milestones must contain 1..${MAX_MILESTONES} entries`);
    }
    for (const m of params.milestones)
      if (m.amount <= 0n) throw new RangeError("milestone amounts must be positive");
    return this.contract.call(
      "create",
      addressVal(params.buyer),
      addressVal(params.seller),
      addressVal(params.arbitrator),
      addressVal(params.asset),
      milestoneInputsVal(params.milestones),
      u64Val(params.expiration),
    );
  }

  fundOp(escrowId: bigint | number): xdr.Operation {
    return this.contract.call("fund", u64Val(escrowId));
  }

  cancelOp(escrowId: bigint | number, caller: string): xdr.Operation {
    return this.contract.call("cancel", u64Val(escrowId), addressVal(caller));
  }

  approveMilestoneOp(escrowId: bigint | number, index: number): xdr.Operation {
    return this.contract.call("approve_milestone", u64Val(escrowId), u32Val(index));
  }

  releaseOp(escrowId: bigint | number, index: number, caller: string): xdr.Operation {
    return this.contract.call("release", u64Val(escrowId), u32Val(index), addressVal(caller));
  }

  refundOp(escrowId: bigint | number): xdr.Operation {
    return this.contract.call("refund", u64Val(escrowId));
  }

  openDisputeOp(escrowId: bigint | number, caller: string, reason: string): xdr.Operation {
    return this.contract.call(
      "open_dispute",
      u64Val(escrowId),
      addressVal(caller),
      stringVal(reason),
    );
  }

  resolveDisputeOp(escrowId: bigint | number, sellerBps: number): xdr.Operation {
    if (!Number.isInteger(sellerBps) || sellerBps < 0 || sellerBps > BPS_DENOMINATOR) {
      throw new RangeError(`sellerBps must be an integer in 0..${BPS_DENOMINATOR}`);
    }
    return this.contract.call("resolve_dispute", u64Val(escrowId), u32Val(sellerBps));
  }

  // ------------------------------------------------------------------ reads

  async readEscrow(escrowId: bigint | number): Promise<Escrow> {
    return decodeEscrow(
      await this.simulateRead(this.contract.call("get_escrow", u64Val(escrowId))),
    );
  }

  async readMilestone(escrowId: bigint | number, index: number): Promise<Milestone> {
    return decodeMilestone(
      await this.simulateRead(this.contract.call("get_milestone", u64Val(escrowId), u32Val(index))),
    );
  }

  async readCount(): Promise<bigint> {
    const val = await this.simulateRead(this.contract.call("count"));
    if (val.type !== "scvU64") throw new Error(`unexpected count type ${val.type}`);
    return BigInt(val.u64);
  }

  /** Simulate a read-only invocation from a throwaway source account and return the result value. */
  private async simulateRead(op: xdr.Operation): Promise<xdr.ScVal> {
    if (!this.server) throw new Error("rpcUrl is required for reads");
    // Any funded account works as a simulation source; we use the contract's own address
    // via a synthetic account with sequence 0, which RPC accepts for simulation.
    const source = await this.server.getAccount(READ_SOURCE_ACCOUNT).catch(() => null);
    if (!source) throw new Error("read source account is not available on this network");
    const tx = new TransactionBuilder(source, {
      fee: "100",
      networkPassphrase: this.opts.networkPassphrase,
    })
      .addOperation(op)
      .setTimeout(30)
      .build();
    const sim = await this.server.simulateTransaction(tx);
    if (rpc.Api.isSimulationError(sim)) throw mapSimulationError(sim.error);
    if (!sim.result) throw new Error("simulation returned no result");
    return sim.result.retval;
  }
}

/** Well-known funded account used purely as a simulation source (testnet friendbot root). Override per network via `READ_SOURCE_ACCOUNT` if needed. */
export const READ_SOURCE_ACCOUNT = "GAIH3ULLFQ4DGSECF2AR555KZ4KNDGEKN4AFI4SU2M7B43MGK3QJZNSR";

const ERROR_RE = /Error\(Contract, #(\d+)\)/;

export function mapSimulationError(message: string): Error {
  const m = ERROR_RE.exec(message);
  if (m) {
    const code = Number(m[1]);
    return new EscrowContractError(code, ESCROW_ERRORS[code] ?? "Unknown");
  }
  return new Error(message);
}
