/** Mirrors `contracts/escrow/src/types.rs`. Amounts are token base units (bigint). */
export type EscrowStatus = "Created" | "Funded" | "Disputed" | "Completed" | "Cancelled";
export type MilestoneStatus = "Pending" | "Approved" | "Released" | "Refunded";

export interface MilestoneInput {
  amount: bigint;
  description: string;
}

export interface Milestone {
  index: number;
  amount: bigint;
  description: string;
  status: MilestoneStatus;
}

export interface Escrow {
  id: bigint;
  buyer: string;
  seller: string;
  arbitrator: string;
  asset: string;
  amount: bigint;
  released: bigint;
  refunded: bigint;
  /** Unix seconds. */
  expiration: bigint;
  milestoneCount: number;
  status: EscrowStatus;
  createdAt: bigint;
}

/** Contract error codes (see `contracts/escrow/src/errors.rs`). */
export const ESCROW_ERRORS: Record<number, string> = {
  1: "NotFound",
  2: "MilestoneNotFound",
  3: "InvalidStatus",
  4: "InvalidMilestoneStatus",
  5: "Unauthorized",
  6: "InvalidAmount",
  7: "InvalidMilestones",
  8: "InvalidExpiration",
  9: "Expired",
  10: "NotExpired",
  11: "Overflow",
  12: "InvalidBasisPoints",
  13: "PartiesNotDistinct",
};

export const MAX_MILESTONES = 32;
export const BPS_DENOMINATOR = 10_000;
