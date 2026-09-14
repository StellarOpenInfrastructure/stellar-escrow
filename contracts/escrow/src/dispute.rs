//! Disputes: either party freezes the escrow; the arbitrator splits remaining funds.
use soroban_sdk::{token, Address, Env, String};

use crate::errors::Error;
use crate::escrow::finalize_if_settled;
use crate::events;
use crate::storage;
use crate::types::{EscrowStatus, MilestoneStatus, BPS_DENOMINATOR};

pub fn open(env: &Env, escrow_id: u64, caller: Address, reason: String) -> Result<(), Error> {
    caller.require_auth();
    let mut escrow = storage::get_escrow(env, escrow_id)?;
    if caller != escrow.buyer && caller != escrow.seller {
        return Err(Error::Unauthorized);
    }
    if escrow.status != EscrowStatus::Funded {
        return Err(Error::InvalidStatus);
    }
    escrow.status = EscrowStatus::Disputed;
    storage::set_escrow(env, &escrow);
    events::Disputed {
        escrow_id,
        by: caller,
        reason,
    }
    .publish(env);
    Ok(())
}

pub fn resolve(env: &Env, escrow_id: u64, seller_bps: u32) -> Result<(), Error> {
    let mut escrow = storage::get_escrow(env, escrow_id)?;
    escrow.arbitrator.require_auth();
    if escrow.status != EscrowStatus::Disputed {
        return Err(Error::InvalidStatus);
    }
    if seller_bps > BPS_DENOMINATOR {
        return Err(Error::InvalidBasisPoints);
    }

    let remaining = escrow.remaining();
    // remaining ≤ amount ≤ i128::MAX, and seller_bps ≤ 10_000, so the product
    // cannot overflow i128 for any realistic token supply; checked anyway.
    let to_seller = remaining
        .checked_mul(seller_bps as i128)
        .ok_or(Error::Overflow)?
        / BPS_DENOMINATOR as i128;
    let to_buyer = remaining - to_seller;

    for index in 0..escrow.milestone_count {
        let mut m = storage::get_milestone(env, escrow_id, index)?;
        if m.status == MilestoneStatus::Pending || m.status == MilestoneStatus::Approved {
            // Milestone-level attribution of a split is ambiguous; mark refunded
            // and rely on escrow-level `released`/`refunded` totals for accounting.
            m.status = MilestoneStatus::Refunded;
            storage::set_milestone(env, escrow_id, &m);
        }
    }
    escrow.released = escrow
        .released
        .checked_add(to_seller)
        .ok_or(Error::Overflow)?;
    escrow.refunded = escrow
        .refunded
        .checked_add(to_buyer)
        .ok_or(Error::Overflow)?;
    escrow.status = EscrowStatus::Funded; // finalize_if_settled moves it to Completed
    storage::set_escrow(env, &escrow);

    let client = token::Client::new(env, &escrow.asset);
    if to_seller > 0 {
        client.transfer(&env.current_contract_address(), &escrow.seller, &to_seller);
    }
    if to_buyer > 0 {
        client.transfer(&env.current_contract_address(), &escrow.buyer, &to_buyer);
    }
    events::Resolved {
        escrow_id,
        to_seller,
        to_buyer,
    }
    .publish(env);
    finalize_if_settled(env, &mut escrow);
    Ok(())
}
