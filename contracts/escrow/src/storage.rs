use soroban_sdk::{contracttype, Env};

use crate::errors::Error;
use crate::types::{Escrow, Milestone};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// u64 counter of escrows created.
    Count,
    /// Escrow by id.
    Escrow(u64),
    /// Milestone by (escrow id, index).
    Milestone(u64, u32),
}

// TTL policy: persistent entries are extended whenever they are touched so an
// active escrow never expires from the ledger. Values are in ledgers
// (~5 s each): extend to ~30 days when fewer than ~7 days remain.
const TTL_THRESHOLD: u32 = 7 * 17_280;
const TTL_EXTEND_TO: u32 = 30 * 17_280;

pub fn count(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::Count).unwrap_or(0)
}

pub fn next_id(env: &Env) -> Result<u64, Error> {
    let id = count(env);
    let next = id.checked_add(1).ok_or(Error::Overflow)?;
    env.storage().instance().set(&DataKey::Count, &next);
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
    Ok(id)
}

pub fn get_escrow(env: &Env, id: u64) -> Result<Escrow, Error> {
    let key = DataKey::Escrow(id);
    let escrow: Escrow = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
    Ok(escrow)
}

pub fn set_escrow(env: &Env, escrow: &Escrow) {
    let key = DataKey::Escrow(escrow.id);
    env.storage().persistent().set(&key, escrow);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
}

pub fn get_milestone(env: &Env, escrow_id: u64, index: u32) -> Result<Milestone, Error> {
    let key = DataKey::Milestone(escrow_id, index);
    let m: Milestone = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::MilestoneNotFound)?;
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
    Ok(m)
}

pub fn set_milestone(env: &Env, escrow_id: u64, milestone: &Milestone) {
    let key = DataKey::Milestone(escrow_id, milestone.index);
    env.storage().persistent().set(&key, milestone);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
}
