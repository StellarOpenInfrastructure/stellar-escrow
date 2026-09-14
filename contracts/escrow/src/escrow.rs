//! Escrow lifecycle: create → fund → (milestones/release/refund/dispute) → completed.
use soroban_sdk::{token, Address, Env, Vec};

use crate::errors::Error;
use crate::events;
use crate::storage;
use crate::types::{
    Escrow, EscrowStatus, Milestone, MilestoneInput, MilestoneStatus, MAX_MILESTONES,
};

pub fn create(
    env: &Env,
    buyer: Address,
    seller: Address,
    arbitrator: Address,
    asset: Address,
    milestones: Vec<MilestoneInput>,
    expiration: u64,
) -> Result<u64, Error> {
    buyer.require_auth();

    if buyer == seller || buyer == arbitrator || seller == arbitrator {
        return Err(Error::PartiesNotDistinct);
    }
    if milestones.is_empty() || milestones.len() > MAX_MILESTONES {
        return Err(Error::InvalidMilestones);
    }
    let now = env.ledger().timestamp();
    if expiration <= now {
        return Err(Error::InvalidExpiration);
    }

    let mut total: i128 = 0;
    for m in milestones.iter() {
        if m.amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        total = total.checked_add(m.amount).ok_or(Error::Overflow)?;
    }

    let id = storage::next_id(env)?;
    let escrow = Escrow {
        id,
        buyer: buyer.clone(),
        seller: seller.clone(),
        arbitrator: arbitrator.clone(),
        asset: asset.clone(),
        amount: total,
        released: 0,
        refunded: 0,
        expiration,
        milestone_count: milestones.len(),
        status: EscrowStatus::Created,
        created_at: now,
    };
    storage::set_escrow(env, &escrow);
    for (index, m) in milestones.iter().enumerate() {
        storage::set_milestone(
            env,
            id,
            &Milestone {
                index: index as u32,
                amount: m.amount,
                description: m.description,
                status: MilestoneStatus::Pending,
            },
        );
    }
    events::Created {
        escrow_id: id,
        buyer,
        seller,
        arbitrator,
        asset,
        amount: total,
        expiration,
    }
    .publish(env);
    Ok(id)
}

pub fn fund(env: &Env, escrow_id: u64) -> Result<(), Error> {
    let mut escrow = storage::get_escrow(env, escrow_id)?;
    escrow.buyer.require_auth();
    if escrow.status != EscrowStatus::Created {
        return Err(Error::InvalidStatus);
    }
    if env.ledger().timestamp() >= escrow.expiration {
        return Err(Error::Expired);
    }
    // Pull the full amount from the buyer. The token contract enforces the
    // buyer's authorization for this transfer via `require_auth` on `from`.
    let contract = env.current_contract_address();
    token::Client::new(env, &escrow.asset).transfer(&escrow.buyer, &contract, &escrow.amount);
    escrow.status = EscrowStatus::Funded;
    storage::set_escrow(env, &escrow);
    events::Funded {
        escrow_id,
        amount: escrow.amount,
    }
    .publish(env);
    Ok(())
}

pub fn cancel(env: &Env, escrow_id: u64, caller: Address) -> Result<(), Error> {
    caller.require_auth();
    let mut escrow = storage::get_escrow(env, escrow_id)?;
    if caller != escrow.buyer && caller != escrow.seller {
        return Err(Error::Unauthorized);
    }
    if escrow.status != EscrowStatus::Created {
        return Err(Error::InvalidStatus);
    }
    escrow.status = EscrowStatus::Cancelled;
    storage::set_escrow(env, &escrow);
    events::Cancelled {
        escrow_id,
        by: caller,
    }
    .publish(env);
    Ok(())
}

/// Mark the escrow completed once nothing is left to distribute.
pub fn finalize_if_settled(env: &Env, escrow: &mut Escrow) {
    if escrow.remaining() == 0 && escrow.status != EscrowStatus::Completed {
        escrow.status = EscrowStatus::Completed;
        storage::set_escrow(env, escrow);
    }
}
