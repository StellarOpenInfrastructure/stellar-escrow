//! Buyer refund of unreleased funds after expiration.
use soroban_sdk::{token, Env};

use crate::errors::Error;
use crate::escrow::finalize_if_settled;
use crate::events;
use crate::storage;
use crate::types::{EscrowStatus, MilestoneStatus};

pub fn refund(env: &Env, escrow_id: u64) -> Result<i128, Error> {
    let mut escrow = storage::get_escrow(env, escrow_id)?;
    escrow.buyer.require_auth();
    // A disputed escrow is settled by the arbitrator, never by unilateral refund.
    if escrow.status != EscrowStatus::Funded {
        return Err(Error::InvalidStatus);
    }
    if env.ledger().timestamp() < escrow.expiration {
        return Err(Error::NotExpired);
    }

    let amount = escrow.remaining();
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    // Every milestone that is not yet released is marked refunded.
    for index in 0..escrow.milestone_count {
        let mut m = storage::get_milestone(env, escrow_id, index)?;
        if m.status == MilestoneStatus::Pending || m.status == MilestoneStatus::Approved {
            m.status = MilestoneStatus::Refunded;
            storage::set_milestone(env, escrow_id, &m);
        }
    }
    escrow.refunded = escrow.refunded.checked_add(amount).ok_or(Error::Overflow)?;
    storage::set_escrow(env, &escrow);

    token::Client::new(env, &escrow.asset).transfer(
        &env.current_contract_address(),
        &escrow.buyer,
        &amount,
    );
    events::Refunded { escrow_id, amount }.publish(env);
    finalize_if_settled(env, &mut escrow);
    Ok(amount)
}
