# Contract interface

| Function            | Args                                                                                                    | Returns                | Auth                  |
| ------------------- | ------------------------------------------------------------------------------------------------------- | ---------------------- | --------------------- |
| `create`            | buyer, seller, arbitrator, asset, milestones: Vec<{amount: i128, description: String}>, expiration: u64 | escrow id (u64)        | buyer                 |
| `fund`              | escrow_id                                                                                               | –                      | buyer                 |
| `cancel`            | escrow_id, caller                                                                                       | –                      | caller (buyer/seller) |
| `approve_milestone` | escrow_id, index: u32                                                                                   | –                      | buyer                 |
| `release`           | escrow_id, index, caller                                                                                | –                      | caller (buyer/seller) |
| `refund`            | escrow_id                                                                                               | refunded amount (i128) | buyer                 |
| `open_dispute`      | escrow_id, caller, reason: String                                                                       | –                      | caller (buyer/seller) |
| `resolve_dispute`   | escrow_id, seller_bps: u32                                                                              | –                      | arbitrator            |
| `get_escrow`        | escrow_id                                                                                               | `Escrow`               | –                     |
| `get_milestone`     | escrow_id, index                                                                                        | `Milestone`            | –                     |
| `count`             | –                                                                                                       | u64                    | –                     |
| `version`           | –                                                                                                       | u32                    | –                     |

Errors are returned as `Error(Contract, #n)`; see `sdk/src/types.ts`
(`ESCROW_ERRORS`) for the mapping.
