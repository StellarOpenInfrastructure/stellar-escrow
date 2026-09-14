import { Address, nativeToScVal, scValToNative, xdr } from "@stellar/stellar-sdk";
import type { Escrow, EscrowStatus, Milestone, MilestoneInput, MilestoneStatus } from "./types.js";

export function addressVal(address: string): xdr.ScVal {
  return new Address(address).toScVal();
}

export function u64Val(value: bigint | number): xdr.ScVal {
  return nativeToScVal(BigInt(value), { type: "u64" });
}

export function u32Val(value: number): xdr.ScVal {
  return nativeToScVal(value, { type: "u32" });
}

export function i128Val(value: bigint): xdr.ScVal {
  return nativeToScVal(value, { type: "i128" });
}

export function stringVal(value: string): xdr.ScVal {
  return nativeToScVal(value, { type: "string" });
}

/** `Vec<MilestoneInput>` — struct fields are encoded as a map with symbol keys, sorted. */
export function milestoneInputsVal(milestones: MilestoneInput[]): xdr.ScVal {
  return xdr.ScVal.scvVec(
    milestones.map((m) =>
      xdr.ScVal.scvMap([
        new xdr.ScMapEntry({ key: xdr.ScVal.scvSymbol("amount"), val: i128Val(m.amount) }),
        new xdr.ScMapEntry({
          key: xdr.ScVal.scvSymbol("description"),
          val: stringVal(m.description),
        }),
      ]),
    ),
  );
}

type Native = Record<string, unknown>;

function enumName(value: unknown): string {
  // Unit enum variants decode as a one-element array of the variant symbol, or a plain string.
  if (Array.isArray(value)) return String(value[0]);
  return String(value);
}

export function decodeEscrow(val: xdr.ScVal): Escrow {
  const n = scValToNative(val) as Native;
  return {
    id: BigInt(n.id as bigint),
    buyer: String(n.buyer),
    seller: String(n.seller),
    arbitrator: String(n.arbitrator),
    asset: String(n.asset),
    amount: BigInt(n.amount as bigint),
    released: BigInt(n.released as bigint),
    refunded: BigInt(n.refunded as bigint),
    expiration: BigInt(n.expiration as bigint),
    milestoneCount: Number(n.milestone_count),
    status: enumName(n.status) as EscrowStatus,
    createdAt: BigInt(n.created_at as bigint),
  };
}

export function decodeMilestone(val: xdr.ScVal): Milestone {
  const n = scValToNative(val) as Native;
  return {
    index: Number(n.index),
    amount: BigInt(n.amount as bigint),
    description: String(n.description),
    status: enumName(n.status) as MilestoneStatus,
  };
}
