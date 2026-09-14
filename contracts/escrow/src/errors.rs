use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Escrow id does not exist.
    NotFound = 1,
    /// Milestone index out of range.
    MilestoneNotFound = 2,
    /// Operation not valid in the escrow's current status.
    InvalidStatus = 3,
    /// Operation not valid in the milestone's current status.
    InvalidMilestoneStatus = 4,
    /// Caller is not a party allowed to perform this operation.
    Unauthorized = 5,
    /// Amount must be positive.
    InvalidAmount = 6,
    /// At least one milestone is required (and at most MAX_MILESTONES).
    InvalidMilestones = 7,
    /// Expiration must be in the future.
    InvalidExpiration = 8,
    /// Escrow has expired; only refund is possible.
    Expired = 9,
    /// Escrow has not expired yet.
    NotExpired = 10,
    /// Arithmetic overflow.
    Overflow = 11,
    /// Basis points must be 0..=10000.
    InvalidBasisPoints = 12,
    /// Buyer, seller and arbitrator must be distinct.
    PartiesNotDistinct = 13,
}
