//! Release approved milestones to the seller.
use soroban_sdk::{token, Address, Env};

use crate::errors::Error;
use crate::escrow::finalize_if_settled;
use crate::events;
use crate::storage;
use crate::types::{EscrowStatus, MilestoneStatus};

pub fn release(env: &Env, escrow_id: u64, index: u32, caller: Address) -> Result<(), Error> {
    caller.require_auth();
    let mut escrow = storage::get_escrow(env, escrow_id)?;
    if caller != escrow.buyer && caller != escrow.seller {
        return Err(Error::Unauthorized);
    }
    if escrow.status != EscrowStatus::Funded {
        return Err(Error::InvalidStatus);
    }
    let mut milestone = storage::get_milestone(env, escrow_id, index)?;
    if milestone.status != MilestoneStatus::Approved {
        return Err(Error::InvalidMilestoneStatus);
    }

    // Effects before interaction: update state, then transfer.
    milestone.status = MilestoneStatus::Released;
    escrow.released = escrow
        .released
        .checked_add(milestone.amount)
        .ok_or(Error::Overflow)?;
    storage::set_milestone(env, escrow_id, &milestone);
    storage::set_escrow(env, &escrow);

    token::Client::new(env, &escrow.asset).transfer(
        &env.current_contract_address(),
        &escrow.seller,
        &milestone.amount,
    );
    events::Released {
        escrow_id,
        index,
        amount: milestone.amount,
    }
    .publish(env);
    finalize_if_settled(env, &mut escrow);
    Ok(())
}
