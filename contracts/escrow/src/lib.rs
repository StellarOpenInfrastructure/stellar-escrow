//! # stellar-escrow
//!
//! Programmable B2B escrow for Soroban.
//!
//! **NOT AUDITED — EXPERIMENTAL.** Do not deploy to mainnet with real value.
//!
//! One escrow holds a SEP-41 token amount from a `buyer` on behalf of a `seller`,
//! split into milestones. The buyer approves milestones; the seller (or buyer)
//! releases approved milestones; an `arbitrator` resolves disputes; the buyer can
//! reclaim unreleased funds after `expiration`.
//!
//! The contract has **no admin and no upgrade path**. This is deliberate for the
//! initial scaffold (see `docs/security/threat-model.md`).
//!
//! Module map (one contract, one responsibility per module):
//! - [`escrow`]     — create, fund, cancel
//! - [`milestones`] — approve
//! - [`release`]    — pay approved milestones to the seller
//! - [`refund`]     — return unreleased funds to the buyer after expiration
//! - [`dispute`]    — open and resolve disputes via the arbitrator
#![no_std]

mod dispute;
mod errors;
mod escrow;
mod events;
mod milestones;
mod refund;
mod release;
mod storage;
mod types;

#[cfg(test)]
mod test;

pub use errors::Error;
pub use types::{Escrow, EscrowStatus, Milestone, MilestoneInput, MilestoneStatus};

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    // ---------------------------------------------------------------- escrow

    /// Create an escrow. Requires `buyer` authorization. Returns the escrow id.
    ///
    /// `milestones` amounts must be positive and sum to the escrow total.
    /// `expiration` is a unix timestamp strictly in the future.
    pub fn create(
        env: Env,
        buyer: Address,
        seller: Address,
        arbitrator: Address,
        asset: Address,
        milestones: Vec<MilestoneInput>,
        expiration: u64,
    ) -> Result<u64, Error> {
        escrow::create(
            &env, buyer, seller, arbitrator, asset, milestones, expiration,
        )
    }

    /// Transfer the full amount from the buyer into the contract. Requires `buyer`.
    pub fn fund(env: Env, escrow_id: u64) -> Result<(), Error> {
        escrow::fund(&env, escrow_id)
    }

    /// Cancel an unfunded escrow. Requires `buyer` or `seller`.
    pub fn cancel(env: Env, escrow_id: u64, caller: Address) -> Result<(), Error> {
        escrow::cancel(&env, escrow_id, caller)
    }

    // ------------------------------------------------------------ milestones

    /// Mark a milestone as approved for release. Requires `buyer`.
    pub fn approve_milestone(env: Env, escrow_id: u64, index: u32) -> Result<(), Error> {
        milestones::approve(&env, escrow_id, index)
    }

    // --------------------------------------------------------------- release

    /// Pay an approved milestone to the seller. Requires `buyer` or `seller`.
    pub fn release(env: Env, escrow_id: u64, index: u32, caller: Address) -> Result<(), Error> {
        release::release(&env, escrow_id, index, caller)
    }

    // ---------------------------------------------------------------- refund

    /// After expiration, return all unreleased funds to the buyer. Requires `buyer`.
    pub fn refund(env: Env, escrow_id: u64) -> Result<i128, Error> {
        refund::refund(&env, escrow_id)
    }

    // --------------------------------------------------------------- dispute

    /// Freeze the escrow pending arbitration. Requires `buyer` or `seller`.
    pub fn open_dispute(
        env: Env,
        escrow_id: u64,
        caller: Address,
        reason: String,
    ) -> Result<(), Error> {
        dispute::open(&env, escrow_id, caller, reason)
    }

    /// Split remaining funds: `seller_bps` basis points (0–10000) to the seller, the
    /// rest to the buyer. Requires `arbitrator`.
    pub fn resolve_dispute(env: Env, escrow_id: u64, seller_bps: u32) -> Result<(), Error> {
        dispute::resolve(&env, escrow_id, seller_bps)
    }

    // ----------------------------------------------------------------- views

    pub fn get_escrow(env: Env, escrow_id: u64) -> Result<Escrow, Error> {
        storage::get_escrow(&env, escrow_id)
    }

    pub fn get_milestone(env: Env, escrow_id: u64, index: u32) -> Result<Milestone, Error> {
        storage::get_milestone(&env, escrow_id, index)
    }

    /// Number of escrows ever created.
    pub fn count(env: Env) -> u64 {
        storage::count(&env)
    }

    /// Contract version, for clients to gate features on.
    pub fn version() -> u32 {
        1
    }
}
