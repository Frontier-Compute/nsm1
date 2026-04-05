//! Shielded payout module.
//!
//! Sends ZEC from the pool wallet to miner shielded addresses.
//! Each payout is broadcast individually and attested as HOSTING_PAYMENT.

use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use zcash_keys::address::UnifiedAddress;
use zcash_keys::encoding::AddressCodec;
use zcash_protocol::consensus::Parameters;
use zcash_protocol::memo::MemoBytes;

use crate::config::Config;
use crate::db::Db;
use crate::memo::{MemoType, StructuredMemo};
use crate::wallet::AnchorWallet;

#[derive(Debug, Clone, Deserialize)]
pub struct PayoutRequest {
    pub address: String,
    pub amount_zat: u64,
    pub wallet_hash: String,
    pub serial_number: String,
    pub month: u32,
    pub year: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct PayoutResult {
    pub address: String,
    pub amount_zat: u64,
    pub txid: String,
    pub leaf_hash: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PayoutError {
    pub address: String,
    pub amount_zat: u64,
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status")]
pub enum PayoutOutcome {
    #[serde(rename = "ok")]
    Ok(PayoutResult),
    #[serde(rename = "error")]
    Err(PayoutError),
}

/// Parse a unified or Orchard address string into an orchard::Address.
/// Supports u1... (unified with Orchard receiver).
fn parse_orchard_recipient<P: Parameters>(params: &P, address: &str) -> Result<orchard::Address> {
    let ua = UnifiedAddress::decode(params, address)
        .map_err(|e| anyhow::anyhow!("Failed to decode address: {}", e))?;
    let orchard_addr: &orchard::Address = ua
        .orchard()
        .ok_or_else(|| anyhow::anyhow!("Address has no Orchard receiver: {}", address))?;
    Ok(*orchard_addr)
}

/// Get chain height from Zebra RPC.
async fn get_chain_height(zebra_url: &str) -> Result<u32> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getblockcount",
        "params": [],
    });

    let resp: serde_json::Value = client
        .post(zebra_url)
        .json(&body)
        .send()
        .await
        .context("getblockcount request failed")?
        .json()
        .await
        .context("getblockcount parse failed")?;

    resp["result"]
        .as_u64()
        .map(|h| h as u32)
        .ok_or_else(|| anyhow::anyhow!("Failed to get chain height"))
}

/// Broadcast a raw transaction via Zebra sendrawtransaction.
async fn broadcast_tx(zebra_url: &str, tx_hex: &str) -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "sendrawtransaction",
        "params": [tx_hex],
    });

    let resp: serde_json::Value = client
        .post(zebra_url)
        .json(&body)
        .send()
        .await
        .context("sendrawtransaction request failed")?
        .json()
        .await
        .context("sendrawtransaction parse failed")?;

    if let Some(err) = resp.get("error") {
        if !err.is_null() {
            anyhow::bail!("sendrawtransaction failed: {}", err);
        }
    }

    Ok(resp["result"].as_str().unwrap_or("unknown").to_string())
}

/// Build a HOSTING_PAYMENT memo for a payout transaction.
fn payout_memo(serial_number: &str, month: u32, year: u32) -> Result<MemoBytes> {
    let payload = crate::memo::hash_hosting_payment(serial_number, month, year);
    let memo = StructuredMemo {
        memo_type: MemoType::HostingPayment,
        payload,
    };
    MemoBytes::from_bytes(memo.encode().as_bytes())
        .map_err(|_| anyhow::anyhow!("Payout memo too long"))
}

/// Process a batch of payouts. Each payout is independent - one failure
/// does not stop the rest.
pub async fn process_payouts(
    pool_wallet: &AnchorWallet,
    config: &Arc<Config>,
    db: &Arc<Db>,
    payouts: Vec<PayoutRequest>,
) -> Vec<PayoutOutcome> {
    let mut results = Vec::with_capacity(payouts.len());

    let chain_height = match get_chain_height(&config.zebra_rpc_url).await {
        Ok(h) => h,
        Err(e) => {
            // All payouts fail if we can't get chain height
            for p in &payouts {
                results.push(PayoutOutcome::Err(PayoutError {
                    address: p.address.clone(),
                    amount_zat: p.amount_zat,
                    error: format!("Chain height unavailable: {}", e),
                }));
            }
            return results;
        }
    };

    for payout in &payouts {
        let outcome = process_single_payout(pool_wallet, config, db, payout, chain_height).await;
        results.push(outcome);
    }

    results
}

async fn process_single_payout(
    pool_wallet: &AnchorWallet,
    config: &Arc<Config>,
    db: &Arc<Db>,
    payout: &PayoutRequest,
    chain_height: u32,
) -> PayoutOutcome {
    let err = |msg: String| {
        PayoutOutcome::Err(PayoutError {
            address: payout.address.clone(),
            amount_zat: payout.amount_zat,
            error: msg,
        })
    };

    // Validate amount
    if payout.amount_zat == 0 {
        return err("Amount must be > 0".to_string());
    }

    // Parse recipient address
    let recipient = match parse_orchard_recipient(&config.network, &payout.address) {
        Ok(r) => r,
        Err(e) => return err(format!("Bad address: {}", e)),
    };

    // Build memo
    let memo = match payout_memo(&payout.serial_number, payout.month, payout.year) {
        Ok(m) => m,
        Err(e) => return err(format!("Memo build: {}", e)),
    };

    // Build transaction
    let (tx_hex, _txid, spent_position) = match pool_wallet.build_payout_tx(
        &config.network,
        recipient,
        payout.amount_zat,
        memo,
        chain_height + 1,
    ) {
        Ok(r) => r,
        Err(e) => return err(format!("Tx build: {}", e)),
    };

    // Broadcast
    let broadcast_txid = match broadcast_tx(&config.zebra_rpc_url, &tx_hex).await {
        Ok(t) => {
            if let Err(e) = pool_wallet.mark_spent_at_position(spent_position) {
                tracing::warn!("Failed to mark spent note: {}", e);
            }
            t
        }
        Err(e) => return err(format!("Broadcast: {}", e)),
    };

    // Attest as HOSTING_PAYMENT
    let leaf_hash = match db.insert_hosting_payment_leaf(
        &payout.wallet_hash,
        &payout.serial_number,
        payout.month,
        payout.year,
    ) {
        Ok((leaf, _root)) => leaf.leaf_hash,
        Err(e) => {
            tracing::warn!(
                "Payout broadcast OK (txid={}) but attestation failed: {}",
                broadcast_txid,
                e
            );
            // Still return success - tx was broadcast, attestation can be retried
            String::from("attestation_failed")
        }
    };

    tracing::info!(
        "Payout complete: {} zat to {} txid={} leaf={}",
        payout.amount_zat,
        &payout.address[..20.min(payout.address.len())],
        &broadcast_txid[..16.min(broadcast_txid.len())],
        &leaf_hash[..16.min(leaf_hash.len())],
    );

    PayoutOutcome::Ok(PayoutResult {
        address: payout.address.clone(),
        amount_zat: payout.amount_zat,
        txid: broadcast_txid,
        leaf_hash,
    })
}
