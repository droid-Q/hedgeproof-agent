use crate::models::{
    DemoScenario, HedgeLeg, MarketMatch, QuoteRequest, QuoteResponse, ReceiptPayload,
    SolidityReceiptArgs,
};
use crate::solidity;
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use std::env;
use uuid::Uuid;

pub fn demo_scenarios() -> Vec<DemoScenario> {
    vec![
        DemoScenario {
            title: "Tokenized equity earnings shock".to_string(),
            exposure: "I hold $25,000 of tokenized NVDA-like equity exposure and I am worried about a negative earnings surprise in the next 21 days.".to_string(),
            asset_type: "RWA equity".to_string(),
            notional_usd: 25_000.0,
            budget_usd: 950.0,
            horizon_days: 21,
        },
        DemoScenario {
            title: "ETH drawdown protection".to_string(),
            exposure: "I hold $18,000 of ETH and want protection if it drops sharply before the next ETF flow report.".to_string(),
            asset_type: "Crypto".to_string(),
            notional_usd: 18_000.0,
            budget_usd: 720.0,
            horizon_days: 30,
        },
        DemoScenario {
            title: "Operations event contingency".to_string(),
            exposure: "Our shipping business loses money if London transit strikes intensify this month.".to_string(),
            asset_type: "Event risk".to_string(),
            notional_usd: 12_000.0,
            budget_usd: 480.0,
            horizon_days: 18,
        },
    ]
}

pub fn build_quote(request: QuoteRequest) -> QuoteResponse {
    let exposure = clean_exposure(&request.exposure);
    let risk_tag = classify_risk(&exposure, request.asset_type.as_deref());
    let horizon_days = request.horizon_days.unwrap_or(21).clamp(1, 90);
    let notional = request.notional_usd.unwrap_or(15_000.0).clamp(1_000.0, 1_000_000.0);
    let default_budget = (notional * 0.04).clamp(250.0, 5_000.0);
    let hedge_budget = request
        .budget_usd
        .unwrap_or(default_budget)
        .clamp(100.0, notional * 0.12);
    let chain_hint = request
        .chain_hint
        .unwrap_or_else(|| "Robinhood Chain Testnet".to_string());

    let markets = market_matches(&risk_tag, notional);
    let avg_price = weighted_price(&markets);
    let payout_if_triggered = hedge_budget / avg_price.max(0.05);
    let protection_band = (payout_if_triggered - hedge_budget).max(0.0).min(notional * 0.42);
    let confidence = confidence_score(&risk_tag, markets.len(), protection_band / notional);

    let hedge_legs = vec![HedgeLeg {
        action: "Buy YES / downside-proxy outcome".to_string(),
        instrument: markets[0].market.clone(),
        estimated_price: avg_price,
        budget_usd: round2(hedge_budget),
        payout_if_triggered_usd: round2(payout_if_triggered),
    }];

    let quote_id = Uuid::new_v4().to_string();
    let intent_summary = summarize_intent(&risk_tag, notional, horizon_days, &exposure);
    let reasoning_trace = reasoning_trace(&risk_tag, &exposure, notional, hedge_budget, &markets);
    let expires_at = Utc::now() + Duration::days(i64::from(horizon_days));
    let summary_hash = hash_hex(&intent_summary);
    let quote_hash = hash_hex(&format!(
        "{quote_id}|{summary_hash}|{risk_tag}|{hedge_budget:.2}|{protection_band:.2}|{}",
        expires_at.timestamp()
    ));
    let quote_id_bytes32 = hash_hex(&quote_id);
    let budget_ceiling_usd = hedge_budget.ceil() as u64;

    let contract_address = env::var("QUOTE_RECEIPT_CONTRACT_ADDRESS").ok().filter(|v| !v.is_empty());
    let solidity_args = SolidityReceiptArgs {
        quote_id: format!("0x{quote_id_bytes32}"),
        quote_hash: format!("0x{quote_hash}"),
        summary_hash: format!("0x{summary_hash}"),
        expires_at_unix: expires_at.timestamp(),
        budget_ceiling_usd,
        risk_tag: risk_tag.clone(),
        chain_hint: chain_hint.clone(),
    };
    let contract_call = solidity::build_create_receipt_call(&solidity_args, contract_address.clone())
        .expect("generated receipt args must ABI-encode");

    let receipt = ReceiptPayload {
        quote_hash: format!("0x{quote_hash}"),
        summary_hash: format!("0x{summary_hash}"),
        expires_at: expires_at.to_rfc3339(),
        budget_ceiling_usd,
        risk_tag: risk_tag.clone(),
        chain_hint: chain_hint.clone(),
        contract_address,
        tx_hash: None,
        solidity_args,
        contract_call,
    };

    QuoteResponse {
        quote_id,
        intent_summary,
        risk_tag,
        hedge_budget_usd: round2(hedge_budget),
        max_loss_usd: round2(hedge_budget),
        protection_band_usd: round2(protection_band),
        confidence,
        market_matches: markets,
        hedge_legs,
        reasoning_trace,
        receipt,
        disclaimers: vec![
            "Quote-only demo: no trade execution, custody, or insurance issuance.".to_string(),
            "Market matches may include demo proxies when live venue credentials are unavailable.".to_string(),
            "On-chain receipt proves quote integrity, not economic performance.".to_string(),
        ],
    }
}

