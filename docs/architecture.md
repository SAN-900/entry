# ENTRY architecture

## On-chain authority

Each event is keyed by `Event = PDA(["event", organizer, event_id])`. That PDA is configured as the private Bubblegum V2 tree delegate, the MPL Core collection update authority, and the collection's `PermanentTransferDelegate` and `PermanentFreezeDelegate`. The organizer key controls event configuration and can rotate the gate authority; it does not sign ticket transfers. Event setup must create the tree and Core collection with these authorities and the Bubblegum V2 plugin before calling `create_event`.

The on-chain code checks that the tree is empty, private, delegated to the Event PDA, and has capacity for the declared supply. It checks that the collection account is owned by MPL Core. It does not deserialize and verify the collection plugin configuration or the compression tree's buffer size; those setup requirements still need a creation/validation tool and runtime tests. Configure the tree buffer to 64 before event creation; the current program does not enforce that value.

## Accounts and identifiers

- `Config ["config"]`: admin, fee vault, platform fee in basis points (capped at 5%).
- `Event ["event", organizer, event_id]`: immutable pricing, supply, collection/tree and current gate keys, plus active/sold state.
- `CheckIns ["checkins", event]`: one zero-copy bitmap. Bit `n` records check-in for Bubblegum leaf nonce `n`; bits are only set. Current single-account supply limit is 80,000.
- `PurchaseCounter ["purchases", event, wallet]`: program-enforced primary and resale purchase count.
- `Listing ["listing", event, asset_id, seller]`: seller-specific listing PDA, so one seller cannot squat another seller's listing.

Asset IDs are derived from the event's configured tree and leaf nonce. There is no user-supplied asset ID in list or purchase instructions. A listing records the signer as seller without checking ownership. At resale settlement, Bubblegum CPI receives that seller as leaf owner and must validate the proof; a dud listing cannot take payment because a failed CPI rolls back the transaction.

## Purchase and transfer control

`purchase` requires both buyer and the configured gatekeeper to sign. It checks the event state, nonce race, per-wallet limit, canonical six-decimal USDC mint, and associated token accounts. It splits the payment with `transfer_checked`, mints via Bubblegum V2 `mintV2` using the Event PDA, then calls `freezeV2` in the same instruction. Metadata hashes and the leaf asset ID are derived from the event data and nonce. The freeze proof is supplied by the client and must match the post-mint root. Whether a client can reliably precompute that root/proof and whether the CPI sequence fits transaction limits still require integration tests.

Resale follows the conservative thaw-transfer-freeze path in one instruction. It rechecks the face-price cap and the check-in bit at purchase time, pays seller/organizer royalty/platform fee with checked transfers, and moves the Bubblegum leaf using the Event PDA's collection authority. Three sequential Merkle proofs are required for thaw, transfer, and refreeze. Any CPI failure reverts payment and state changes. Since this path can exceed a legacy transaction's account/size limits, a production Blink builder still needs a v0 transaction, an Address Lookup Table, and an explicit compute budget (initial target 400,000 units).

The Actions API currently returns demo metadata and a 501 response for purchase and resale transaction construction. The program's gatekeeper signature requirement exists, but no backend key loading, human verification, or partial-signing flow is wired up. Never expose a gatekeeper secret to the browser or logs.

## Check-in

The configured `gate_authority` signs `check_in_batch`. Each nonce must be less than `sold`; setting an existing bit is an idempotent no-op with an event. Listing and resale both reject checked-in nonces. This gives a global on-chain single-use bit, but an offline scanner cannot know that another disconnected gate has already checked the same ticket until sync.

## Validation status

Compilation and pure arithmetic/bitmap unit tests do not prove Bubblegum CPI behavior, Core plugin configuration, correct post-mint proof construction, direct-transfer rejection, or full USDC rollback behavior. Those need validator integration tests before describing the transfer restriction as deployed or production validated. See [implementation status](implementation-status.md) and [threat model](threat-model.md).
