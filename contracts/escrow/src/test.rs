#![cfg(test)]

use soroban_sdk::testutils::{Address as _, Events, Ledger};
use soroban_sdk::{
    map, symbol_short, token, vec, Address, Env, IntoVal, Map, String, Symbol, Val, Vec,
};

use crate::{
    Error, EscrowContract, EscrowContractClient, EscrowStatus, MilestoneInput, MilestoneStatus,
};

const DAY: u64 = 86_400;

struct Setup<'a> {
    env: Env,
    client: EscrowContractClient<'a>,
    token: token::Client<'a>,
    buyer: Address,
    seller: Address,
    arbitrator: Address,
    asset: Address,
}

fn setup() -> Setup<'static> {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000_000);

    let contract_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let asset = env.register_stellar_asset_contract_v2(admin).address();
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    token::StellarAssetClient::new(&env, &asset).mint(&buyer, &1_000_000);
    let token = token::Client::new(&env, &asset);

    Setup {
        env,
        client,
        token,
        buyer,
        seller,
        arbitrator,
        asset,
    }
}

fn milestones(env: &Env, amounts: &[i128]) -> Vec<MilestoneInput> {
    let mut v = Vec::new(env);
    for a in amounts {
        v.push_back(MilestoneInput {
            amount: *a,
            description: String::from_str(env, "work"),
        });
    }
    v
}

fn create_default(s: &Setup) -> u64 {
    s.client.create(
        &s.buyer,
        &s.seller,
        &s.arbitrator,
        &s.asset,
        &milestones(&s.env, &[400, 600]),
        &(1_000_000 + 30 * DAY),
    )
}

// ------------------------------------------------------------------ create

#[test]
fn create_stores_escrow_and_milestones() {
    let s = setup();
    let id = create_default(&s);
    assert_eq!(id, 0);
    assert_eq!(s.client.count(), 1);

    let e = s.client.get_escrow(&id);
    assert_eq!(e.amount, 1_000);
    assert_eq!(e.status, EscrowStatus::Created);
    assert_eq!(e.milestone_count, 2);
    assert_eq!(e.remaining(), 1_000);

    let m1 = s.client.get_milestone(&id, &1);
    assert_eq!(m1.amount, 600);
    assert_eq!(m1.status, MilestoneStatus::Pending);

    // Second escrow gets the next id.
    assert_eq!(create_default(&s), 1);
    assert_eq!(s.client.version(), 1);
}

#[test]
fn create_requires_buyer_auth() {
    let s = setup();
    create_default(&s);
    let auths = s.env.auths();
    assert_eq!(auths[0].0, s.buyer);
}

#[test]
fn create_rejects_bad_inputs() {
    let s = setup();
    let exp = 1_000_000 + DAY;
    let ms = milestones(&s.env, &[100]);

    assert_eq!(
        s.client
            .try_create(&s.buyer, &s.buyer, &s.arbitrator, &s.asset, &ms, &exp),
        Err(Ok(Error::PartiesNotDistinct))
    );
    assert_eq!(
        s.client.try_create(
            &s.buyer,
            &s.seller,
            &s.arbitrator,
            &s.asset,
            &Vec::new(&s.env),
            &exp
        ),
        Err(Ok(Error::InvalidMilestones))
    );
    assert_eq!(
        s.client.try_create(
            &s.buyer,
            &s.seller,
            &s.arbitrator,
            &s.asset,
            &milestones(&s.env, &[100, 0]),
            &exp
        ),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        s.client.try_create(
            &s.buyer,
            &s.seller,
            &s.arbitrator,
            &s.asset,
            &ms,
            &1_000_000
        ),
        Err(Ok(Error::InvalidExpiration))
    );
    let too_many = milestones(&s.env, &[1; 33]);
    assert_eq!(
        s.client.try_create(
            &s.buyer,
            &s.seller,
            &s.arbitrator,
            &s.asset,
            &too_many,
            &exp
        ),
        Err(Ok(Error::InvalidMilestones))
    );
    let overflow = milestones(&s.env, &[i128::MAX, 1]);
    assert_eq!(
        s.client.try_create(
            &s.buyer,
            &s.seller,
            &s.arbitrator,
            &s.asset,
            &overflow,
            &exp
        ),
        Err(Ok(Error::Overflow))
    );
}

// -------------------------------------------------------------------- fund

#[test]
fn fund_moves_tokens_into_contract() {
    let s = setup();
    let id = create_default(&s);
    s.client.fund(&id);
    assert_eq!(s.token.balance(&s.buyer), 999_000);
    assert_eq!(s.token.balance(&s.client.address), 1_000);
    assert_eq!(s.client.get_escrow(&id).status, EscrowStatus::Funded);

    // Cannot fund twice.
    assert_eq!(s.client.try_fund(&id), Err(Ok(Error::InvalidStatus)));
    assert_eq!(s.client.try_fund(&99), Err(Ok(Error::NotFound)));
}

