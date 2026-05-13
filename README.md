# HedgeProof Agent

Agentic RWA hedge quote desk for the Arbitrum Open House buildathon.

HedgeProof Agent turns a natural-language risk exposure into a prediction-market hedge quote, then prepares a verifiable quote receipt payload that can be recorded on Arbitrum or Robinhood Chain Testnet.

It is a quote-only demo. It does not place trades, custody funds, sell insurance, or provide financial advice.

## What is included

- Vue 3 quote desk UI for RWA, crypto, and event-risk scenarios.
- Rust Axum API with deterministic agent reasoning, market matching, quote math, and receipt hashing.
- Solidity `QuoteReceiptRegistry` contract for on-chain quote receipts.
- Demo scenarios and submission notes for hackathon presentation.

## Run locally

```bash
pnpm install
cp .env.example .env
pnpm dev:server
pnpm dev:web
```

Open the Vite URL, normally `http://127.0.0.1:5173`.

## Verify

```bash
cargo test --manifest-path apps/server/Cargo.toml
pnpm --filter @hedgeproof/web build
pnpm --filter @hedgeproof/contracts test
```

## Demo positioning

HedgeProof is designed to compete for Agentic Project and overall tracks:

- The agent transforms a messy exposure into a structured hedge quote.
- The quote is transparent: reasoning, assumptions, budget, loss cap, and market legs are visible.
- The blockchain integration records a tamper-evident receipt hash without touching user funds.
- The RWA angle is explicit through stock, ETF, treasury, and tokenized-asset risk examples.
