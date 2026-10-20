# ENTRY technical architecture

This document combines the supplied full-system design with the accounts and
instructions that exist in this repository. It distinguishes the current
prototype from the intended production design. A design in a future phase is
not evidence that the feature already works.

## Design decisions

### Admission is based on program state

Bubblegum V2 compressed NFTs are Merkle-tree leaves, not SPL Token or
Token-2022 accounts. ENTRY therefore cannot use a Token-2022 transfer hook for
these tickets. The program currently attempts to freeze each ticket and routes
resale through its own instruction. The production design adds a program-owned
compressed ownership registry and makes that registry the source of truth at
the gate. This provides a defensible admission rule even if a raw Bubblegum
transfer is possible: an ownership change is valid for entry only when ENTRY
also commits the matching registry update. Freezing remains a useful extra
restriction, but is not the sole admission security boundary until the exact
Bubblegum V2 and MPL Core plugin behavior is proven on a validator.

The registry tree and its proof/update flow now exist in source. A reproducible
ProgramTest loads the real SPL Account Compression SBF to verify tree
initialization from `create_event`, direct leaf replacement, and rejection of
a stale/replayed leaf update. ENTRY's own leaf-replacement CPI wrapper and its
atomic use with purchase/resale are not yet runtime-tested. Check-in currently
uses the single-use bitmap, gate-authority signature, and an owner-of-record
proof for each ticket. The client also does not yet build ownership proofs.
Describe the present system as a prototype with unverified Bubblegum freeze
enforcement and unverified end-to-end registry behavior, not as preventing
every off-program transfer.

### Two ticket modes

- **Standard:** ownership can change through protocol settlement and, in the
  target design, a gift flow with a cooldown. The current code has resale but
  no gift or cooldown instruction.
- **ID-bound:** a future admission policy checks a privacy-preserving identity
  binding. No identity provider, ZK verifier, or identity-bound state is
  implemented. Do not store raw identity data on-chain.

The initial release should use the Standard flow and online gate validation.
Offline checkpoints and walletless checkout are later product phases.

### Costs and product claims

Tree rent, transaction fees, priority fees, RPC/indexing, and infrastructure
vary. Do not publish the pasted dollar estimates as guaranteed costs. Say
“low Solana transaction costs” until measured on the target network and tree
configuration. Solana Actions/Blinks can serve compatible surfaces such as X;
other social platforms should link to the PWA rather than be described as
native Blink clients.

## System shape

```text
Wallet / Actions client ──> ENTRY Actions API ──> Solana program
PWA checkout ────────────> transaction service ─> Solana program
Gate scanner ────────────> online redemption ───> Solana program
                                      │
                                      ├── DAS / proof RPC
                                      ├── event indexer + database
                                      └── notifications and payout jobs (later)
```

The repository currently contains four Next.js shells (Blinks API, gate PWA,
organizer web, wallet web) and shared pricing code. It does not contain a
production indexer, proof service, identity service, payout worker, mobile
scanner, embedded-wallet provider, or payment on-ramp.

## On-chain program

The repository has one Anchor program (`entry`); Bubblegum V2, SPL Account
Compression, SPL Token, and MPL Core are external programs called by CPI.
Keep ticketing rules in ENTRY rather than forking those protocols.

### Current account model

| Account | Seeds | Current role |
| --- | --- | --- |
| `Config` | `["config"]` | Admin, fee vault, fee basis points. |
| `Event` | `["event", organizer, event_id]` | Event authority, gates, ticket and registry trees, registry root/sequence, collection, face price, supply, wallet limit, resale cap, royalty, active/sold state. |
| `CheckIns` | `["checkins", event]` | Zero-copy bitmap indexed by Bubblegum leaf nonce; current maximum is 80,000 tickets per event. |
| `PurchaseCounter` | `["purchases", event, wallet]` | Program-enforced wallet purchase count. |
| `Listing` | `["listing", event, asset_id, seller]` | Seller-bound resale ask; rent is returned when closed. |

The registry is a second SPL ConcurrentMerkleTree. ENTRY initializes it with
the Event PDA as authority and stores the current root and monotonic update
sequence on Event. Registry leaf `nonce` is a domain-separated hash of the
event, nonce, and owner-of-record. Purchase replaces the initially empty leaf;
resale verifies the current seller proof and replaces it with the buyer leaf
in the same transaction as payment and Bubblegum transfer. The tree uses a
64-entry changelog buffer; supported depth is 14 for supplies up to 16,384
and 20 for the current 80,000-ticket event limit. The registry tree account
must be allocated by the client with the matching SPL account-compression
size before `create_event`; ENTRY initializes it and signs as its authority.

Asset IDs are derived from the configured Bubblegum tree and nonce. Listing
does not prove ownership; settlement supplies the listing signer as the leaf
owner and relies on Bubblegum to reject an invalid leaf/proof. A failed CPI
should roll back earlier token transfers atomically; that behavior has not
yet been verified across ENTRY, SPL Token, Bubblegum, and compression
processors in one runtime test.

