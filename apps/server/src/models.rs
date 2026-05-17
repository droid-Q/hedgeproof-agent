use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub exposure: String,
    pub asset_type: Option<String>,
    pub notional_usd: Option<f64>,
    pub budget_usd: Option<f64>,
    pub horizon_days: Option<u32>,
    pub chain_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    pub quote_id: String,
    pub intent_summary: String,
    pub risk_tag: String,
    pub hedge_budget_usd: f64,
    pub max_loss_usd: f64,
    pub protection_band_usd: f64,
    pub confidence: f64,
    pub market_matches: Vec<MarketMatch>,
    pub hedge_legs: Vec<HedgeLeg>,
    pub reasoning_trace: Vec<String>,
    pub receipt: ReceiptPayload,
    pub disclaimers: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketMatch {
    pub venue: String,
    pub market: String,
    pub probability: f64,
    pub liquidity_usd: f64,
    pub basis_risk: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HedgeLeg {
    pub action: String,
    pub instrument: String,
    pub estimated_price: f64,
    pub budget_usd: f64,
    pub payout_if_triggered_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptPayload {
    pub quote_hash: String,
    pub summary_hash: String,
    pub expires_at: String,
    pub budget_ceiling_usd: u64,
    pub risk_tag: String,
    pub chain_hint: String,
    pub contract_address: Option<String>,
    pub tx_hash: Option<String>,
    pub solidity_args: SolidityReceiptArgs,
    pub contract_call: ContractCallPayload,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolidityReceiptArgs {
    pub quote_id: String,
    pub quote_hash: String,
    pub summary_hash: String,
    pub expires_at_unix: i64,
    pub budget_ceiling_usd: u64,
    pub risk_tag: String,
    pub chain_hint: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContractCallPayload {
    pub contract_name: String,
    pub function_signature: String,
    pub method_id: String,
    pub calldata: String,
    pub to: Option<String>,
    pub value_wei: String,
    pub chain_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoScenario {
    pub title: String,
    pub exposure: String,
    pub asset_type: String,
    pub notional_usd: f64,
    pub budget_usd: f64,
    pub horizon_days: u32,
}
