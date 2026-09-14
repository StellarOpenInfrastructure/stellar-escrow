# ADR 0002: One escrow contract with internal modules, no upgradeability

- Status: Accepted
- Date: 2026-09-14

## Context

The scope asks for contracts covering escrow, milestones, release, refund and
dispute. These could be five contracts calling each other, or one contract with
five modules.

## Decision

Ship **one** contract (`contracts/escrow`) whose Rust modules map one-to-one to
the functional areas. Do not include an admin role or an upgrade entrypoint in
the initial scaffold.

## Consequences

- Single storage namespace and single authorization surface — easier to audit.
- No cross-contract calls between escrow components → smaller attack surface,
  lower fees.
- Bug fixes require a new deployment; in-flight escrows finish on old code.
- Future modularisation (e.g. pluggable dispute resolvers) is a High Drips
  Wave issue and must come with its own threat model update.
