# Implementation status

This file reports what the repository proves today. Planned architecture is
tracked separately in [architecture.md](architecture.md).

## Present in source

- Anchor `entry` has config/event initialization, organizer pause/resume and
  gate-authority rotation.
- Config initialization verifies the supplied ProgramData belongs to this
  program and the initializer is its upgrade authority.
- Events use `PDA(["event", organizer, event_id])`. Event creation validates
  price, supply, limits, fee/royalty bounds, supported USDC mint, Bubblegum
  tree ownership/configuration/capacity and depth-14/20 buffer-64 CMT header,
  and the MPL Core collection's
  BubblegumV2/permanent transfer/permanent freeze plugins and authorities. It
  initializes a separate SPL Account Compression registry tree with the Event
  PDA as its authority; supported tree depth is 14 or 20 with a 64-entry
  changelog buffer.
- The Event stores a domain-separated ownership registry root and sequence.
  Primary purchase replaces the empty nonce leaf with the buyer owner record;
  resale requires the current seller proof, replaces that leaf with the buyer,
  and emits the new registry root/sequence. Both updates are in the same
  transaction as the ticket/payment operations.
- A per-event zero-copy check-in bitmap is allocated for supply up to 80,000.
  Batch check-in requires the ticket owner's registry proof, rejects unsold
  nonces and invalid owners, sets bits idempotently, emits events, and has Rust
  unit and ProgramTest coverage. The proof-based batch is bounded by the
  64-account transaction limit (up to four tickets at depth 14; three at
  depth 20).
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
  bounds/idempotence, initializer authorization, transfer-proof flag/hash/nonce
  checks, pause/resume and batch check-in. The Anchor ProgramTest harness has
  local tests for pause/resume and check-in. TypeScript has a shared resale-cap
  test.
- Core collection plugin parsing is unit-tested. An ignored ProgramTest loads
  ENTRY and the real SPL Account Compression SBF program and verifies that
  `create_event` initializes a depth-14, buffer-64 registry tree with the Event
  PDA authority at a 400,000 CU transaction limit. Another ignored ProgramTest
  runs purchase and resale with the real SPL Token and compression programs
  and a mock Bubblegum processor that accepts only the `mint_v2`, `freeze_v2`,
  `thaw_v2`, and `transfer_v2` discriminators. It proves invalid registry
  proof failure rolls back USDC; successful purchase and resale split USDC
  exactly, close the listing, and update the real registry tree. It rejects a
  150%-of-face ask at listing time and again at settlement when given an
  artificially over-cap listing, without moving USDC or closing the listing.
  This does not validate Bubblegum leaf transitions. Client builders
  must set a compute budget for event creation. Run
  `scripts/test-registry-cpi.sh` to build both SBF artifacts in a temporary
  directory and execute these tests.
- The purchase/resale account layouts include SBF stack-frame mitigations,
  and the Anchor IDL has been regenerated from those layouts. Canonical USDC
  ATA checks run in the handlers, so client builders must derive/pass those
  accounts explicitly; the IDL no longer derives them from associated-token
  constraints.

## Present only as a design or stub

- The real SPL compression program's initialize and leaf-replacement
  instructions are tested, including stale/replayed leaf-update rejection and
  unchanged tree state after failure. `create_event` invokes real initialization
  from ENTRY, while direct leaf replacement is also tested directly against
  SPL. ENTRY's `replace_owner_leaf` wrapper is exercised during purchase and
  resale against the real compression processor. Registry tree
  allocation/proof building and client-side ownership proof verification
  remain unimplemented. Tiers,
  purchase receipts, identity nullifiers, checkpoints, gift/recovery,
  escrow/refunds, and offline conflict reconciliation are also not implemented.
- Collection plugin presence and authority are validated, but that does not
  prove the full raw-transfer policy on-chain. Bubblegum V2 CPI state
  transitions have not been exercised by validator integration tests.
- The purchase/resale test proves rollback for an invalid purchase registry
  proof and exact USDC settlement using the real SPL Token processor. Bubblegum
  transition behavior and rollback after an actual Bubblegum failure remain
  untested. Seller ownership rejection, stale Bubblegum proofs, direct-transfer
  policy, and check-in/list/resale with real Bubblegum remain untested.
  Check-in proof validation is tested, but gate-service proof fetching and
  owner resolution are not.
- `app/blinks-api` exposes Actions metadata and validates/proxies buy and resale
  requests to a configured transaction-builder service. That trusted service
  is not implemented here; gatekeeper verification/partial signing, v0/ALT
  construction, compute-budget selection, and DAS proof fetching are absent.
- `app/wallet-web` connects a wallet and can review/simulate/submit a prepared
  transaction, but depends on the missing builder and ticket lookup service.
- `app/gate-pwa` can scan fresh QR codes, queue pending scans locally, and
  prepare a wallet-confirmed check-in through a configured service. The
  `ENTRY_CHECKIN_SERVICE_URL` adapter only validates and proxies; proof
  resolution and the transaction service are absent. Offline scans do not
  grant admission.
- `app/organizer-web` remains a shell: wallet connection, tree/collection
  provisioning, event creation, and event management are not wired up.
- `scripts/create-tree.ts` and `scripts/seed-demo-event.ts` throw stub errors.
  There is no verified tree/Core collection provisioning workflow, indexer,
  proof service, identity service, or production RPC configuration.
- The default ProgramTest suite does not exercise Bubblegum or payment CPI
  integrations. The separate ignored CPI tests cover real registry setup,
  direct leaf replacement, and purchase/resale payment and registry updates
  with a mock Bubblegum processor. Bubblegum's actual state transitions remain
  outside runtime coverage. The current tests are useful but are not a protocol
  security suite or audit.

## Recommended implementation order

1. **Finish validating the on-chain asset policy.** Keep the existing program
   ID. Extend local validator coverage to prove registry leaf updates and
   validate mint, resale, direct transfer, stale proof and atomic rollback.
   Files: `programs/entry/src/state/`,
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

Not ready for deployment or client use. Collection validation, ENTRY's
registry replacement wrapper, and SPL Token purchase/resale settlement have
local coverage. Bubblegum CPI behavior and rollback after Bubblegum failure
still need tests against the real Bubblegum program. Purchase, resale, and gate
client flows are also incomplete.
Deployment, live-chain provisioning, and secret configuration remain separate
operator actions.
