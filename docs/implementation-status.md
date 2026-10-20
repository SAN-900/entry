# Implementation status

## Implemented in source

- Config and event initialization, including fee/supply/price/royalty limits and gatekeeper/gate-authority fields.
- Event PDA authority model and event activation/gate-authority update instructions.
- Single per-event zero-copy check-in bitmap; batch check-in is bounded, idempotent for duplicate bits, emits per-nonce events, and rejects unsold nonces.
- Listing PDA includes event, derived Bubblegum asset ID, and seller. Listing checks the check-in bit and face-price cap. Cancellation requires the listing seller.
- Purchase requires buyer and gatekeeper signatures, checks canonical USDC mint/ATAs and wallet limit, transfers price/fee, derives the asset ID from tree + nonce, and contains Bubblegum V2 `mintV2` then `freezeV2` CPIs in one instruction.
- Resale checks the listing's derived asset/nonce, cap, check-in state, and wallet limit; splits checked USDC transfers; then contains thaw/transfer/freeze Bubblegum V2 CPIs and closes the listing.
- Arithmetic and bitmap unit tests cover cap calculation, exact remainder splits, bit isolation/idempotence, and bitmap bounds.

## Not implemented or not proven

- The Actions API purchase/resale endpoints remain 501 transaction-builder stubs. The Blink does not load a gatekeeper key, verify a human, partially sign a purchase, build v0 transactions, use an Address Lookup Table, or set a compute budget.
- Event creation checks that the tree is empty/private, delegated to the Event PDA, and has sufficient declared mint capacity. It does not validate the MPL Core collection's `BubblegumV2`, permanent transfer delegate, and permanent freeze delegate plugin configuration. It also does not validate the compression tree buffer size. Provision an appropriately configured tree (buffer 64) and collection before event creation.
- No validator integration test has exercised Bubblegum V2 mint/freeze or thaw/transfer/freeze. The supplied freeze/proof sequence and direct owner-transfer rejection are not runtime proven.
- No integration test proves payment rollback, USDC settlement, wallet limits, seller ownership validation by Bubblegum, or check-in/listing/resale interactions on a deployed program.
- No DAS proof client, Core collection/tree setup transaction builder, gatekeeper verification service, or production RPC configuration is present.
- The shared TypeScript resale-cap test and focused Rust unit tests are not a protocol security suite.

Do not describe marketplace transfer prevention, anti-bot verification, end-to-end purchase/resale, or production check-in as validated until these runtime/client pieces pass integration tests.
