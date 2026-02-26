# SwappingSats

**Alpha** — A truly decentralized, self-custodial exchange for Liquid Network assets.

SwappingSats lets two parties swap **L-BTC** and **USDT** on the Liquid Network with no
custodian, no central server, and no trusted third party.  Nostr relays serve as a
censorship-resistant order book; [LiquiDEX](https://github.com/Blockstream/lwk/tree/master/lwk_wollet/src/liquidex)
atomic swaps guarantee that either both legs of the trade settle on-chain or neither does.

---

## Table of Contents

- [How It Works](#how-it-works)
- [Key Design Decisions](#key-design-decisions)
- [Tech Stack](#tech-stack)
- [Project Structure](#project-structure)
- [Prerequisites](#prerequisites)
- [Running Locally](#running-locally)
- [Swap Lifecycle](#swap-lifecycle)
- [Nostr Event Format](#nostr-event-format)
- [Network Configuration](#network-configuration)
- [Known Limitations](#known-limitations)

---

## How It Works

```
┌─────────────────────────────────────────────────────────────────┐
│                        MAKER (Alice)                            │
│  1. Selects a UTXO to offer (e.g. 1 000 000 sats of L-BTC)     │
│  2. Enters the amount she wants back (e.g. 650 USDT)           │
│  3. App creates a half-signed PSET (LiquiDEX proposal)         │
│  4. Serialises it to JSON and publishes a Nostr kind-32121 note │
└─────────────────────────────────────────────────────────────────┘
                              │  Nostr relays
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        TAKER (Bob)                              │
│  5. Sees Alice's offer in the order book                        │
│  6. App fetches the referenced tx from Esplora & validates it   │
│  7. Adds his own UTXOs, signs, finalises, and broadcasts       │
│  8. A single Liquid tx atomically delivers both assets          │
└─────────────────────────────────────────────────────────────────┘
```

No routing, no channels, no counter-party risk during the swap.  The only trust
assumption is the Liquid Network itself (a federated sidechain).

---

## Key Design Decisions

### Single key for both identities

A BIP-39 mnemonic is derived from the user's Nostr secret key.  The same seed
powers both the Nostr identity (used to sign order-book events) and the Liquid
wallet (used to sign swap transactions).  Users back up one 24-word phrase and
recover everything.

### Nostr as the order book

Swap offers are Nostr **kind `32121`** events.  Any Nostr relay can carry them;
the protocol imposes no special infrastructure.  Offers expire naturally (the
subscription filters for the last hour) and cancellations are just additional
events — no delete API needed.

### LiquiDEX atomic swaps

A LiquiDEX proposal is a Partially Signed Elements Transaction (PSET) where
the maker has signed only their own input.  The taker completes and broadcasts
it.  Because PSETs are atomic, it is cryptographically impossible for the taker
to take the maker's asset without simultaneously delivering the requested asset.

### Browser-native, no server

The app is compiled to WebAssembly and runs entirely in the browser.  Wallet
state is persisted in IndexedDB; the only network calls are WebSocket connections
to Nostr relays and HTTP calls to a public Esplora API.

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend framework | [Yew](https://yew.rs/) 0.21 (Rust → WASM) |
| Styling | [TailwindCSS](https://tailwindcss.com/) v4 via `tailwindcss4` CLI |
| UI components | [shady-minions](https://crates.io/crates/shady-minions) |
| Build tool | [Trunk](https://trunkrs.dev/) |
| Liquid wallet | [LWK (Liquid Wallet Kit)](https://github.com/Blockstream/lwk) 0.11 |
| Atomic swaps | LiquiDEX (built into LWK) |
| Blockchain data | Esplora HTTP API (waterfalls variant) |
| Nostr protocol | [nostr-minions](https://crates.io/crates/nostr-minions) 0.1 |
| Local storage | IndexedDB via [idb](https://crates.io/crates/idb) 0.6 |
| Async runtime | Tokio (WASM-compatible subset) |

---

## Project Structure

```
swappingsats/
├── src/
│   ├── main.rs                          # App root, relay config, WalletSync
│   ├── router.rs                        # Client-side routing
│   ├── components/
│   │   ├── icons.rs                     # SVG icon components
│   │   └── toast.rs                     # Toast notification helpers
│   ├── contexts/
│   │   ├── orderbook_provider.rs        # Nostr subscription + order book state
│   │   ├── wallet_provider/
│   │   │   ├── wallet.rs                # LiquidWebWallet (LWK wrapper)
│   │   │   ├── provider.rs              # Yew state layer, swap hooks
│   │   │   └── wallet_provider.rs       # Wallet context setup
│   │   └── persister/
│   │       ├── mod.rs                   # PersistedSwap, IdbUpdate types
│   │       ├── handler.rs               # IndexedDB read/write operations
│   │       ├── hooks.rs                 # use_nostrades_db hook
│   │       └── provider.rs              # DB context provider
│   └── pages/
│       ├── login.rs                     # Key generation & recovery UI
│       ├── home.rs                      # Dashboard: balances & my orders
│       ├── swaps.rs                     # Create swap offer UI
│       ├── orderbook.rs                 # Browse & take peer offers
│       ├── history.rs                   # Transaction history
│       ├── send_coins.rs                # Send asset to address
│       └── receive_coins.rs             # Display receive address
├── bitcoin_qr/                          # QR code component crate
├── shady-minions/                       # UI component library crate
├── styles/
│   ├── input.css                        # Tailwind source
│   └── output.css                       # Generated (committed for convenience)
├── public/                              # Static assets (logo, icons)
├── index.html                           # Trunk entry point
├── Trunk.toml                           # Trunk build config (serves on :2100)
└── Cargo.toml
```

---

## Prerequisites

| Tool | Purpose | Install |
|------|---------|---------|
| Rust + `wasm32-unknown-unknown` target | Compile to WASM | `rustup target add wasm32-unknown-unknown` |
| [Trunk](https://trunkrs.dev/) | WASM bundler & dev server | `cargo install trunk` |
| `tailwindcss4` CLI | Generate CSS | See [Tailwind v4 docs](https://tailwindcss.com/docs/installation) |

Trunk automatically runs `tailwindcss4 -i styles/input.css -o styles/output.css --minify`
before every build (configured in `Trunk.toml`).

---

## Running Locally

```bash
# Clone the repo
git clone <repo-url>
cd swappingsats

# Start the development server (hot-reload on src/ and styles/ changes)
trunk serve
```

The app will be available at **http://localhost:2100**.

For a production build:

```bash
trunk build --release
# Output is written to dist/
```

---

## Swap Lifecycle

### 1. Identity setup

On first visit the user is prompted to either generate a new key or recover an
existing one.  SwappingSats supports two key formats:

- **24-word BIP-39 seed phrase** — the primary backup format.
- **`nsec` secret key** — the Nostr standard hex-encoded secret key.

The key is stored in the browser's secure storage via the Web Crypto API and
is not transmitted anywhere.

### 2. Wallet sync

`WalletSync` (mounted in `main.rs`) triggers a full Esplora scan on load,
applying the resulting LWK `Update` to the in-memory wallet and persisting it
to IndexedDB so future loads can resume from the last-known chain tip.

### 3. Making an offer

1. Navigate to **Swaps** and select a UTXO from the list of available outputs.
2. Enter the amount of the counterparty asset you want in return.
3. Click **Create Swap Offer**.

Internally:

```
wallet.create_swap_offer(utxo, requested_amount, target_asset_id, &db)
  └─ wallet.liquidex_proposal()     // build + half-sign PSET
  └─ db.push_locked_utxo(utxo)      // lock UTXO in IndexedDB

NostrNote {
  kind:    32121,
  content: serde_json::to_string(&proposal),
  tags:    [["param", "<txid>:<vout>"]],
}
nostr_key.sign_note(&mut note)
relay.send(note)                     // broadcast to both relays
```

The UTXO is locked locally to prevent it from being spent in another operation
while the offer is open.

### 4. Browsing the order book

`OrderBookProvider` maintains a live Nostr subscription for kind-`32121` events
from the last hour.  Arriving notes are routed:

| Condition | Result |
|-----------|--------|
| `content == "canceled"` | Remove matching offer from the book |
| Content is not a valid `LiquidexProposal` | Ignore |
| `pubkey` matches current user | Add to *My Offers* |
| Otherwise | Add to *Order Book* |

Before the order book renders, `parsed_offers()` validates each stored
proposal on-chain:

1. Parse JSON content → `LiquidexProposal<Unvalidated>`
2. Extract the required transaction ID via `proposal.needed_tx()`
3. Batch-fetch transactions from Esplora
4. Call `proposal.validate(tx)` — only proposals whose UTXO still exists
   and whose amounts match the chain record are displayed

Offers already recorded in IndexedDB (accepted or failed) are filtered out
to prevent duplicate attempts.

### 5. Taking an offer

Click **Buy** or **Sell** on any order book entry.

```
wallet.liquidex_take(validated_proposal, note_id, &db)
  └─ wallet.liquidex_take(utxos, proposal)
       └─ build PSET (taker side)
       └─ sign taker inputs
       └─ finalize PSET → complete Liquid tx
       └─ ESPLORA_CLIENT.broadcast(tx)   // atomic on-chain settlement
  └─ db.push_swap(PersistedSwap { id: note_id, status: Accepted })
```

On failure the swap is recorded with `SwapStatus::Failed` so the offer is
hidden from the UI.

### 6. Cancelling an offer

Two cancellation strategies are available from the **Locked UTXOs** list:

| Strategy | What it does |
|----------|--------------|
| **Soft cancel** | Publishes a `"canceled"` Nostr event; unlocks the UTXO locally.  Fast and free, but relies on peers seeing the event. |
| **Hard cancel** | Spends the UTXO on-chain to the platform address (420 sats), permanently invalidating the proposal regardless of relay state.  Also publishes a soft-cancel event. |

---

## Nostr Event Format

### Swap offer

```json
{
  "kind": 32121,
  "content": "{\"input\":{\"txid\":\"...\",\"vout\":0,\"asset\":\"...\",\"amount\":1000000},\"output\":{...},\"transaction\":\"...\"}",
  "tags": [["param", "<txid>:<vout>"]],
  "pubkey": "<maker hex pubkey>",
  "created_at": 1700000000,
  "id": "<event hash>",
  "sig": "<schnorr signature>"
}
```

The `content` is the JSON serialisation of a `LiquidexProposal<Unvalidated>`
as produced by LWK.  The `param` tag contains `"txid:vout"` of the offered UTXO
and is used to match cancellation events to offers.

### Cancellation

```json
{
  "kind": 32121,
  "content": "canceled",
  "tags": [["param", "<txid>:<vout>"]],
  "pubkey": "<maker hex pubkey>",
  ...
}
```

---

## Network Configuration

> **⚠️ This software currently operates on Liquid Testnet only.**

| Parameter | Value |
|-----------|-------|
| Network | Liquid Testnet |
| Esplora API | `https://waterfalls.liquidwebwallet.org/liquidtestnet/api` |
| Nostr relay 1 | `wss://no.str.cr` |
| Nostr relay 2 | `wss://relay.illuminodes.com` |
| L-BTC asset ID | `144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49` |
| USDT asset ID | `38fca2d939696061a8f76d4e6b5eecd54e3b4221c846f24a6b279e79952850a5` |
| Fee address | `tlq1qqv8caryh8kdy6v3mgn6cljngks9geedrcdsxa8eav5l2p8hmcz3kedv082nkdurnjta8rrt2wjlhgk86mlhk5r2tjt0hkp4ty` |

Testnet L-BTC and USDT can be obtained from the
[Liquid Testnet Faucet](https://liquidtestnet.com/).

---

## Known Limitations

- **Testnet only** — not yet configured for Liquid mainnet.
- **Static market rate** — the suggested price uses a hardcoded $65 000/BTC;
  no live price feed is implemented.
- **Whole-UTXO offers only** — a swap offer covers the entire value of the
  selected UTXO; partial fills are not supported.
- **No continuous sync** — the wallet syncs once on page load and when the
  sync button is clicked; background polling is commented out.
- **Single asset pair** — only L-BTC ↔ USDT is supported.
- **Soft-cancel race** — between a soft-cancel event being published and a
  taker seeing it, a swap can still be taken.  Use hard-cancel for certainty.
- **Fee rate** — transaction fee rate is hardcoded at 100 sat/vB.
