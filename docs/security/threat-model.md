# Threat model — stellar-escrow contract

> **NOT AUDITED — EXPERIMENTAL.** This document describes the intended security
> properties of the scaffold contract. It is not an audit and does not claim the
> contract is safe for production value.

## Assets at risk

| Asset                   | Holder                    | Exposure                                                    |
| ----------------------- | ------------------------- | ----------------------------------------------------------- |
| Escrowed token balance  | contract address          | full loss if release/refund logic or authorization is wrong |
| Buyer's token allowance | buyer                     | `fund` pulls exactly `amount`; no approvals are stored      |
| Milestone state         | ledger persistent entries | corruption changes who can be paid                          |
| Escrow counter          | instance storage          | reuse would let a new escrow overwrite an old id            |

## Trust assumptions

- The SEP-41 token contract behaves per spec (`transfer` moves exactly the
  amount and enforces `from` authorization). A malicious token can lie about
  balances; parties must agree on a trusted asset.
- The arbitrator is trusted to resolve disputes fairly. The contract cannot
  judge merit; it only enforces that _someone agreed upon at creation_ decides.
- Ledger timestamps are trustworthy within Stellar's consensus tolerance.

## Attack surfaces and mitigations

| Surface             | Threat                                         | Mitigation                                                                                                                                       | Status    |
| ------------------- | ---------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ | --------- |
| `create`            | wrong party pays                               | `buyer.require_auth()`; parties must be distinct                                                                                                 | ✅ tested |
| `fund`              | funding twice / after expiry                   | status must be `Created`; expiry checked                                                                                                         | ✅ tested |
| `approve_milestone` | seller self-approves                           | buyer-only auth                                                                                                                                  | ✅ tested |
| `release`           | double release / unapproved release / outsider | milestone must be `Approved`; state updated **before** transfer; caller must be buyer or seller and authorize                                    | ✅ tested |
| `refund`            | buyer reclaims early / during dispute          | requires `Funded` and `now ≥ expiration`; `Disputed` escrows cannot be refunded unilaterally                                                     | ✅ tested |
| `open_dispute`      | griefing by freezing forever                   | any party can freeze; only the arbitrator can unfreeze. **Known gap**: an unresponsive arbitrator locks funds — see roadmap (arbitrator timeout) | ⚠️        |
| `resolve_dispute`   | arbitrator steals                              | split is bounded to remaining funds by basis points; arbitrator cannot direct funds to a third address                                           | ✅ tested |
| Storage             | entries expire                                 | persistent TTL extended on every access (7d threshold → 30d)                                                                                     | ✅        |
| Arithmetic          | overflow                                       | `checked_add/mul`, `overflow-checks = true` in release profile                                                                                   | ✅ tested |

## Authorization model

| Function            | Who must authorize                        | Extra checks                                                   |
| ------------------- | ----------------------------------------- | -------------------------------------------------------------- |
| `create`            | buyer                                     | distinct parties, 1..32 positive milestones, future expiration |
| `fund`              | buyer (also enforced by token `transfer`) | status `Created`, not expired                                  |
| `cancel`            | caller ∈ {buyer, seller}                  | status `Created`                                               |
| `approve_milestone` | buyer                                     | status `Funded`, milestone `Pending`                           |
| `release`           | caller ∈ {buyer, seller}                  | status `Funded`, milestone `Approved`                          |
| `refund`            | buyer                                     | status `Funded`, expired, remaining > 0                        |
| `open_dispute`      | caller ∈ {buyer, seller}                  | status `Funded`                                                |
| `resolve_dispute`   | arbitrator                                | status `Disputed`, bps ≤ 10000                                 |

Authorization uses Soroban's `require_auth`; there is no signature parsing in
the contract.

## Replay attacks

Soroban authorization entries include nonces and expiration ledgers; a signed
auth cannot be replayed. Contract-level state transitions are idempotent-safe:
repeating any call after it succeeded fails on a status check rather than
moving funds again.

## Reentrancy considerations

Soroban prohibits reentrant contract calls at the host level. The contract
nonetheless follows checks-effects-interactions: state is written before each
token `transfer`, so even a hypothetical callback could not observe stale
state.

## Integer / overflow concerns

- Amounts are `i128`; sums use `checked_add`, split uses `checked_mul`.
- `resolve_dispute` rounds toward the buyer (integer division floor on the
  seller share). Tested in `resolve_all_to_buyer_and_edge_rounding`.
- Milestone count is bounded (32) to keep loops cheap and bounded.

## Storage concerns

- `Escrow` and `Milestone` entries live in **persistent** storage keyed by
  id; the counter lives in **instance** storage.
- No entry is ever deleted; completed escrows remain readable (audit trail).
  Storage growth is linear in escrows created — acceptable for the scaffold;
  archival/cleanup is a roadmap item.

## TTL considerations

Every read/write extends the entry's TTL when fewer than ~7 days remain, to
~30 days. An escrow untouched for >30 days becomes archived and must be
restored (`RestoreFootprint`) before use — clients should handle
`restore_required` from simulation. Long expirations (e.g. 90 days) therefore
require periodic touches or a longer extension (roadmap: configurable TTL).

## Upgradeability assumptions

**None.** The contract has no admin and no `upgrade` entrypoint. Bugs require
deploying a new contract; existing escrows complete on the old code. This is a
deliberate scaffold decision (ADR 0002) to avoid an admin key that could
rewrite escrow logic while funds are held.

## Oracle assumptions

None. Milestone completion is attested by the buyer (approval) or the
arbitrator (dispute), not by an oracle.

## Admin privileges

There is no admin role. The only privileged party per escrow is its
arbitrator, whose power is limited to splitting _that escrow's_ remaining
funds between _its_ buyer and seller.

## Emergency controls

There is no pause switch. Rationale: a pause key is an admin key. Parties can
protect themselves by choosing short expirations (buyer can refund) and a
trusted arbitrator. Roadmap: opt-in per-escrow "mutual cancel" and arbitrator
timeout.

## Known gaps (tracked in docs/drips-wave.md)

1. Unresponsive arbitrator locks disputed funds indefinitely.
2. No mutual early termination while funded.
3. Fixed TTL policy.
4. `account_merge`-style contract address reuse is not a concern, but token
   contracts with transfer fees or rebasing supply would break amount
   accounting.
5. No formal verification or fuzzing yet.
