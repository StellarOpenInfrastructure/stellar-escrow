//! Milestone approval by the buyer.
use soroban_sdk::Env;

use crate::errors::Error;
use crate::events;
use crate::storage;
use crate::types::{EscrowStatus, MilestoneStatus};

pub fn approve(env: &Env, escrow_id: u64, index: u32) -> Result<(), Error> {
    let escrow = storage::get_escrow(env, escrow_id)?;
    escrow.buyer.require_auth();
    if escrow.status != EscrowStatus::Funded {
        return Err(Error::InvalidStatus);
    }
    let mut milestone = storage::get_milestone(env, escrow_id, index)?;
    if milestone.status != MilestoneStatus::Pending {
        return Err(Error::InvalidMilestoneStatus);
    }
    milestone.status = MilestoneStatus::Approved;
    storage::set_milestone(env, escrow_id, &milestone);
    events::Approved { escrow_id, index }.publish(env);
    Ok(())
}
