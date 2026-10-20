# ENTRY threat model

| Threat | On-chain defense in source | Remaining gap |
|---|---|---|
| Direct ticket transfer bypasses resale cap | Event PDA is intended to hold Bubblegum tree authority and collection permanent transfer/freeze delegates; purchase freezes and resale thaws/transfers/refreezes atomically | Collection plugins are not inspected by `create_event`; Bubblegum V2 integration and direct-transfer rejection have not been runtime-tested |
| Listing squatting | Listing PDA includes seller as a seed | A seller can create a dud listing for an asset they do not own; resale CPI should reject it |
| Price cap compounds | Every list and buy checks `face_price * cap_bps` | Off-program side payments cannot be prevented |
| Sale after check-in | Bitmap bit is checked both when listing and at resale settlement | Only online on-chain check-in is globally current across gates |
| Duplicate or replayed check-in | Gate authority signer, sold nonce bound, set-only bit, idempotent duplicate handling | Offline gates can scan the same ticket before syncing |
| Bot wallet farming | Buyer requires gatekeeper co-sign; per-wallet limit also applies | Backend gatekeeper human verification/partial signing is not implemented; wallet limits alone are sybilable |
| Stale or forged Bubblegum proof | Bubblegum CPIs validate supplied roots/proofs; asset ID derives from event tree and nonce | Sequential proof construction and tree buffer configuration need runtime validation |
| Payment theft or rounding residue | Canonical six-decimal USDC, associated token accounts, `transfer_checked`, checked arithmetic, seller gets exact remainder | Token CPI and rollback behavior need validator integration tests |
| Compromised organizer or gate key | Organizer controls event settings; gate authority is separately rotatable | Key management and rotation procedures are operational requirements |

Do not store private keys in frontend code, source control, logs, or transaction metadata. Keep gatekeeper signing server-side and avoid logging serialized secret material.