### Target account additions

These are target-design additions, not current accounts:

| Account / data | Target role |
| --- | --- |
| `Tier` | Multiple price/supply/limit/sale-window configurations per event. The current single `Event` price and supply represent one implicit tier. |
| `MintReceipt` | Idempotency / replay receipt keyed by client order ID or a unique purchase nonce. Current nonce serialization and `sold` count protect tree ordering, but there is no receipt PDA. |
| `Nullifier` | One-human-per-tier anti-sybil state, only after a ZK verification design is selected. |
| `Checkpoint` | Signed, sequenced admitted-set roots for offline scanners, after scanner key management and replay/rollback handling are designed. |
| `Vault` | Only for an explicitly approved escrow/refund mode. Current settlement is direct USDC transfer; there is no escrow. |

The supplied architecture's `["event", event_id]` seed is intentionally not
used: this repository already keys events by organizer and ID, avoiding
cross-organizer collisions. Likewise, keep the existing seller-specific
listing seed instead of the proposed leaf-only listing seed to prevent
listing squats.

### Authority and asset setup

The intended model assigns event operations to the Event PDA: private
Bubblegum tree delegation, collection authority for mint, and the relevant
MPL Core permanent delegate plugins. Organizer signatures configure the
event; the gate authority signs check-in; the configured gatekeeper co-signs
purchase. The program ID remains the value in `Anchor.toml` and `lib.rs`.

`create_event` checks Bubblegum tree and tree-config ownership, V2 tree
version, private mode, Event PDA delegation, zero prior mints, declared
capacity, depth, and 64-entry changelog buffer. It also decodes the MPL Core
collection and requires the `BubblegumV2`, permanent transfer delegate, and
permanent freeze delegate plugins with Event-controlled authority. These
checks have source/unit coverage and a real-compression runtime test for event
creation; they do not prove Bubblegum's later mint, freeze, thaw, or transfer
behavior. The collection policy still needs a real Bubblegum/Core runtime test
before production claims are safe.

### Current instruction flows

| Instruction | Current behavior |
| --- | --- |
| `init_config` | Initializes fee config; initializer must be the program upgrade authority. |
| `create_event` | Creates one event and its check-in bitmap after validating tree version, ownership, privacy, capacity, depth/buffer and Core plugins; initializes the registry CMT through the real compression CPI. |
| `set_active`, `set_gate_authority` | Organizer controls sales and rotates the check-in key. |
| `purchase` | Buyer and gatekeeper sign; checks active state, supply, sequential tree nonce, wallet limit, USDC mint/ATAs; transfers USDC, calls Bubblegum V2 `mintV2`, then attempts `freezeV2` in the same atomic instruction. |
| `list_for_resale` / `cancel_listing` | Seller-scoped listing; enforces face-price cap and rejects checked-in tickets; cancellation is seller-authorized. |
| `buy_resale` | Rechecks cap, check-in and wallet limit; splits checked USDC; attempts thaw/transfer/freeze CPIs; closes listing. |
| `check_in_batch` | Gate-authority signature; bounded batch of owner/proof pairs, rejects unsold nonces and invalid registry owners, idempotently sets bitmap bits and emits events. |

Primary and resale payments use the legacy SPL Token program and six-decimal
USDC allowlisting. Purchase currently splits organizer proceeds and platform
fee. Resale splits seller proceeds, organizer royalty, and platform fee.
Token-2022 tickets/add-ons are outside the current scope.

### Target instruction and state flow

The registry update and current-root owner proof check-in are present in the
source. The Account Compression CPI wrapper and its purchase/resale atomic use,
tree allocation, and client proof generation still need runtime/integration
coverage before the registry is considered production-proven. Keep
the bitmap as the on-chain single-use check-in record unless measurements
show checkpoint accounts should replace it; a future checkpoint must define
scanner signatures, sequence/replay protection, stale-state windows, and
conflict handling for disconnected scanners.

After the registry is established, expand the single-tier model to `Tier`,
add an idempotent receipt/order key, and add gift/recovery only with explicit
cooldown and authorization rules. Human verification progresses from the
existing gatekeeper co-sign to an anti-sybil provider behind a stable
interface; ZK nullifiers require a separately audited verifier and replay
tests.

## Payments and resale invariants

- Resale cap is always computed from original face price; it does not compound.
- Settlement uses checked integer arithmetic and basis points. The fee split
  must sum exactly to gross, with a defined remainder recipient and no dust.
- Check-in rejects later listing and resale; the bitmap only changes from 0
  to 1. Duplicate batch inputs do not fail the batch.
- Wallet limits are enforced by program state. The current counter is a
  wallet limit, not proof of one human per wallet/device.
- Atomic transaction failure must revert both payment and ticket/state
  changes. Prove this for successful and failing CPI paths with validator
  integration tests before launch.
- Registry ownership is the intended admission authority;
  ticket NFT metadata or wallet display alone is never proof of admission.

## Client and service flows

### Wallet purchase