#[test]
fn fund_rejected_after_expiration() {
    let s = setup();
    let id = create_default(&s);
    s.env.ledger().set_timestamp(1_000_000 + 31 * DAY);
    assert_eq!(s.client.try_fund(&id), Err(Ok(Error::Expired)));
}

#[test]
fn cancel_only_before_funding_by_a_party() {
    let s = setup();
    let id = create_default(&s);
    assert_eq!(
        s.client.try_cancel(&id, &s.arbitrator),
        Err(Ok(Error::Unauthorized))
    );
    s.client.cancel(&id, &s.seller);
    assert_eq!(s.client.get_escrow(&id).status, EscrowStatus::Cancelled);
    assert_eq!(s.client.try_fund(&id), Err(Ok(Error::InvalidStatus)));

    let id2 = create_default(&s);
    s.client.fund(&id2);
    assert_eq!(
        s.client.try_cancel(&id2, &s.buyer),
        Err(Ok(Error::InvalidStatus))
    );
}

// ------------------------------------------------------- approve + release

#[test]
fn approve_and_release_milestone_pays_seller() {
    let s = setup();
    let id = create_default(&s);
    s.client.fund(&id);

    // Cannot release before approval.
    assert_eq!(
        s.client.try_release(&id, &0, &s.seller),
        Err(Ok(Error::InvalidMilestoneStatus))
    );
    s.client.approve_milestone(&id, &0);
    assert_eq!(
        s.client.get_milestone(&id, &0).status,
        MilestoneStatus::Approved
    );
    // Double approval fails.
    assert_eq!(
        s.client.try_approve_milestone(&id, &0),
        Err(Ok(Error::InvalidMilestoneStatus))
    );

    s.client.release(&id, &0, &s.seller);
    assert_eq!(s.token.balance(&s.seller), 400);
    assert_eq!(s.token.balance(&s.client.address), 600);
    let e = s.client.get_escrow(&id);
    assert_eq!(e.released, 400);
    assert_eq!(e.status, EscrowStatus::Funded);

    // Double release fails; unknown milestone fails; outsider fails.
    assert_eq!(
        s.client.try_release(&id, &0, &s.seller),
        Err(Ok(Error::InvalidMilestoneStatus))
    );
    assert_eq!(
        s.client.try_release(&id, &5, &s.seller),
        Err(Ok(Error::MilestoneNotFound))
    );
    assert_eq!(
        s.client.try_release(&id, &1, &s.arbitrator),
        Err(Ok(Error::Unauthorized))
    );

    // Releasing the last milestone completes the escrow.
    s.client.approve_milestone(&id, &1);
    s.client.release(&id, &1, &s.buyer);
    let e = s.client.get_escrow(&id);
    assert_eq!(e.status, EscrowStatus::Completed);
    assert_eq!(e.remaining(), 0);
    assert_eq!(s.token.balance(&s.seller), 1_000);
}

#[test]
fn approve_requires_funded_status() {
    let s = setup();
    let id = create_default(&s);
    assert_eq!(
        s.client.try_approve_milestone(&id, &0),
        Err(Ok(Error::InvalidStatus))
    );
}

// ------------------------------------------------------------------ refund

#[test]
fn refund_after_expiration_returns_unreleased_funds() {
    let s = setup();
    let id = create_default(&s);
    s.client.fund(&id);
    s.client.approve_milestone(&id, &0);
    s.client.release(&id, &0, &s.seller);

    assert_eq!(s.client.try_refund(&id), Err(Ok(Error::NotExpired)));
    s.env.ledger().set_timestamp(1_000_000 + 30 * DAY);
    assert_eq!(s.client.refund(&id), 600);
    assert_eq!(s.token.balance(&s.buyer), 999_600);
    assert_eq!(s.token.balance(&s.client.address), 0);

    let e = s.client.get_escrow(&id);
    assert_eq!(e.status, EscrowStatus::Completed);
    assert_eq!(e.refunded, 600);
    assert_eq!(
        s.client.get_milestone(&id, &1).status,
        MilestoneStatus::Refunded
    );
    assert_eq!(
        s.client.get_milestone(&id, &0).status,
        MilestoneStatus::Released
    );

    // Nothing left: second refund fails on status.
    assert_eq!(s.client.try_refund(&id), Err(Ok(Error::InvalidStatus)));
}

#[test]
fn refund_not_allowed_while_disputed_or_unfunded() {
    let s = setup();
    let id = create_default(&s);
    s.env.ledger().set_timestamp(1_000_000 + 30 * DAY);
    assert_eq!(s.client.try_refund(&id), Err(Ok(Error::InvalidStatus)));

    let s = setup();
    let id = create_default(&s);
    s.client.fund(&id);
    s.client
        .open_dispute(&id, &s.seller, &String::from_str(&s.env, "late"));
    s.env.ledger().set_timestamp(1_000_000 + 30 * DAY);
    assert_eq!(s.client.try_refund(&id), Err(Ok(Error::InvalidStatus)));
}

// ----------------------------------------------------------------- dispute

