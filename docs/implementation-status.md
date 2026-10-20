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
  PDA authority at a 400,000 CU transaction limit. Client builders must set a
  compute budget for event creation. Run `scripts/test-registry-cpi.sh` to build
  both SBF artifacts in a temporary directory and execute this test.
- The purchase/resale account layouts include SBF stack-frame mitigations,
  and the Anchor IDL has been regenerated from those layouts. Canonical USDC
  ATA checks run in the handlers, so client builders must derive/pass those
  accounts explicitly; the IDL no longer derives them from associated-token
  constraints.

## Present only as a design or stub

- The real SPL compression program's initialize and leaf-replacement
  instructions are tested, including stale/replayed leaf-update rejection and
  unchanged tree state after failure. `create_event` invokes real initialization
  from ENTRY, while leaf replacement is currently tested directly against SPL.
  ENTRY's `replace_owner_leaf` CPI wrapper and its same-transaction use with
  purchase/resale still need integration coverage. Registry tree
  allocation/proof building and client-side ownership proof verification
  remain unimplemented. Tiers,
  purchase receipts, identity nullifiers, checkpoints, gift/recovery,
  escrow/refunds, and offline conflict reconciliation are also not implemented.
- Collection plugin presence and authority are validated, but that does not
  prove the full raw-transfer policy on-chain. Bubblegum V2 mint/freeze and
  thaw/transfer/freeze have not been exercised by validator integration tests.
- No test proves USDC settlement rollback, seller ownership rejection,
  stale-proof behavior, direct-transfer policy, or the complete
  check-in/list/resale interaction. Check-in proof validation is tested, but
  complete gate-service proof fetching and owner resolution are not.
- `app/blinks-api` buy/resale POST routes return HTTP 501. No real Actions
  transaction builder, gatekeeper verification/partial signer, v0/ALT
  builder, compute-budget selection, or DAS proof client is wired up.
- `app/gate-pwa` scanner, `app/wallet-web`, and `app/organizer-web` are shells;
  no production wallet connection, ticket display, organizer provisioning,
  checkout, or scanner redemption flow is implemented.
- `scripts/create-tree.ts` and `scripts/seed-demo-event.ts` throw stub errors.
  There is no verified tree/Core collection provisioning workflow, indexer,
  proof service, identity service, or production RPC configuration.
- The default ProgramTest suite does not exercise Bubblegum or payment CPI
  integrations. The separate ignored CPI tests cover registry setup and a
  direct SPL leaf replacement, not the complete purchase/resale path. The
  current tests are useful but are not a protocol security suite or audit.

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

Not ready for deployment or client use. Collection validation and SPL registry
tree initialization and replacement have local coverage, but ENTRY's registry
replacement wrapper and all Bubblegum/payment flows still need tests against
their real processors. After those pass, bring the final IDL into sync before
wiring client transaction builders.
Deployment, live-chain provisioning, and secret configuration remain separate
operator actions.
