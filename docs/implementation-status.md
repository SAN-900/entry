# Implementation status

This file reports what the repository proves today. Planned architecture is
tracked separately in [architecture.md](architecture.md).

## Present in source

- Anchor `entry` has config/event initialization, organizer pause/resume and
  gate-authority rotation.
- Config initialization verifies the supplied ProgramData belongs to this
  program and the initializer is its upgrade authority.
- Events use `PDA(["event", organizer, event_id])`. Event creation validates
  price, supply, limits, fee/royalty bounds, supported USDC mint, and basic
  Bubblegum tree ownership/configuration/capacity.
- A per-event zero-copy check-in bitmap is allocated for supply up to 80,000.
  Batch check-in is bounded, rejects unsold nonces, sets bits idempotently,
  emits events, and has Rust unit and ProgramTest coverage.
- Seller-specific listing PDAs include event, derived asset ID, and seller.
  Listing checks the face-price cap and check-in status; cancellation is
  seller-authorized.
- Purchase requires buyer and configured gatekeeper signers, validates the
  supported six-decimal USDC mint and canonical token accounts, checks the
  wallet counter and expected sequential tree nonce, then calls Bubblegum V2
  `mintV2` followed by `freezeV2` in one instruction.
- Resale rechecks ask price from face value, check-in status, and buyer wallet
  count; it performs checked USDC splits then attempts Bubblegum V2
  thaw/transfer/freeze and closes the listing.
- Rust unit tests cover cap arithmetic, exact settlement splits, bitmap
  bounds/idempotence, initializer authorization, pause/resume and batch check
  in. The Anchor ProgramTest harness has local tests for pause/resume and
  check-in. TypeScript has a shared resale-cap test.
- The purchase/resale account layouts include SBF stack-frame mitigations,
  and the Anchor IDL has been regenerated from those layouts. Canonical USDC
  ATA checks run in the handlers, so client builders must derive/pass those
  accounts explicitly; the IDL no longer derives them from associated-token
  constraints.

## Present only as a design or stub

- The dual-tree admission registry, registry proofs/root updates, tiers,
  purchase receipts, identity nullifiers, checkpoints, gift/recovery,
  escrow/refunds, and offline conflict reconciliation are not implemented.
- Freeze/delegate behavior is not established as a universal raw-transfer
  block. `create_event` checks tree ownership, private mode, Event PDA
  delegation, empty state and capacity, but does not decode the MPL Core
  collection plugins or validate a compression tree buffer size.
- Bubblegum V2 mint/freeze and thaw/transfer/freeze have not been exercised
  by validator integration tests. No test proves USDC settlement rollback,
  seller ownership rejection, stale-proof behavior, direct-transfer policy,
  or the complete check-in/list/resale interaction.
- `app/blinks-api` buy/resale POST routes return HTTP 501. No real Actions
  transaction builder, gatekeeper verification/partial signer, v0/ALT
  builder, compute-budget selection, or DAS proof client is wired up.
- `app/gate-pwa` scanner, `app/wallet-web`, and `app/organizer-web` are shells;
  no production wallet connection, ticket display, organizer provisioning,
  checkout, or scanner redemption flow is implemented.
- `scripts/create-tree.ts` and `scripts/seed-demo-event.ts` throw stub errors.
  There is no verified tree/Core collection provisioning workflow, indexer,
  proof service, identity service, or production RPC configuration.
- The supplied ProgramTest suite does not exercise CPI integrations. The
  current tests are useful but are not a protocol security suite or audit.

## Recommended implementation order

1. **Reconcile and validate the on-chain asset policy.** Keep the existing
   program ID. Design the registry account and Bubblegum V2 append/update
   interface; implement collection plugin and tree configuration validation;
   test mint, resale, direct transfer, stale proof and atomic rollback on a
   local validator. Files: `programs/entry/src/state/`,
   `instructions/create_event.rs`, `purchase.rs`, `buy_resale.rs`,
   `programs/entry/tests/`, `scripts/create-tree.ts`, and `idls/`.
2. **Regenerate the client interface and build secure transaction services.**
   Match generated IDL to the final accounts, then implement GET/POST Actions
   handlers, event/proof lookup, user verification, backend-only gatekeeper
   co-sign, bounded transaction construction and safe rate limiting. Files:
   `idls/`, `app/blinks-api/`, shared client package, manifests/lockfile.
   Tests: IDL account serialization, Actions response/protocol, malformed
   inputs, stale proofs, missing gatekeeper and no secret leakage.
3. **Complete online redemption before offline mode.** Implement ticket
   ownership lookup and single-use online check-in in the gate PWA; only then
   add signed checkpoints/local journals and conflict reconciliation. Files:
   `app/gate-pwa/`, new scanner modules, checkpoint state/instructions.
   Tests: duplicate/replayed scans, invalid owner, stale checkpoint, scanner
   key rotation, event isolation and concurrent disconnected scanners.
4. **Add organizer setup and operations.** Implement event/tree/collection
   provisioning, environment validation, indexer/reconciliation and
   monitoring. Files: `app/organizer-web/`, `scripts/`, new service packages.
   Tests: repeatable idempotent setup, wrong plugin/tree authority rejection,
   indexer replay and recovery from RPC interruption.
5. **Only then add optional scale/product modes.** Tiers, ZK nullifiers,
   walletless checkout, identity-bound entry, gifts, escrow refunds and
   offline checkpoints each need a separate threat model, implementation,
   privacy/security review and tests.

## Client/deployment readiness

Not ready for production deployment or client use. A successful compile does
not validate Bubblegum CPIs, the collection plugins, transaction-size/compute
limits, real proofs, or the client flow. The next implementation milestone is
Phase 1 in [architecture.md](architecture.md): prove the asset/admission
policy on a local validator and bring the IDL into sync before wiring any
client transaction builder. Deployment, live-chain provisioning, and secret
configuration remain separate operator actions.
