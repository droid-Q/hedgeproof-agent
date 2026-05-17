use crate::models::{ContractCallPayload, SolidityReceiptArgs};
use sha3::{Digest, Keccak256};

const CREATE_RECEIPT_SIGNATURE: &str =
    "createReceipt(bytes32,bytes32,bytes32,uint64,uint128,string,string)";
const CONTRACT_NAME: &str = "QuoteReceiptRegistry";

pub fn build_create_receipt_call(
    args: &SolidityReceiptArgs,
    contract_address: Option<String>,
) -> Result<ContractCallPayload, String> {
    let calldata = encode_create_receipt(args)?;
    let method_id = method_id(CREATE_RECEIPT_SIGNATURE);

    Ok(ContractCallPayload {
        contract_name: CONTRACT_NAME.to_string(),
        function_signature: CREATE_RECEIPT_SIGNATURE.to_string(),
        method_id: format!("0x{}", hex::encode(method_id)),
        calldata: format!("0x{}", hex::encode(calldata)),
        to: contract_address,
        value_wei: "0".to_string(),
        chain_id: chain_id_for_hint(&args.chain_hint),
    })
}

fn encode_create_receipt(args: &SolidityReceiptArgs) -> Result<Vec<u8>, String> {
    let risk_tag = encode_string(&args.risk_tag);
    let chain_hint = encode_string(&args.chain_hint);
    let static_slots = 7usize * 32usize;
    let risk_tag_offset = static_slots;
    let chain_hint_offset = static_slots + risk_tag.len();

    let mut encoded = Vec::with_capacity(4 + static_slots + risk_tag.len() + chain_hint.len());
    encoded.extend_from_slice(&method_id(CREATE_RECEIPT_SIGNATURE));
    encoded.extend_from_slice(&parse_bytes32(&args.quote_id)?);
    encoded.extend_from_slice(&parse_bytes32(&args.quote_hash)?);
    encoded.extend_from_slice(&parse_bytes32(&args.summary_hash)?);
    encoded.extend_from_slice(&encode_u64(args.expires_at_unix as u64));
    encoded.extend_from_slice(&encode_u128(args.budget_ceiling_usd as u128));
    encoded.extend_from_slice(&encode_usize(risk_tag_offset));
    encoded.extend_from_slice(&encode_usize(chain_hint_offset));
    encoded.extend_from_slice(&risk_tag);
    encoded.extend_from_slice(&chain_hint);
    Ok(encoded)
}

fn method_id(signature: &str) -> [u8; 4] {
    let digest = Keccak256::digest(signature.as_bytes());
    [digest[0], digest[1], digest[2], digest[3]]
}

fn parse_bytes32(value: &str) -> Result<[u8; 32], String> {
    let trimmed = value.strip_prefix("0x").unwrap_or(value);
    let bytes = hex::decode(trimmed).map_err(|err| format!("invalid bytes32 hex: {err}"))?;
    if bytes.len() != 32 {
        return Err(format!("bytes32 value must be 32 bytes, got {}", bytes.len()));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn encode_u64(value: u64) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[24..].copy_from_slice(&value.to_be_bytes());
    out
}

fn encode_u128(value: u128) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[16..].copy_from_slice(&value.to_be_bytes());
    out
}

fn encode_usize(value: usize) -> [u8; 32] {
    encode_u128(value as u128)
}

fn encode_string(value: &str) -> Vec<u8> {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(32 + padded_len(bytes.len()));
    out.extend_from_slice(&encode_usize(bytes.len()));
    out.extend_from_slice(bytes);
    out.resize(32 + padded_len(bytes.len()), 0);
    out
}

fn padded_len(value: usize) -> usize {
    if value == 0 {
        0
    } else {
        value.div_ceil(32) * 32
    }
}

fn chain_id_for_hint(chain_hint: &str) -> Option<u64> {
    if chain_hint.eq_ignore_ascii_case("Arbitrum Sepolia") {
        Some(421_614)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args() -> SolidityReceiptArgs {
        SolidityReceiptArgs {
            quote_id: "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
            quote_hash: "0x2222222222222222222222222222222222222222222222222222222222222222".to_string(),
            summary_hash: "0x3333333333333333333333333333333333333333333333333333333333333333".to_string(),
            expires_at_unix: 1_780_000_000,
            budget_ceiling_usd: 950,
            risk_tag: "RWA_EARNINGS_GAP".to_string(),
            chain_hint: "Arbitrum Sepolia".to_string(),
        }
    }

    #[test]
    fn computes_create_receipt_method_id() {
        assert_eq!(hex::encode(method_id(CREATE_RECEIPT_SIGNATURE)), "3a09bc9e");
    }

    #[test]
    fn encodes_create_receipt_calldata() {
        let call = build_create_receipt_call(&args(), Some("0x0000000000000000000000000000000000000001".to_string()))
            .expect("call payload");

        assert_eq!(call.contract_name, "QuoteReceiptRegistry");
        assert_eq!(call.function_signature, CREATE_RECEIPT_SIGNATURE);
        assert_eq!(call.method_id, "0x3a09bc9e");
        assert_eq!(call.chain_id, Some(421_614));
        assert_eq!(call.value_wei, "0");
        assert!(call.calldata.starts_with("0x3a09bc9e11111111"));
        assert!(call.calldata.contains("5257415f4541524e494e47535f474150"));
        assert!(call.calldata.contains("417262697472756d205365706f6c6961"));
    }

    #[test]
    fn rejects_invalid_bytes32() {
        let mut args = args();
        args.quote_id = "0x1234".to_string();
        let err = build_create_receipt_call(&args, None).expect_err("invalid bytes32");
        assert!(err.contains("bytes32 value must be 32 bytes"));
    }
}