#[test]
fn dispute_freezes_and_arbitrator_splits() {
    let s = setup();
    let id = create_default(&s);
    s.client.fund(&id);
    s.client.approve_milestone(&id, &0);
    s.client.release(&id, &0, &s.seller); // 400 to seller, 600 remaining

    assert_eq!(
        s.client
            .try_open_dispute(&id, &s.arbitrator, &String::from_str(&s.env, "x")),
        Err(Ok(Error::Unauthorized))
    );
    s.client
        .open_dispute(&id, &s.buyer, &String::from_str(&s.env, "defective"));
    assert_eq!(s.client.get_escrow(&id).status, EscrowStatus::Disputed);

    // Frozen: no approvals, releases or new disputes.
    assert_eq!(
        s.client.try_approve_milestone(&id, &1),
        Err(Ok(Error::InvalidStatus))
    );
    assert_eq!(
        s.client.try_release(&id, &1, &s.seller),
        Err(Ok(Error::InvalidStatus))
    );
    assert_eq!(
        s.client
            .try_open_dispute(&id, &s.seller, &String::from_str(&s.env, "again")),
        Err(Ok(Error::InvalidStatus))
    );
    assert_eq!(
        s.client.try_resolve_dispute(&id, &10_001),
        Err(Ok(Error::InvalidBasisPoints))
    );

    // 25% of the remaining 600 to the seller.
    s.client.resolve_dispute(&id, &2_500);
    assert_eq!(s.token.balance(&s.seller), 550);
    assert_eq!(s.token.balance(&s.buyer), 999_450);
    assert_eq!(s.token.balance(&s.client.address), 0);
    let e = s.client.get_escrow(&id);
    assert_eq!(e.status, EscrowStatus::Completed);
    assert_eq!(e.released, 550);
    assert_eq!(e.refunded, 450);
    assert_eq!(
        s.client.get_milestone(&id, &1).status,
        MilestoneStatus::Refunded
    );

    // Cannot resolve twice.
    assert_eq!(
        s.client.try_resolve_dispute(&id, &0),
        Err(Ok(Error::InvalidStatus))
    );
}

#[test]
fn resolve_requires_arbitrator_auth() {
    let s = setup();
    let id = create_default(&s);
    s.client.fund(&id);
    s.client
        .open_dispute(&id, &s.buyer, &String::from_str(&s.env, "r"));
    s.client.resolve_dispute(&id, &10_000);
    let auths = s.env.auths();
    let (addr, _) = auths.last().expect("auth recorded");
    assert_eq!(*addr, s.arbitrator);
    assert_eq!(s.token.balance(&s.seller), 1_000);
}

#[test]
fn resolve_all_to_buyer_and_edge_rounding() {
    let s = setup();
    let id = s.client.create(
        &s.buyer,
        &s.seller,
        &s.arbitrator,
        &s.asset,
        &milestones(&s.env, &[3]),
        &(1_000_000 + DAY),
    );
    s.client.fund(&id);
    s.client
        .open_dispute(&id, &s.buyer, &String::from_str(&s.env, "r"));
    // 1/3 of 3 with 3333 bps → 0 (rounds down); remainder to buyer.
    s.client.resolve_dispute(&id, &3_333);
    assert_eq!(s.token.balance(&s.seller), 0);
    assert_eq!(s.token.balance(&s.buyer), 1_000_000);
    assert_eq!(s.client.get_escrow(&id).status, EscrowStatus::Completed);
}

// ------------------------------------------------------------------ events

#[test]
fn emits_events_for_state_changes() {
    // `env.events().all()` only holds events of the most recent invocation.
    let s = setup();
    let id = create_default(&s);
    let created = s.env.events().all().filter_by_contract(&s.client.address);
    let created_data: Map<Symbol, Val> = map![
        &s.env,
        (symbol_short!("amount"), 1_000i128.into_val(&s.env)),
        (
            Symbol::new(&s.env, "arbitrator"),
            s.arbitrator.clone().into_val(&s.env)
        ),
        (symbol_short!("asset"), s.asset.clone().into_val(&s.env)),
        (symbol_short!("buyer"), s.buyer.clone().into_val(&s.env)),
        (
            Symbol::new(&s.env, "expiration"),
            (1_000_000u64 + 30 * DAY).into_val(&s.env)
        ),
        (symbol_short!("seller"), s.seller.clone().into_val(&s.env)),
    ];
    assert_eq!(
        created,
        vec![
            &s.env,
            (
                s.client.address.clone(),
                (symbol_short!("created"), id).into_val(&s.env),
                created_data.into_val(&s.env),
            ),
        ]
    );
    s.client.fund(&id);
    let funded_data: Map<Symbol, Val> = map![
        &s.env,
        (symbol_short!("amount"), 1_000i128.into_val(&s.env))
    ];
    assert_eq!(
        s.env.events().all().filter_by_contract(&s.client.address),
        vec![
            &s.env,
            (
                s.client.address.clone(),
                (symbol_short!("funded"), id).into_val(&s.env),
                funded_data.into_val(&s.env),
            ),
        ]
    );
}
