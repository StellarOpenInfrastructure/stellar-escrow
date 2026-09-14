use soroban_sdk::{contracttype, Address, String};

/// Upper bound keeps storage entries small and iteration cheap.
pub const MAX_MILESTONES: u32 = 32;
pub const BPS_DENOMINATOR: u32 = 10_000;

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    /// Created, not yet funded.
    Created,
    /// Funds held by the contract.
    Funded,
    /// Frozen pending arbitration.
    Disputed,
    /// All funds distributed (released and/or refunded).
    Completed,
    /// Cancelled before funding.
    Cancelled,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MilestoneStatus {
    Pending,
    Approved,
    Released,
    /// Returned to the buyer by refund or dispute resolution.
    Refunded,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneInput {
    pub amount: i128,
    pub description: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    pub index: u32,
    pub amount: i128,
    pub description: String,
    pub status: MilestoneStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Escrow {
    pub id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub arbitrator: Address,
    /// SEP-41 token contract address.
    pub asset: Address,
    /// Sum of milestone amounts.
    pub amount: i128,
    /// Amount paid to the seller so far.
    pub released: i128,
    /// Amount returned to the buyer so far.
    pub refunded: i128,
    /// Unix timestamp after which the buyer may refund unreleased funds.
    pub expiration: u64,
    pub milestone_count: u32,
    pub status: EscrowStatus,
    /// Ledger timestamp of creation.
    pub created_at: u64,
}

impl Escrow {
    /// Funds still held by the contract for this escrow.
    pub fn remaining(&self) -> i128 {
        self.amount - self.released - self.refunded
    }
}
