//! Crosslink staking attestation loop.
//! Polls a Crosslink feature net RPC endpoint for staking events,
//! attests each one via ZAP1 STAKING_DEPOSIT/WITHDRAW/REWARD leaves,
//! and sends a Signal notification per event.
//!
//! Disabled at startup if CROSSLINK_RPC_URL is not set.
//! On connection failure, logs and retries next cycle - no crash.

use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use crate::db::Db;

const POLL_INTERVAL_SECS: u64 = 60;

#[derive(Debug, Clone)]
pub struct StakingState {
    pub total_staked_zat: u64,
    pub total_rewards_zat: u64,
    pub epoch_count: u32,
    pub last_seen_event_id: Option<u64>,
}

impl StakingState {
    fn new() -> Self {
        Self {
            total_staked_zat: 0,
            total_rewards_zat: 0,
            epoch_count: 0,
            last_seen_event_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum StakingEventKind {
    Deposit,
    Withdraw,
    Reward,
}

#[derive(Debug, Clone)]
pub struct StakingEvent {
    pub id: u64,
    pub kind: StakingEventKind,
    pub wallet_hash: String,
    pub amount_zat: u64,
    pub validator_id: String,
    pub epoch: u32,
}

pub async fn staking_loop(config: Arc<Config>, db: Arc<Db>) {
    let rpc_url = match &config.crosslink_rpc_url {
        Some(url) => url.clone(),
        None => {
            tracing::info!("Crosslink staking loop disabled (no CROSSLINK_RPC_URL)");
            return;
        }
    };

    let validator_id = config
        .crosslink_validator_id
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    tracing::info!(
        "Crosslink staking loop started - RPC: {} - validator: {} - polling every {}s",
        rpc_url,
        validator_id,
        POLL_INTERVAL_SECS
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap_or_default();

    let mut state = StakingState::new();

    loop {
        match poll_staking_events(&client, &rpc_url, &validator_id, &state).await {
            Ok(events) => {
                if !events.is_empty() {
                    tracing::info!("Crosslink: {} new staking event(s)", events.len());
                }
                for event in &events {
                    process_event(&config, &db, &mut state, event).await;
                }
                // Track highest seen id
                if let Some(last) = events.iter().map(|e| e.id).max() {
                    state.last_seen_event_id = Some(last);
                }
            }
            Err(e) => {
                tracing::warn!("Crosslink RPC poll failed: {} - retrying next cycle", e);
            }
        }

        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
}

async fn process_event(config: &Config, db: &Db, state: &mut StakingState, event: &StakingEvent) {
    let kind_str = match event.kind {
        StakingEventKind::Deposit => "STAKING_DEPOSIT",
        StakingEventKind::Withdraw => "STAKING_WITHDRAW",
        StakingEventKind::Reward => "STAKING_REWARD",
    };

    tracing::info!(
        "Crosslink event #{}: {} - wallet {}... - {} zat - epoch {}",
        event.id,
        kind_str,
        &event.wallet_hash[..event.wallet_hash.len().min(12)],
        event.amount_zat,
        event.epoch,
    );

    // Attest to ZAP1 merkle tree
    let attest_result = match event.kind {
        StakingEventKind::Deposit => db.insert_staking_deposit_leaf(
            &event.wallet_hash,
            event.amount_zat,
            &event.validator_id,
        ),
        StakingEventKind::Withdraw => db.insert_staking_withdraw_leaf(
            &event.wallet_hash,
            event.amount_zat,
            &event.validator_id,
        ),
        StakingEventKind::Reward => {
            db.insert_staking_reward_leaf(&event.wallet_hash, event.amount_zat, event.epoch)
        }
    };

    let leaf_hash = match attest_result {
        Ok((leaf, _root)) => {
            tracing::info!(
                "Crosslink: attested {} event #{} - leaf {}",
                kind_str,
                event.id,
                &leaf.leaf_hash[..12]
            );
            Some(leaf.leaf_hash)
        }
        Err(e) => {
            tracing::warn!(
                "Crosslink: failed to attest {} event #{}: {}",
                kind_str,
                event.id,
                e
            );
            None
        }
    };

    // Update running totals
    match event.kind {
        StakingEventKind::Deposit => {
            state.total_staked_zat = state.total_staked_zat.saturating_add(event.amount_zat);
        }
        StakingEventKind::Withdraw => {
            state.total_staked_zat = state.total_staked_zat.saturating_sub(event.amount_zat);
        }
        StakingEventKind::Reward => {
            state.total_rewards_zat = state.total_rewards_zat.saturating_add(event.amount_zat);
            if event.epoch > state.epoch_count {
                state.epoch_count = event.epoch;
            }
        }
    }

    // Signal notification
    send_staking_signal(config, event, kind_str, leaf_hash.as_deref(), state).await;
}

async fn send_staking_signal(
    config: &Config,
    event: &StakingEvent,
    kind_str: &str,
    leaf_hash: Option<&str>,
    state: &StakingState,
) {
    let Some(number) = &config.signal_number else {
        return;
    };
    let signal_url = config
        .signal_api_url
        .as_deref()
        .unwrap_or("http://127.0.0.1:8431");

    let leaf_line = match leaf_hash {
        Some(h) => format!("\nLeaf: {}...", &h[..h.len().min(16)]),
        None => "\nLeaf: attest failed".to_string(),
    };

    let msg = format!(
        "Crosslink staking\n\n\
         Event: {} #{}\n\
         Validator: {}\n\
         Amount: {:.4} ZEC\n\
         Epoch: {}{}\n\n\
         Totals: staked {:.4} ZEC / rewards {:.4} ZEC / epoch {}",
        kind_str,
        event.id,
        event.validator_id,
        event.amount_zat as f64 / 100_000_000.0,
        event.epoch,
        leaf_line,
        state.total_staked_zat as f64 / 100_000_000.0,
        state.total_rewards_zat as f64 / 100_000_000.0,
        state.epoch_count,
    );

    let payload = serde_json::json!({
        "message": msg,
        "number": number,
        "recipients": [number]
    });

    let url = format!("{}/v2/send", signal_url);
    let _ = reqwest::Client::new()
        .post(&url)
        .json(&payload)
        .send()
        .await;
}

// RPC polling - returns new events since last seen id.
// The feature net RPC spec is not yet finalized. This function parses the
// expected response shape and will gracefully return an empty list on any
// parse or connection failure. Swap in the real field names once the spec
// lands from the Signal group.
async fn poll_staking_events(
    client: &reqwest::Client,
    rpc_url: &str,
    validator_id: &str,
    state: &StakingState,
) -> anyhow::Result<Vec<StakingEvent>> {
    let since = state.last_seen_event_id.unwrap_or(0);

    // POST JSON-RPC to the Crosslink feature net
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "crosslink_getStakingEvents",
        "params": [{
            "validator_id": validator_id,
            "since_id": since
        }]
    });

    let resp = client.post(rpc_url).json(&body).send().await?;

    if !resp.status().is_success() {
        anyhow::bail!("RPC returned status {}", resp.status());
    }

    let data: serde_json::Value = resp.json().await?;

    // Check for JSON-RPC error object
    if let Some(err) = data.get("error") {
        anyhow::bail!("RPC error: {}", err);
    }

    let events_arr = match data["result"]["events"].as_array() {
        Some(arr) => arr,
        None => return Ok(vec![]),
    };

    let mut events = Vec::new();
    for raw in events_arr {
        let id = match raw["id"].as_u64() {
            Some(v) => v,
            None => continue,
        };
        let kind_str = raw["kind"].as_str().unwrap_or("").to_lowercase();
        let kind = match kind_str.as_str() {
            "deposit" => StakingEventKind::Deposit,
            "withdraw" => StakingEventKind::Withdraw,
            "reward" => StakingEventKind::Reward,
            other => {
                tracing::warn!(
                    "Crosslink: unknown staking event kind '{}' - skipping",
                    other
                );
                continue;
            }
        };

        let wallet_hash = raw["wallet_hash"].as_str().unwrap_or("unknown").to_string();
        let amount_zat = raw["amount_zat"].as_u64().unwrap_or(0);
        let vid = raw["validator_id"]
            .as_str()
            .unwrap_or(validator_id)
            .to_string();
        let epoch = raw["epoch"].as_u64().unwrap_or(0) as u32;

        events.push(StakingEvent {
            id,
            kind,
            wallet_hash,
            amount_zat,
            validator_id: vid,
            epoch,
        });
    }

    Ok(events)
}
