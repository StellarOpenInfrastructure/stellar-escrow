# Architecture

## State machine

```mermaid
stateDiagram-v2
    [*] --> Created: create (buyer)
    Created --> Funded: fund (buyer)
    Created --> Cancelled: cancel (buyer|seller)
    Funded --> Funded: approve_milestone (buyer) / release (buyer|seller)
    Funded --> Completed: release of last milestone
    Funded --> Completed: refund after expiration (buyer)
    Funded --> Disputed: open_dispute (buyer|seller)
    Disputed --> Completed: resolve_dispute (arbitrator)
    Completed --> [*]
    Cancelled --> [*]
```

Milestones: `Pending → Approved → Released`, or `Pending|Approved → Refunded`
(refund or dispute resolution).

## Components

```mermaid
flowchart LR
    W[Wallet / backend] -->|builds ops| SDK["@stellar-escrow/sdk"]
    SDK -->|invokeHostFunction| RPC[(Stellar RPC)]
    RPC --> C[escrow contract]
    C -->|transfer| T[SEP-41 token]
    C -->|events| IDX[Indexers / stellar-contract-observer]
```

- **Contract** (`contracts/escrow`): all business rules, no admin.
- **SDK** (`sdk`): typed invocation builders and RPC readers. Never signs.
- **Stellar infrastructure**: Stellar RPC for simulation, submission and
  reads; contract events for off-chain tracking. No Horizon dependency.

## Storage layout

| Key                    | Type        | Storage    |
| ---------------------- | ----------- | ---------- |
| `Count`                | `u64`       | instance   |
| `Escrow(id)`           | `Escrow`    | persistent |
| `Milestone(id, index)` | `Milestone` | persistent |

## Events

`created`, `funded`, `cancelled`, `approved`, `released`, `refunded`,
`disputed`, `resolved` — each with `escrow_id` as topic and a map of named
fields as data (see `contracts/escrow/src/events.rs`).
