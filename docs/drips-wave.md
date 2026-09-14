# Drips Wave issue roadmap — stellar-escrow

The scaffold contract is functional and tested but **not audited**. This
backlog hardens it, extends the SDK, and adds the tooling an audit requires.
Contract changes must update `docs/security/threat-model.md`.

## How to use this file

Maintainers copy an entry below into a GitHub issue using the **Drips Wave task** template, apply the `wave` label plus the complexity label, and add the `area:*` label. Contributors claim an issue by commenting on it. Every entry is independently actionable: none depends on another entry being finished first unless stated in _Technical notes_.

| Complexity | Label          | Expected points | Typical effort        |
| ---------- | -------------- | --------------- | --------------------- |
| Trivial    | `wave-trivial` | 2               | a few hours           |
| Medium     | `wave-medium`  | 5               | 1–3 days              |
| High       | `wave-high`    | 10              | up to one sprint week |

Totals: **11 trivial**, **10 medium**, **5 high**.

## Index

### Trivial

1. [Add `get_milestones` view returning all milestones of an escrow](#1-add-getmilestones-view-returning-all-milestones-of-an-escrow) — `area:contract`
2. [Document deployments in docs/deployments.md with a template](#2-document-deployments-in-docsdeploymentsmd-with-a-template) — `area:docs`
3. [Add authorization matrix test](#3-add-authorization-matrix-test) — `area:contract`
4. [Emit `version` in the `created` event and document event schemas](#4-emit-version-in-the-created-event-and-document-event-schemas) — `area:docs`
5. [SDK event decoders](#5-sdk-event-decoders) — `area:sdk`
6. [Add wasm size check to CI](#6-add-wasm-size-check-to-ci) — `area:devops`
7. [Improve error messages in the SDK with hints](#7-improve-error-messages-in-the-sdk-with-hints) — `area:sdk`
8. [Example: full lifecycle script against a local network](#8-example-full-lifecycle-script-against-a-local-network) — `area:docs`
9. [Bound `reason` string length in `open_dispute`](#9-bound-reason-string-length-in-opendispute) — `area:security`
10. [Add `.vscode` recommendations for rust-analyzer and ESLint](#10-add-vscode-recommendations-for-rust-analyzer-and-eslint) — `area:devops`
11. [Typed contract client generation from the wasm spec](#11-typed-contract-client-generation-from-the-wasm-spec) — `area:sdk`

### Medium

12. [Arbitrator timeout: buyer refund path for stale disputes](#12-arbitrator-timeout-buyer-refund-path-for-stale-disputes) — `area:contract`
13. [Mutual early termination](#13-mutual-early-termination) — `area:contract`
14. [Platform fee on release](#14-platform-fee-on-release) — `area:contract`
15. [Configurable TTL policy and `extend_ttl` entrypoint](#15-configurable-ttl-policy-and-extendttl-entrypoint) — `area:contract`
16. [Fuzz and property tests for the split and lifecycle](#16-fuzz-and-property-tests-for-the-split-and-lifecycle) — `area:contract`
17. [SDK: prepare-sign-send helper with pluggable signer](#17-sdk-prepare-sign-send-helper-with-pluggable-signer) — `area:sdk`
18. [Live testnet integration test workflow](#18-live-testnet-integration-test-workflow) — `area:devops`
19. [Multi-asset milestone support](#19-multi-asset-milestone-support) — `area:contract`
20. [Event indexing example with stellar-contract-observer](#20-event-indexing-example-with-stellar-contract-observer) — `area:docs`
21. [Gas and storage benchmarks](#21-gas-and-storage-benchmarks) — `area:devops`

### High

22. [Pluggable dispute resolution (multi-arbitrator / DAO vote)](#22-pluggable-dispute-resolution-multi-arbitrator--dao-vote) — `area:contract`
23. [Reference web UI for parties](#23-reference-web-ui-for-parties) — `area:frontend`
24. [Formal specification and model checking of the state machine](#24-formal-specification-and-model-checking-of-the-state-machine) — `area:security`
25. [Recurring / subscription escrows](#25-recurring--subscription-escrows) — `area:contract`
26. [Independent security audit preparation package](#26-independent-security-audit-preparation-package) — `area:security`

## Trivial issues

### 1. Add `get_milestones` view returning all milestones of an escrow

- **Complexity:** Trivial (`wave-trivial`)
- **Expected points:** 2
- **Area:** `area:contract`

**Problem**

Clients must call `get_milestone` once per index to render an escrow.

**Scope**

Add a view returning `Vec<Milestone>` (bounded by `MAX_MILESTONES`) and an SDK `readMilestones` helper.

**Acceptance criteria**

- [ ] View returns milestones in index order
- [ ] SDK helper decodes the vector
- [ ] Contract test added

**Technical notes**

Iterate `0..milestone_count`; no storage changes.

**Files likely affected**

- `contracts/escrow/src/lib.rs`
- `contracts/escrow/src/test.rs`
- `sdk/src/client.ts`
- `docs/api/README.md`

**Tests required**

Contract unit test and SDK decoder test.

**Documentation required**

Contract interface table.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 2. Document deployments in docs/deployments.md with a template

- **Complexity:** Trivial (`wave-trivial`)
- **Expected points:** 2
- **Area:** `area:docs`

**Problem**

Shared testnet deployments are not tracked anywhere.

**Scope**

Create `docs/deployments.md` with a table (network, contract id, wasm hash, deployer, date, notes) and have `scripts/deploy-testnet.sh` print a ready-to-paste row.

**Acceptance criteria**

- [ ] Template exists
- [ ] Deploy script prints the row including the wasm hash from `stellar contract install` output or sha256 of the file

**Technical notes**

Use `sha256sum` on the wasm.

**Files likely affected**

- `docs/deployments.md`
- `scripts/deploy-testnet.sh`
- `README.md`

**Tests required**

None.

**Documentation required**

The new page.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 3. Add authorization matrix test

- **Complexity:** Trivial (`wave-trivial`)
- **Expected points:** 2
- **Area:** `area:contract`

**Problem**

Authorization is tested per function but not exhaustively for every (caller × function) pair.

**Scope**

Add a table-driven test that calls every mutating function with each of buyer, seller, arbitrator and a stranger, asserting `Unauthorized` or auth failure where expected.

**Acceptance criteria**

- [ ] Matrix covers all 8 mutating functions × 4 callers
- [ ] Uses `mock_auths` for precise auth, not `mock_all_auths`

**Technical notes**

Use `soroban_sdk::testutils::MockAuth` to authorize a specific address.

**Files likely affected**

- `contracts/escrow/src/test.rs`
- `docs/security/threat-model.md`

**Tests required**

The matrix test.

**Documentation required**

Threat model authorization section links to the test.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 4. Emit `version` in the `created` event and document event schemas

- **Complexity:** Trivial (`wave-trivial`)
- **Expected points:** 2
- **Area:** `area:docs`

**Problem**

Indexers cannot tell which contract version produced an event.

**Scope**

Add a `version: u32` field to `Created` and write `docs/api/events.md` listing every event's topics and data fields.

**Acceptance criteria**

- [ ] Event includes version
- [ ] Events page complete for all 8 events
- [ ] SDK exports event names

**Technical notes**

Update the event test expectations.

**Files likely affected**

- `contracts/escrow/src/events.rs`
- `contracts/escrow/src/escrow.rs`
- `contracts/escrow/src/test.rs`
- `docs/api/events.md`

**Tests required**

Updated event test.

**Documentation required**

Events page.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 5. SDK event decoders

- **Complexity:** Trivial (`wave-trivial`)
- **Expected points:** 2
- **Area:** `area:sdk`

**Problem**

Applications must hand-parse contract events.

**Scope**

Add `decodeEvent(event)` in the SDK mapping topic symbols to typed objects for all eight events.

**Acceptance criteria**

- [ ] All events decode with unit tests built from hand-constructed XDR
- [ ] Unknown events return `null`

**Technical notes**

Topics are `[Symbol name, u64 escrow_id]`; data is a map.

**Files likely affected**

- `sdk/src/events.ts`
- `sdk/src/events.test.ts`
- `sdk/src/index.ts`

**Tests required**

Unit tests per event.

**Documentation required**

README SDK section.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 6. Add wasm size check to CI

- **Complexity:** Trivial (`wave-trivial`)
- **Expected points:** 2
- **Area:** `area:devops`

**Problem**

Contract size affects deployment cost; regressions go unnoticed.

**Scope**

Add a CI step that fails if the wasm exceeds a threshold (start at 64 KB) and prints the size in the job summary.

**Acceptance criteria**

- [ ] Step present in ci.yml
- [ ] Threshold documented

**Technical notes**

Use `stat -c %s` and `$GITHUB_STEP_SUMMARY`.

**Files likely affected**

- `.github/workflows/ci.yml`
- `CONTRIBUTING.md`

**Tests required**

None.

**Documentation required**

CONTRIBUTING.md.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 7. Improve error messages in the SDK with hints

- **Complexity:** Trivial (`wave-trivial`)
- **Expected points:** 2
- **Area:** `area:sdk`

**Problem**

`EscrowContractError` exposes a code and name but no guidance.

**Scope**

Add a `hint` per error (e.g. NotExpired → 'wait until expiration or open a dispute').

**Acceptance criteria**

- [ ] Every code has a hint
- [ ] Test asserts hints exist for all codes

**Technical notes**

Extend `ESCROW_ERRORS` to objects.

**Files likely affected**

- `sdk/src/types.ts`
- `sdk/src/client.ts`
- `sdk/src/client.test.ts`

**Tests required**

Unit test.

**Documentation required**

None.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 8. Example: full lifecycle script against a local network

- **Complexity:** Trivial (`wave-trivial`)
- **Expected points:** 2
- **Area:** `area:docs`

**Problem**

There is only a `create` example; contributors need an end-to-end walkthrough.

**Scope**

Write `examples/lifecycle.sh` using the Stellar CLI against `stellar container start local` (or testnet): deploy, create, fund, approve, release, refund.

**Acceptance criteria**

- [ ] Script runs end-to-end on a local quickstart container
- [ ] Documented in docs/guides

**Technical notes**

Do not embed secret keys; use CLI identities.

**Files likely affected**

- `examples/lifecycle.sh`
- `docs/guides/quickstart.md`

**Tests required**

Manual run documented; optional CI job later.

**Documentation required**

Quickstart guide.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 9. Bound `reason` string length in `open_dispute`

- **Complexity:** Trivial (`wave-trivial`)
- **Expected points:** 2
- **Area:** `area:security`

**Problem**

The dispute reason is unbounded and is stored in an event; oversized inputs waste fees and could bloat indexers.

**Scope**

Reject reasons longer than 256 bytes with a new `InvalidInput` error.

**Acceptance criteria**

- [ ] Contract error added
- [ ] Test for boundary (256 ok, 257 fails)
- [ ] SDK validates client-side

**Technical notes**

`String::len()` returns bytes.

**Files likely affected**

- `contracts/escrow/src/dispute.rs`
- `contracts/escrow/src/errors.rs`
- `contracts/escrow/src/test.rs`
- `sdk/src/client.ts`

**Tests required**

Contract and SDK tests.

**Documentation required**

API docs.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 10. Add `.vscode` recommendations for rust-analyzer and ESLint

- **Complexity:** Trivial (`wave-trivial`)
- **Expected points:** 2
- **Area:** `area:devops`

**Problem**

Contributors lose time configuring editors for a mixed Rust/TS repo.

**Scope**

Add `.vscode/extensions.json` and settings for rust-analyzer targeting the `contracts` workspace.

**Acceptance criteria**

- [ ] Files committed; rust-analyzer picks up contracts/Cargo.toml

**Technical notes**

Set `rust-analyzer.linkedProjects`.

**Files likely affected**

- `.vscode/extensions.json`
- `.vscode/settings.json`
- `.gitignore`

**Tests required**

None.

**Documentation required**

CONTRIBUTING.md.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 11. Typed contract client generation from the wasm spec

- **Complexity:** Trivial (`wave-trivial`)
- **Expected points:** 2
- **Area:** `area:sdk`

**Problem**

The SDK hand-encodes invocations; a spec-derived client would guarantee parity.

**Scope**

Add `scripts/gen-bindings.sh` running `stellar contract bindings typescript` and document how to use the generated client alongside the hand-written SDK.

**Acceptance criteria**

- [ ] Script works with the built wasm
- [ ] Generated output is git-ignored and documented

**Technical notes**

Bindings depend on Stellar CLI; keep it optional.

**Files likely affected**

- `scripts/gen-bindings.sh`
- `docs/guides/bindings.md`
- `.gitignore`

**Tests required**

None.

**Documentation required**

New guide.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

## Medium issues

### 12. Arbitrator timeout: buyer refund path for stale disputes

- **Complexity:** Medium (`wave-medium`)
- **Expected points:** 5
- **Area:** `area:contract`

**Problem**

A disputed escrow whose arbitrator never acts locks funds forever (threat model gap #1).

**Scope**

Add `dispute_opened_at` to `Escrow` and `dispute_timeout` (seconds) set at creation; after timeout the buyer may call `refund` on a `Disputed` escrow.

**Acceptance criteria**

- [ ] Refund allowed only after `dispute_opened_at + dispute_timeout`
- [ ] Seller cannot use the path
- [ ] Threat model updated

**Technical notes**

Changes the `Escrow` struct: bump `version()` to 2 and update SDK decoders.

**Files likely affected**

- `contracts/escrow/src/types.rs`
- `contracts/escrow/src/dispute.rs`
- `contracts/escrow/src/refund.rs`
- `contracts/escrow/src/test.rs`
- `sdk/src/**`
- `docs/security/threat-model.md`

**Tests required**

Contract tests for before/after timeout and wrong caller.

**Documentation required**

Threat model and API docs.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 13. Mutual early termination

- **Complexity:** Medium (`wave-medium`)
- **Expected points:** 5
- **Area:** `area:contract`

**Problem**

Funded escrows can only end via full release, expiry or dispute; parties who agree to stop early have no path.

**Scope**

Add `propose_termination(escrow_id, caller, seller_bps)` and `accept_termination(escrow_id, caller)`; when both parties agree, split remaining funds like `resolve_dispute`.

**Acceptance criteria**

- [ ] Proposal stored per escrow and replaced by newer proposals
- [ ] Acceptance by the other party only
- [ ] Events emitted

**Technical notes**

Reuse the split logic from `dispute.rs`.

**Files likely affected**

- `contracts/escrow/src/termination.rs`
- `contracts/escrow/src/lib.rs`
- `contracts/escrow/src/test.rs`
- `sdk/src/client.ts`

**Tests required**

Contract tests including proposal replacement and self-acceptance rejection.

**Documentation required**

Architecture state diagram and API docs.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 14. Platform fee on release

- **Complexity:** Medium (`wave-medium`)
- **Expected points:** 5
- **Area:** `area:contract`

**Problem**

Marketplaces operating escrows need a fee mechanism.

**Scope**

Optional `fee_recipient` and `fee_bps` set at creation; deducted from each release and dispute payout to the seller.

**Acceptance criteria**

- [ ] Fee ≤ 1000 bps enforced
- [ ] Fee accounted in `released`
- [ ] Zero-fee escrows unchanged

**Technical notes**

Rounding toward the seller; document.

**Files likely affected**

- `contracts/escrow/src/types.rs`
- `contracts/escrow/src/release.rs`
- `contracts/escrow/src/dispute.rs`
- `contracts/escrow/src/test.rs`
- `sdk/src/**`

**Tests required**

Contract tests for fee math and bounds.

**Documentation required**

Threat model (new party) and API docs.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 15. Configurable TTL policy and `extend_ttl` entrypoint

- **Complexity:** Medium (`wave-medium`)
- **Expected points:** 5
- **Area:** `area:contract`

**Problem**

Long escrows (90+ days) exceed the fixed 30-day TTL extension.

**Scope**

Add `extend_ttl(escrow_id, ledgers)` callable by any party that extends all entries of an escrow, and set the initial TTL based on expiration.

**Acceptance criteria**

- [ ] Extension bounded to a max
- [ ] SDK helper added
- [ ] Threat model TTL section updated

**Technical notes**

Use `env.ledger().sequence()` and expiration to compute ledgers.

**Files likely affected**

- `contracts/escrow/src/storage.rs`
- `contracts/escrow/src/lib.rs`
- `sdk/src/client.ts`

**Tests required**

Contract tests with ledger manipulation.

**Documentation required**

Threat model.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 16. Fuzz and property tests for the split and lifecycle

- **Complexity:** Medium (`wave-medium`)
- **Expected points:** 5
- **Area:** `area:contract`

**Problem**

Rounding and state transitions are only example-tested.

**Scope**

Add `proptest`-based tests: for random milestone sets and bps, seller+buyer payouts always equal remaining and never exceed balances; random valid action sequences never leave funds stuck.

**Acceptance criteria**

- [ ] Property tests run in CI under `cargo test`
- [ ] Any found bug fixed with a regression test

**Technical notes**

Keep iteration counts modest for CI time.

**Files likely affected**

- `contracts/escrow/Cargo.toml`
- `contracts/escrow/src/proptest.rs`

**Tests required**

The property tests.

**Documentation required**

Testing section in README.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 17. SDK: prepare-sign-send helper with pluggable signer

- **Complexity:** Medium (`wave-medium`)
- **Expected points:** 5
- **Area:** `area:sdk`

**Problem**

Integrators repeat the simulate/prepare/sign/submit sequence.

**Scope**

Add `EscrowClient.execute(op, { signer })` where `signer` is an interface (`signTransaction(xdr) => Promise<xdr>`), with polling for the result and contract-error mapping.

**Acceptance criteria**

- [ ] Works with Freighter-style and Keypair signers via adapters
- [ ] Errors mapped to `EscrowContractError`
- [ ] Unit tests with mocked rpc.Server

**Technical notes**

Never accept secret keys directly.

**Files likely affected**

- `sdk/src/execute.ts`
- `sdk/src/signers.ts`
- `sdk/src/index.ts`

**Tests required**

Unit tests with mocks.

**Documentation required**

README SDK section.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 18. Live testnet integration test workflow

- **Complexity:** Medium (`wave-medium`)
- **Expected points:** 5
- **Area:** `area:devops`

**Problem**

Contract behaviour on a real network (fees, TTL, SAC transfers) is untested.

**Scope**

Add `tests/live/escrow.live.test.ts` that deploys the wasm to testnet using a CI-provisioned identity, runs the lifecycle and asserts balances; schedule weekly.

**Acceptance criteria**

- [ ] Skipped unless `LIVE_TESTS=1`
- [ ] Uses friendbot; no long-lived secrets
- [ ] Handles testnet resets

**Technical notes**

Generate a fresh keypair per run in CI.

**Files likely affected**

- `tests/live/escrow.live.test.ts`
- `.github/workflows/live.yml`

**Tests required**

The live test.

**Documentation required**

docs/guides/testing.md.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 19. Multi-asset milestone support

- **Complexity:** Medium (`wave-medium`)
- **Expected points:** 5
- **Area:** `area:contract`

**Problem**

All milestones share one asset; some contracts pay in stablecoin for some milestones and native for others.

**Scope**

Allow an optional per-milestone asset override; fund pulls each asset's total.

**Acceptance criteria**

- [ ] Backward compatible when overrides are absent
- [ ] Refund/dispute logic handles per-asset remaining

**Technical notes**

Significant accounting change — store remaining per asset.

**Files likely affected**

- `contracts/escrow/src/types.rs`
- `contracts/escrow/src/escrow.rs`
- `contracts/escrow/src/refund.rs`
- `contracts/escrow/src/dispute.rs`
- `contracts/escrow/src/test.rs`

**Tests required**

Contract tests across two assets.

**Documentation required**

Threat model and API docs.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 20. Event indexing example with stellar-contract-observer

- **Complexity:** Medium (`wave-medium`)
- **Expected points:** 5
- **Area:** `area:docs`

**Problem**

There is no worked example of tracking escrows off-chain.

**Scope**

Write a guide that configures stellar-contract-observer (or a small RPC `getEvents` script) to follow escrow events and render a status table.

**Acceptance criteria**

- [ ] Script in `examples/` runs against testnet
- [ ] Guide explains topic filters

**Technical notes**

Use `getEvents` with the contract id and `created`/`released` topics.

**Files likely affected**

- `examples/watch-events.ts`
- `docs/guides/events.md`

**Tests required**

None beyond the script running.

**Documentation required**

The guide.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 21. Gas and storage benchmarks

- **Complexity:** Medium (`wave-medium`)
- **Expected points:** 5
- **Area:** `area:devops`

**Problem**

No visibility into CPU/memory/fee costs per function.

**Scope**

Add a Rust test that prints `env.cost_estimate()` per function and a script that records results in `docs/benchmarks.md`; CI fails on >20% regression.

**Acceptance criteria**

- [ ] Benchmarks documented
- [ ] Regression check in CI

**Technical notes**

Use `env.cost_estimate().budget()` in tests.

**Files likely affected**

- `contracts/escrow/src/bench.rs`
- `scripts/bench.sh`
- `docs/benchmarks.md`
- `.github/workflows/ci.yml`

**Tests required**

The benchmark test.

**Documentation required**

Benchmarks page.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

## High issues

### 22. Pluggable dispute resolution (multi-arbitrator / DAO vote)

- **Complexity:** High (`wave-high`)
- **Expected points:** 10
- **Area:** `area:contract`

**Problem**

A single arbitrator is a trust bottleneck for larger contracts.

**Scope**

Allow `arbitrator` to be a contract implementing a `Resolver` interface; support an M-of-N arbitrator panel contract in `contracts/panel` that calls `resolve_dispute`.

**Acceptance criteria**

- [ ] Panel contract with threshold votes
- [ ] Escrow verifies the caller is its configured resolver via `require_auth` on the contract address
- [ ] Threat model extended

**Technical notes**

Cross-contract auth: the panel contract authorizes as itself.

**Files likely affected**

- `contracts/panel/**`
- `contracts/Cargo.toml`
- `contracts/escrow/src/dispute.rs`
- `docs/security/threat-model.md`

**Tests required**

Tests for both contracts and the integration.

**Documentation required**

Architecture and threat model.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 23. Reference web UI for parties

- **Complexity:** High (`wave-high`)
- **Expected points:** 10
- **Area:** `area:frontend`

**Problem**

Non-technical buyers and sellers cannot interact with the contract.

**Scope**

Build `frontend/` (Next.js, Tailwind) with wallet connection (Freighter), escrow creation wizard, milestone board, dispute flow and event history. Minimal, accessible design; real empty/loading states.

**Acceptance criteria**

- [ ] All contract functions reachable from the UI
- [ ] Reads via SDK; signs via wallet
- [ ] CI lint/typecheck/build

**Technical notes**

Depends on the SDK execute helper (medium issue) or implements its own.

**Files likely affected**

- `frontend/**`
- `pnpm-workspace.yaml`
- `.github/workflows/ci.yml`

**Tests required**

Component tests and a Playwright smoke test against a local network.

**Documentation required**

docs/guides/ui.md.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 24. Formal specification and model checking of the state machine

- **Complexity:** High (`wave-high`)
- **Expected points:** 10
- **Area:** `area:security`

**Problem**

Correctness of state transitions is argued informally.

**Scope**

Write a TLA+ (or Quint) model of the escrow state machine and check invariants: funds conservation, no release without approval, no refund before expiry, disputes always resolvable. Map each invariant to a contract test.

**Acceptance criteria**

- [ ] Model committed under `docs/formal/`
- [ ] Model checker runs in CI (bounded)
- [ ] Invariants cross-referenced in threat model

**Technical notes**

Keep the model small: amounts abstracted to counts.

**Files likely affected**

- `docs/formal/escrow.tla`
- `.github/workflows/formal.yml`
- `docs/security/threat-model.md`

**Tests required**

Model checking job.

**Documentation required**

Threat model.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 25. Recurring / subscription escrows

- **Complexity:** High (`wave-high`)
- **Expected points:** 10
- **Area:** `area:contract`

**Problem**

Retainers and SaaS-style engagements need periodic releases without new escrows.

**Scope**

Add a `Schedule` variant: fixed amount per period releasable by the seller after each period elapses unless the buyer has paused; buyer can cancel with notice.

**Acceptance criteria**

- [ ] Period accounting tested across boundaries
- [ ] Pause/cancel semantics documented
- [ ] Events for each period release

**Technical notes**

Separate module `schedule.rs`; do not overload milestones.

**Files likely affected**

- `contracts/escrow/src/schedule.rs`
- `contracts/escrow/src/types.rs`
- `contracts/escrow/src/test.rs`
- `sdk/src/**`
- `docs/security/threat-model.md`

**Tests required**

Contract tests with ledger time manipulation.

**Documentation required**

Architecture, API and threat model.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged

### 26. Independent security audit preparation package

- **Complexity:** High (`wave-high`)
- **Expected points:** 10
- **Area:** `area:security`

**Problem**

The contract cannot be audited without a prepared scope, invariants and test evidence.

**Scope**

Produce `docs/security/audit-scope.md` (scope, out-of-scope, invariants, known gaps), a reproducible build script with wasm hash, coverage report, and a Slither-style static analysis run (e.g. `cargo geiger`, `cargo audit`), plus an issue template for audit findings.

**Acceptance criteria**

- [ ] Reproducible build documented with hash
- [ ] Coverage ≥ 90% of contract lines
- [ ] All known gaps listed with severity

**Technical notes**

Coordinate with the threat model and the formal spec issue.

**Files likely affected**

- `docs/security/audit-scope.md`
- `scripts/reproducible-build.sh`
- `.github/ISSUE_TEMPLATE/audit_finding.md`

**Tests required**

Coverage job in CI.

**Documentation required**

Audit scope page.

**Definition of done**

- [ ] All acceptance criteria met
- [ ] Tests added and passing in CI (`make test`, `make lint`, `make build`)
- [ ] Documentation updated as listed above
- [ ] No secrets or credentials introduced
- [ ] PR reviewed and squash-merged