The intended flow fetches current event and tree state, obtains a fresh DAS
proof/root, builds the purchase instruction, and asks the wallet to sign. The
gatekeeper co-signature is produced by a trusted backend only after its
verification succeeds; its key must never reach the browser or logs. Any
proof-dependent post-mint freeze flow must be simulated and runtime-proven
against tree races before exposing it to buyers.

The Blinks endpoints currently return demo metadata or HTTP 501. They do not
fetch event state, verify a user, build a transaction, partially sign, or
submit it. A production builder needs bounded input parsing, event/account
validation, rate limits, transaction-size/compute handling, and safe key
management. A v0 transaction/Address Lookup Table and explicit compute
budget should be evaluated for resale; the proposed 400,000-unit starting
limit is unmeasured and not a guarantee.

### Walletless checkout and social surfaces

Privy (or another embedded-wallet provider), Apple Pay/on-ramp, and
export-to-wallet are optional product integrations and are absent. X may
render compatible Actions/Blinks; Instagram and TikTok should use a PWA link
out unless their platform behavior changes and is verified.

### Gate and offline operation

The current scanner UI is a placeholder. Current on-chain check-in provides
online, global single-use state when all scans submit to the same program
state. `check_in_batch` takes `(nonce, owner)` for each scan and the matching
registry sibling accounts in ticket order. Proofs must match the current
`Event.registry_root`; the transaction supports at most four tickets at depth
14 or three at depth 20. Truly offline admission is future work: preload a
registry/checkpoint root and proofs, use signed short-lived ticket challenges,
protect device keys, journal scans locally, and design reconciliation for
multiple disconnected scanners. An offline scan cannot prevent two isolated scanners
from accepting the same ticket before they sync. Do not claim a guaranteed
400 ms offline flow until measured on target devices and tested under
reconciliation/race conditions.

## Off-chain components (target system)

1. **Actions / transaction API:** surface-agnostic metadata and transaction
   construction for compatible Blink clients and the PWA.
2. **Proof service:** DAS-backed proof fetch/cache with freshness checks;
   later, a Rust service may apply tree changelog deltas.
3. **Indexer:** confirmed program/Bubblegum events into a database with
   replay-safe slot/signature cursors; use an account/transaction source such
   as Yellowstone only after operations and reorg semantics are specified.
4. **Identity adapter:** gatekeeper verification first; optional phone/device
   checks and later audited ZK humanity proofs. Store PII off-chain, encrypted
   and minimized.
5. **Organizer and notification services:** event management, reconciliation,
   exports, email/push, and payout reporting after on-chain settlement is
   correct.

None of these production services is present in this repository today.

## Delivery phases and files

| Phase | Deliverable | Main files | Required verification |
| --- | --- | --- | --- |
| 0 — buildable MVP core | Stable IDL/account layout, event/tree provisioning validation, registry CPI and proof-tested purchase/resale. | `programs/entry/src/state/`, `instructions/create_event.rs`, `purchase.rs`, `buy_resale.rs`, `programs/entry/tests/`, `scripts/create-tree.ts`, `idls/` | Anchor/SBF build; validator tests for plugin/tree validation, mint/freeze, registry initialization/update/admission, resale sequence, payment rollback, race/stale proofs and direct transfer. |
| 1 — wallet and Actions | Real buy/resale transaction builders and secure gatekeeper co-sign service. | `app/blinks-api/`, new shared Solana client module, app manifests/lockfile, `idls/` | Typecheck; Actions protocol tests; transaction simulation; no key material in client bundle/logs; tests for malformed input, wrong event, missing signer, stale proof and cap. |
| 2 — scanning | Online scan/redeem flow, then signed checkpoint/offline cache. | `app/gate-pwa/`, new scanner/storage modules, on-chain checkpoint instruction/state, `programs/entry/tests/` | Duplicate scan, replay, forged signature, stale root, cross-event, disconnected scanner conflict and resync tests. |
| 3 — organizer and operations | Provisioning, indexer, proof cache, reconciliation, monitoring and export. | `app/organizer-web/`, `scripts/`, new service packages, deployment configuration | End-to-end devnet rehearsal, incident/pause runbook, reconciliation/idempotency tests. |
| 4 — optional modes | Multiple tiers, gifts/recovery, ZK nullifiers, walletless/on-ramp, escrow only if legally/operationally approved. | New state/instructions/services and dedicated test suites | Independent security review, privacy review, audit and jurisdiction-specific acceptance. |

Dependency versions and the program ID are pinned by the existing project
configuration; any needed new client dependency must be added without
upgrading existing versions and with its lockfile reviewed.

## Release gate

The current source is not production/deploy-ready for client use. Rust unit
and ProgramTest coverage, and an SBF build, are necessary but do not prove
Bubblegum CPI behavior or client transaction construction. Do not deploy or
advertise enforced resale/transfer policy until Phase 0 runtime invariants
pass, and do not enable clients until the relevant Phase 1/2 flows are
implemented and tested. See [implementation status](implementation-status.md)
and [threat model](threat-model.md).
