# ENTRY

**Fair ticketing with programmable resale on Solana.** This repository is a development scaffold for a demo-day MVP, not a production ticketing system.

## What this scaffold targets

- Anchor program for event configuration, purchase limits, resale listings, price-cap arithmetic, and check-in state.
- Bubblegum V2 + MPL-Core collection integration boundary.
- USDC payment-flow design.
- Solana Actions/Blinks API scaffold.
- Organizer, wallet, and gate app placeholders.
- Repeatable scripts and tests for the demo path.

## Important implementation boundary

The Anchor source now contains Bubblegum V2 mint/freeze and thaw/transfer/freeze CPI paths, USDC settlement, per-event check-in bitmaps, and gatekeeper co-sign authorization. These paths compile, but they have not been validated against a running Bubblegum V2/Core deployment. The Blink transaction builders, gatekeeper human verification and partial signing, collection-plugin validation, and proof acquisition are not implemented. Do not deploy this scaffold to mainnet or use real customer payments. See [implementation status](docs/implementation-status.md) for the current boundary.

## Requirements

- Node.js 22+
- pnpm 9+
- Rust + Solana CLI + Anchor CLI (versions pinned by your installed toolchain)
- A DAS-enabled Solana RPC for Bubblegum V2 asset/proof queries

## Setup

```bash
pnpm install
cp .env.example .env
```

Configure `RPC_URL`, `DAS_URL`, `USDC_MINT`, and a devnet-only `PAYER_KEYPAIR` path. Never commit private keys or real credentials.

## Workspace commands

```bash
pnpm typecheck
pnpm test
pnpm idl
pnpm dev:organizer
pnpm dev:wallet
pnpm dev:gate
pnpm dev:clients
pnpm dev:blinks
```

The role-specific client pages run on separate local ports:

- Organizer: http://localhost:3001
- Wallet: http://localhost:3002
- Gate: http://localhost:3003

Run one role with its individual command, or start all three with
`pnpm dev:clients`. These are preview interfaces; wallet connection, event
creation, ticket lookup and checkout, and camera scanning and redemption are
not yet integrated.

Anchor commands depend on your local Anchor/Solana installation:

```bash
anchor build
anchor test
```

## Intended demo sequence (requires integration work)

1. Create an event and prepare a Bubblegum V2 tree/collection.
2. Configure the Core collection plugins and a private Bubblegum V2 tree delegated to the event PDA.
3. Complete a purchase through a gatekeeper-co-signed transaction and verify the minted leaf is frozen.
4. Create a resale listing, verify the face-price cap, then complete settlement with fresh sequential proofs.
5. Check in a ticket and verify that listing and resale are rejected afterward.

The purchase/resale Blink builders and validator integration tests are still required before this sequence can run end to end.

## Security and product limitations

- Online check-in is the default. Offline scanners can accept the same ticket independently if they have stale snapshots; offline mode cannot guarantee global single-use redemption without a coordination design.
- Wallet-based purchase caps do not stop one person using multiple wallets.
- QR rotation reduces replay opportunities but does not establish human identity.
- Blinks are supported only in compatible Actions-aware clients; always provide a browser checkout fallback.
- Fees and cNFT costs depend on current provider pricing, tree configuration, transaction fees, and infrastructure. Measure before advertising a price claim.
- Obtain legal/privacy review before handling real ticket buyers or payments.

## Current docs

- Bubblegum V2: https://www.metaplex.com/docs/smart-contracts/bubblegum-v2
- Bubblegum V2 transfers: https://www.metaplex.com/docs/smart-contracts/bubblegum-v2/transfer-cnfts
- Solana Actions/Blinks: https://solana.com/docs/tools/actions

See `docs/architecture.md` and `docs/threat-model.md` before implementing the remaining CPI paths.
