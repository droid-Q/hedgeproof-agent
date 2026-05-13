# Demo Scenarios

Use these three cases in the hackathon video.

## 1. RWA / equity earnings risk

Input:

> I hold $25,000 of a tokenized NVDA-like equity exposure and I am worried about a negative earnings surprise in the next 21 days.

Expected output:

- Risk tag: `RWA_EARNINGS_GAP`
- Hedge quote uses a downside earnings or market drawdown proxy.
- Receipt payload can be recorded on Robinhood Chain Testnet.

## 2. Crypto drawdown risk

Input:

> I hold $18,000 of ETH and want protection if it drops sharply before the next ETF flow report.

Expected output:

- Risk tag: `CRYPTO_DRAWDOWN`
- Hedge quote prioritizes ETH or broad crypto drawdown markets.
- Reasoning explains basis risk.

## 3. Event-driven operations risk

Input:

> Our shipping business loses money if London transit strikes intensify this month.

Expected output:

- Risk tag: `EVENT_CONTINGENCY`
- Hedge quote uses event-probability markets.
- Reasoning calls out liquidity and event-resolution mismatch.
