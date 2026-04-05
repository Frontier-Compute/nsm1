//! NEAR testnet anchor registration.
//!
//! After each successful Zcash anchor broadcast, register the Merkle root
//! on the NEAR testnet contract via the `near` CLI.

use anyhow::{Context, Result};

use crate::config::Config;

/// Call `register_anchor` on the NEAR contract. Fire-and-forget via `near` CLI.
/// Returns CLI stdout on success.
pub async fn register_on_near(
    config: &Config,
    root_hash: &str,
    zcash_height: Option<u32>,
) -> Result<String> {
    let contract = config
        .near_anchor_contract
        .as_deref()
        .context("NEAR_ANCHOR_CONTRACT not set")?;
    let account = config
        .near_account_id
        .as_deref()
        .context("NEAR_ACCOUNT_ID not set")?;

    // Convert 64-char hex root to byte array for JSON args
    let root_bytes: Vec<u8> = hex::decode(root_hash)
        .context("invalid root hash hex")?;
    let height = zcash_height.unwrap_or(0) as u64;

    let json_args = serde_json::json!({
        "root": root_bytes,
        "zcash_height": height,
    });

    let output = tokio::process::Command::new("near")
        .args([
            "contract",
            "call-function",
            "as-transaction",
            contract,
            "register_anchor",
            "json-args",
            &json_args.to_string(),
            "prepaid-gas",
            "10 Tgas",
            "attached-deposit",
            "0 NEAR",
            "sign-as",
            account,
            "network-config",
            "testnet",
            "sign-with-legacy-keychain",
            "send",
        ])
        .output()
        .await
        .context("failed to execute near CLI")?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        anyhow::bail!("near CLI failed: {}", if stderr.is_empty() { &stdout } else { &stderr });
    }

    Ok(stdout)
}