fn clean_exposure(exposure: &str) -> String {
    let trimmed = exposure.trim();
    if trimmed.is_empty() {
        "User has an unspecified portfolio risk and wants bounded-loss hedge guidance.".to_string()
    } else {
        trimmed.to_string()
    }
}

fn classify_risk(exposure: &str, asset_type: Option<&str>) -> String {
    let asset_text = asset_type.unwrap_or_default().to_lowercase();
    let exposure_text = exposure.to_lowercase();
    let text = format!("{exposure_text} {asset_text}");

    if contains_any(&asset_text, &["crypto", "eth", "btc", "bitcoin", "ethereum", "sol"])
        || contains_any(&exposure_text, &["eth", "btc", "bitcoin", "ethereum"])
    {
        "CRYPTO_DRAWDOWN".to_string()
    } else if contains_any(&text, &["earnings", "stock", "equity", "etf", "tokenized", "treasury", "rwa"]) {
        "RWA_EARNINGS_GAP".to_string()
    } else {
        "EVENT_CONTINGENCY".to_string()
    }
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn market_matches(risk_tag: &str, notional: f64) -> Vec<MarketMatch> {
    let liquidity_base = (notional * 18.0).clamp(75_000.0, 2_500_000.0);
    match risk_tag {
        "RWA_EARNINGS_GAP" => vec![
            MarketMatch {
                venue: "Polymarket proxy".to_string(),
                market: "Major tech equity closes down 8% after earnings window".to_string(),
                probability: 0.31,
                liquidity_usd: round2(liquidity_base),
                basis_risk: "Medium: tokenized equity exposure may not match the listed event exactly.".to_string(),
            },
            MarketMatch {
                venue: "Kalshi-style proxy".to_string(),
                market: "Nasdaq weekly drawdown exceeds 5%".to_string(),
                probability: 0.24,
                liquidity_usd: round2(liquidity_base * 0.62),
                basis_risk: "High: broad index proxy only protects systemic shock.".to_string(),
            },
        ],
        "CRYPTO_DRAWDOWN" => vec![
            MarketMatch {
                venue: "Polymarket proxy".to_string(),
                market: "ETH trades below key support before quote expiry".to_string(),
                probability: 0.38,
                liquidity_usd: round2(liquidity_base * 1.4),
                basis_risk: "Low: exposure and hedge proxy reference the same asset family.".to_string(),
            },
            MarketMatch {
                venue: "Polymarket proxy".to_string(),
                market: "Crypto market cap falls more than 10% this month".to_string(),
                probability: 0.34,
                liquidity_usd: round2(liquidity_base),
                basis_risk: "Medium: broad basket proxy can diverge from ETH.".to_string(),
            },
        ],
        _ => vec![
            MarketMatch {
                venue: "Prediction market proxy".to_string(),
                market: "Named operational event escalates before expiry".to_string(),
                probability: 0.42,
                liquidity_usd: round2(liquidity_base * 0.55),
                basis_risk: "Medium-high: event wording and business loss trigger may differ.".to_string(),
            },
            MarketMatch {
                venue: "News-event proxy".to_string(),
                market: "Local disruption remains active for three or more days".to_string(),
                probability: 0.29,
                liquidity_usd: round2(liquidity_base * 0.35),
                basis_risk: "High: useful as a fallback indicator, not a precise hedge.".to_string(),
            },
        ],
    }
}

fn weighted_price(markets: &[MarketMatch]) -> f64 {
    let total_liquidity: f64 = markets.iter().map(|market| market.liquidity_usd).sum();
    if total_liquidity <= 0.0 {
        return 0.35;
    }
    let weighted: f64 = markets
        .iter()
        .map(|market| market.probability * market.liquidity_usd)
        .sum();
    round4(weighted / total_liquidity)
}

fn confidence_score(risk_tag: &str, match_count: usize, coverage_ratio: f64) -> f64 {
    let base = match risk_tag {
        "CRYPTO_DRAWDOWN" => 0.74,
        "RWA_EARNINGS_GAP" => 0.67,
        _ => 0.58,
    };
    let score = base + (match_count as f64 * 0.03) + coverage_ratio.min(0.2);
    round4(score.min(0.92))
}

fn summarize_intent(risk_tag: &str, notional: f64, horizon_days: u32, exposure: &str) -> String {
    format!(
        "{risk_tag}: quote hedge for ${:.0} notional over {horizon_days} days. Exposure: {exposure}",
        notional
    )
}

fn reasoning_trace(
    risk_tag: &str,
    exposure: &str,
    notional: f64,
    hedge_budget: f64,
    markets: &[MarketMatch],
) -> Vec<String> {
    vec![
        format!("Classified the exposure as {risk_tag} from: {exposure}"),
        format!("Normalized notional to ${:.0} and capped hedge spend at ${:.0}.", notional, hedge_budget),
        format!("Selected {} market proxies and ranked them by liquidity plus basis-risk fit.", markets.len()),
        "Computed payout by treating prediction-market price as the premium per $1 of triggered payout.".to_string(),
        "Prepared a receipt hash so the quote can be verified on-chain without exposing private portfolio data.".to_string(),
    ]
}

fn hash_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_rwa_quote_with_receipt_hashes() {
        let quote = build_quote(QuoteRequest {
            exposure: "I hold tokenized equity and fear earnings downside".to_string(),
            asset_type: Some("RWA equity".to_string()),
            notional_usd: Some(25_000.0),
            budget_usd: Some(900.0),
            horizon_days: Some(21),
            chain_hint: None,
        });

        assert_eq!(quote.risk_tag, "RWA_EARNINGS_GAP");
        assert_eq!(quote.hedge_budget_usd, 900.0);
        assert!(quote.protection_band_usd > 0.0);
        assert!(quote.receipt.quote_hash.starts_with("0x"));
        assert_eq!(quote.receipt.solidity_args.quote_hash, quote.receipt.quote_hash);
        assert_eq!(quote.receipt.contract_call.contract_name, "QuoteReceiptRegistry");
        assert_eq!(quote.receipt.contract_call.method_id, "0x3a09bc9e");
        assert!(quote.receipt.contract_call.calldata.starts_with("0x3a09bc9e"));
    }

    #[test]
    fn empty_input_still_returns_demo_quote() {
        let quote = build_quote(QuoteRequest {
            exposure: String::new(),
            asset_type: None,
            notional_usd: None,
            budget_usd: None,
            horizon_days: None,
            chain_hint: Some("Arbitrum Sepolia".to_string()),
        });

        assert_eq!(quote.receipt.chain_hint, "Arbitrum Sepolia");
        assert!(!quote.market_matches.is_empty());
        assert!(quote.max_loss_usd <= 5_000.0);
    }

    #[test]
    fn crypto_asset_type_wins_over_etf_keyword_in_context() {
        let quote = build_quote(QuoteRequest {
            exposure: "I hold ETH and worry about the next ETF flow report".to_string(),
            asset_type: Some("Crypto".to_string()),
            notional_usd: Some(18_000.0),
            budget_usd: Some(720.0),
            horizon_days: Some(30),
            chain_hint: None,
        });

        assert_eq!(quote.risk_tag, "CRYPTO_DRAWDOWN");
        assert!(quote.market_matches[0].market.contains("ETH"));
    }
}
