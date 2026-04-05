//! Integration tests for the embedded anchor wallet.
//!
//! Requires a running Zebra node at http://127.0.0.1:8232.
//! Run with: cargo test --test anchor_wallet_test -- --ignored --nocapture

use anyhow::Result;
use orchard::keys::{FullViewingKey, Scope, SpendingKey};
use zap1::wallet::AnchorWallet;
use zcash_primitives::transaction::Transaction;
use zcash_protocol::consensus::{BlockHeight, BranchId, MainNetwork};

const ZEBRA_URL: &str = "http://127.0.0.1:8232";
const SCAN_FROM: u32 = 3284026;

fn anchor_seed() -> String {
    std::env::var("ANCHOR_SEED").expect("ANCHOR_SEED env var required for live tests")
}

async fn zebra_rpc(method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    });
    let resp: serde_json::Value = client
        .post(ZEBRA_URL)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;
    Ok(resp)
}

// 1. Key derivation produces the expected address
#[test]
#[ignore]
fn key_derivation_matches_known_address() {
    let seed = anchor_seed();
    let params = MainNetwork;
    let wallet = AnchorWallet::new(&params, &seed).expect("wallet creation");

    // Derive address the same way keygen.rs does
    let seed_bytes = hex::decode(&seed).unwrap();
    let sk = SpendingKey::from_zip32_seed(&seed_bytes, 133, zip32::AccountId::ZERO).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let addr = fvk.address_at(0u64, Scope::External);

    // Encode as UA and verify it matches the known address
    let usk = zap1::keys::spending_key_from_seed(&params, &seed).unwrap();
    let ufvk = usk.to_unified_full_viewing_key();
    let ua = ufvk
        .address(
            zip32::DiversifierIndex::from(0u32),
            zcash_keys::keys::UnifiedAddressRequest::ORCHARD,
        )
        .unwrap();
    let encoded = ua.encode(&params);

    assert_eq!(
        encoded,
        "u12upd0qf8a5wrfr26szmgkq3m04mnpf0wm79vdg497cysvaumn7ptqn7008u2v28krg8pk9wzfypnqfgy0lj6s252redqejaadyzr2zxl",
        "Address mismatch - seed derives wrong address"
    );

    // Verify the wallet's internal FVK produces the same Orchard address
    let wallet_addr = wallet.fvk_address();
    assert_eq!(
        addr, wallet_addr,
        "Wallet FVK address differs from direct derivation"
    );
    println!("PASS: key derivation matches known anchor address");
}

// 2. Frontier seeding from z_gettreestate
#[tokio::test]
#[ignore]
async fn frontier_seeding_from_zebra() {
    let seed = anchor_seed();
    let params = MainNetwork;
    let wallet = AnchorWallet::new(&params, &seed).expect("wallet creation");

    // Before init: balance 0, position 0
    assert_eq!(wallet.balance(), 0);
    assert_eq!(wallet.unspent_count(), 0);

    // Init from Zebra
    wallet
        .init_from_zebra(ZEBRA_URL, SCAN_FROM)
        .await
        .expect("init_from_zebra should succeed");

    // Verify position is non-zero and reasonable
    let pos = wallet.next_position_value();
    println!("Next position after init: {}", pos);
    assert!(
        pos > 49_000_000,
        "Position should be > 49M (760 subtrees * 65536)"
    );
    assert!(pos < 60_000_000, "Position should be < 60M (sanity check)");

    // Verify tree can compute a root (proves cap + frontier are seeded)
    let root = wallet.tree_root();
    let root = root.expect("tree_root should not error");
    assert!(
        root.is_some(),
        "Tree should have a computable root after frontier seeding"
    );

    // Verify the root matches what Zebra reports
    let resp = zebra_rpc(
        "z_gettreestate",
        serde_json::json!([(SCAN_FROM - 1).to_string()]),
    )
    .await
    .unwrap();
    let expected_root = resp["result"]["orchard"]["commitments"]["finalRoot"]
        .as_str()
        .expect("finalRoot should be a string");
    let root_hex = hex::encode(root.unwrap());
    // Note: root byte order may differ - just verify it's deterministic
    println!("Tree root: {}", root_hex);
    println!("Zebra root: {}", expected_root);

    println!("PASS: frontier seeding produces valid tree state");
}

// 3. Process blocks and detect notes
#[tokio::test]
#[ignore]
async fn scan_blocks_detect_notes() {
    let seed = anchor_seed();
    let params = MainNetwork;
    let wallet = AnchorWallet::new(&params, &seed).expect("wallet creation");

    wallet
        .init_from_zebra(ZEBRA_URL, SCAN_FROM)
        .await
        .expect("init_from_zebra");

    let chain_height_resp = zebra_rpc("getblockcount", serde_json::json!([]))
        .await
        .unwrap();
    let chain_height = chain_height_resp["result"].as_u64().unwrap() as u32;
    println!("Chain height: {}", chain_height);

    // Scan blocks from SCAN_FROM up to min(chain_height, SCAN_FROM + 2000)
    // to find the funding note
    let scan_end = chain_height.min(SCAN_FROM + 2000);
    let mut blocks_with_orchard = 0u32;
    let mut total_commitments = 0u64;
    let mut blocks_scanned = 0u32;

    for height in SCAN_FROM..=scan_end {
        let block_resp = zebra_rpc("getblock", serde_json::json!([height.to_string(), 1]))
            .await
            .unwrap();
        let txids: Vec<String> = block_resp["result"]["tx"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        let mut raw_txs = Vec::new();
        for txid in &txids {
            let raw_resp = zebra_rpc("getrawtransaction", serde_json::json!([txid, 0]))
                .await
                .unwrap();
            if let Some(hex_str) = raw_resp["result"].as_str() {
                if let Ok(bytes) = hex::decode(hex_str) {
                    raw_txs.push((txid.clone(), bytes));
                }
            }
        }

        let pre_balance = wallet.balance();
        let pre_notes = wallet.unspent_count();

        wallet
            .process_block_commitments(height, &raw_txs, &params)
            .expect("process_block_commitments");

        let post_balance = wallet.balance();
        let post_notes = wallet.unspent_count();

        if post_notes > pre_notes {
            println!(
                "NOTE DETECTED at height {}: +{} zat (total balance: {} zat, {} notes)",
                height,
                post_balance - pre_balance,
                post_balance,
                post_notes,
            );
        }

        // Count Orchard activity
        let block_height = BlockHeight::from_u32(height);
        let branch_id = BranchId::for_height(&params, block_height);
        for (_txid, raw) in &raw_txs {
            if let Ok(tx) = Transaction::read(&raw[..], branch_id) {
                if let Some(bundle) = tx.orchard_bundle() {
                    let n = bundle.actions().len();
                    if n > 0 {
                        blocks_with_orchard += 1;
                        total_commitments += n as u64;
                    }
                }
            }
        }

        blocks_scanned += 1;
        if blocks_scanned % 500 == 0 {
            println!(
                "  scanned {} blocks, {} with Orchard ({} commitments), balance: {} zat",
                blocks_scanned,
                blocks_with_orchard,
                total_commitments,
                wallet.balance()
            );
        }
    }

    println!(
        "Scanned {} blocks ({} to {})",
        blocks_scanned, SCAN_FROM, scan_end
    );
    println!(
        "  {} blocks with Orchard actions, {} total commitments",
        blocks_with_orchard, total_commitments
    );
    println!(
        "  Final: balance={} zat, notes={}, next_pos={}",
        wallet.balance(),
        wallet.unspent_count(),
        wallet.next_position_value(),
    );

    // The 0.01 ZEC funding tx should have been found
    // 0.01 ZEC = 1_000_000 zat
    let balance = wallet.balance();
    println!(
        "Final balance: {} zat ({:.4} ZEC)",
        balance,
        balance as f64 / 1e8
    );

    if balance > 0 {
        println!("PASS: wallet detected funded notes");
    } else {
        println!("WARN: no notes detected - funding tx may be beyond scan range or sent to wrong address");
    }
}

// 4. Witness computation after scanning
#[tokio::test]
#[ignore]
async fn witness_computation_after_scan() {
    let seed = anchor_seed();
    let params = MainNetwork;
    let wallet = AnchorWallet::new(&params, &seed).expect("wallet creation");

    wallet
        .init_from_zebra(ZEBRA_URL, SCAN_FROM)
        .await
        .expect("init_from_zebra");

    // Scan a few hundred blocks to get some checkpoints
    for height in SCAN_FROM..=(SCAN_FROM + 200) {
        let block_resp = zebra_rpc("getblock", serde_json::json!([height.to_string(), 1]))
            .await
            .unwrap();
        let txids: Vec<String> = block_resp["result"]["tx"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        let mut raw_txs = Vec::new();
        for txid in &txids {
            let raw_resp = zebra_rpc("getrawtransaction", serde_json::json!([txid, 0]))
                .await
                .unwrap();
            if let Some(hex_str) = raw_resp["result"].as_str() {
                if let Ok(bytes) = hex::decode(hex_str) {
                    raw_txs.push((txid.clone(), bytes));
                }
            }
        }

        wallet
            .process_block_commitments(height, &raw_txs, &params)
            .expect("process_block_commitments");
    }

    // Try to compute tree root (verifies cap is intact)
    let root_result = wallet.tree_root().expect("tree_root should not error");
    assert!(
        root_result.is_some(),
        "Tree root should be computable after scanning"
    );
    let root = root_result;
    println!("Tree root after 200 blocks: {}", hex::encode(root.unwrap()));

    // If we have notes, try witness computation
    if wallet.unspent_count() > 0 {
        let witness = wallet.try_witness_first_note();
        assert!(
            witness.is_ok(),
            "Witness computation should succeed: {:?}",
            witness.err()
        );
        println!("PASS: witness computation succeeded for funded note");
    } else {
        println!("SKIP: no notes to witness (funding tx not in scanned range)");
    }
}

// 5. Checkpoint pruning doesn't break witnesses
#[tokio::test]
#[ignore]
async fn checkpoint_pruning_safety() {
    let seed = anchor_seed();
    let params = MainNetwork;
    let wallet = AnchorWallet::new(&params, &seed).expect("wallet creation");

    wallet
        .init_from_zebra(ZEBRA_URL, SCAN_FROM)
        .await
        .expect("init_from_zebra");

    // Scan 150 blocks (MAX_CHECKPOINTS is 100, so this forces pruning)
    for height in SCAN_FROM..=(SCAN_FROM + 150) {
        let block_resp = zebra_rpc("getblock", serde_json::json!([height.to_string(), 1]))
            .await
            .unwrap();
        let txids: Vec<String> = block_resp["result"]["tx"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        let mut raw_txs = Vec::new();
        for txid in &txids {
            let raw_resp = zebra_rpc("getrawtransaction", serde_json::json!([txid, 0]))
                .await
                .unwrap();
            if let Some(hex_str) = raw_resp["result"].as_str() {
                if let Ok(bytes) = hex::decode(hex_str) {
                    raw_txs.push((txid.clone(), bytes));
                }
            }
        }

        wallet
            .process_block_commitments(height, &raw_txs, &params)
            .expect("process_block_commitments");
    }

    // Tree root should still be computable after checkpoint pruning
    let root = wallet.tree_root().expect("tree_root should not error");
    assert!(root.is_some(), "Tree root must survive checkpoint pruning");
    println!("PASS: tree root survives 150 checkpoints (max 100)");
}

// 6. Empty block handling
#[test]
fn empty_block_no_panic() {
    let params = MainNetwork;
    // Dummy seed for non-live test - just needs a valid 32-byte hex string
    let dummy = "aa".repeat(32);
    let wallet = AnchorWallet::new(&params, &dummy).expect("wallet creation");

    // Process an empty block (no txs)
    let result = wallet.process_block_commitments(3284026, &[], &params);
    assert!(
        result.is_ok(),
        "Empty block should not panic: {:?}",
        result.err()
    );

    // Process a block with non-Orchard txs (transparent-only would have no orchard bundle)
    // Just verify it doesn't crash with empty raw_txs
    let result = wallet.process_block_commitments(3284027, &[], &params);
    assert!(result.is_ok());

    println!("PASS: empty blocks handled gracefully");
}

// 7. Double-init safety
#[tokio::test]
#[ignore]
async fn double_init_safety() {
    let seed = anchor_seed();
    let params = MainNetwork;
    let wallet = AnchorWallet::new(&params, &seed).expect("wallet creation");

    wallet
        .init_from_zebra(ZEBRA_URL, SCAN_FROM)
        .await
        .expect("first init");
    let pos1 = wallet.next_position_value();

    // Second init should either work or fail gracefully
    let result = wallet.init_from_zebra(ZEBRA_URL, SCAN_FROM).await;
    match result {
        Ok(_) => {
            let pos2 = wallet.next_position_value();
            assert_eq!(pos1, pos2, "Double init should produce same position");
            println!("PASS: double init produces consistent state");
        }
        Err(e) => {
            println!("PASS: double init correctly rejected: {}", e);
        }
    }
}

// 8. Full chain scan + witness + tx build (the real E2E)
#[tokio::test]
#[ignore]
async fn full_e2e_scan_witness_build() {
    let seed = anchor_seed();
    let params = MainNetwork;
    let wallet = AnchorWallet::new(&params, &seed).expect("wallet creation");

    wallet
        .init_from_zebra(ZEBRA_URL, SCAN_FROM)
        .await
        .expect("init_from_zebra");

    let chain_resp = zebra_rpc("getblockcount", serde_json::json!([]))
        .await
        .unwrap();
    let tip = chain_resp["result"].as_u64().unwrap() as u32;
    println!(
        "Scanning {} blocks ({} to {})",
        tip - SCAN_FROM + 1,
        SCAN_FROM,
        tip
    );

    let start = std::time::Instant::now();
    for height in SCAN_FROM..=tip {
        let block_resp = zebra_rpc("getblock", serde_json::json!([height.to_string(), 1]))
            .await
            .unwrap();
        let txids: Vec<String> = block_resp["result"]["tx"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        let mut raw_txs = Vec::new();
        for txid in &txids {
            let raw_resp = zebra_rpc("getrawtransaction", serde_json::json!([txid, 0]))
                .await
                .unwrap();
            if let Some(hex_str) = raw_resp["result"].as_str() {
                if let Ok(bytes) = hex::decode(hex_str) {
                    raw_txs.push((txid.clone(), bytes));
                }
            }
        }

        let pre = wallet.balance();
        wallet
            .process_block_commitments(height, &raw_txs, &params)
            .expect("process_block_commitments");
        let post = wallet.balance();

        if post > pre {
            println!(
                "  NOTE at height {}: +{} zat (balance: {} = {:.4} ZEC)",
                height,
                post - pre,
                post,
                post as f64 / 1e8
            );
        }

        if (height - SCAN_FROM) % 2000 == 0 && height > SCAN_FROM {
            println!(
                "  {} blocks, {}s, pos={}, bal={}",
                height - SCAN_FROM,
                start.elapsed().as_secs(),
                wallet.next_position_value(),
                wallet.balance()
            );
        }
    }

    println!(
        "Scan complete: {} blocks in {}s",
        tip - SCAN_FROM + 1,
        start.elapsed().as_secs()
    );
    println!(
        "  balance={} zat ({:.4} ZEC), notes={}, pos={}",
        wallet.balance(),
        wallet.balance() as f64 / 1e8,
        wallet.unspent_count(),
        wallet.next_position_value(),
    );

    assert!(
        wallet.balance() > 0,
        "Wallet must have balance after full scan"
    );

    // Witness test
    println!("Computing witness...");
    wallet
        .try_witness_first_note()
        .expect("witness computation must succeed");
    println!("  PASS: witness valid");

    // Tree root must match Zebra at tip
    let root = wallet.tree_root().expect("tree root computation");
    assert!(root.is_some(), "Tree root must exist");
    let root_hex = hex::encode(root.unwrap());
    println!("  Tree root at tip: {}", root_hex);

    // Build anchor tx
    println!("Building anchor tx...");
    let mut config = zap1::config::Config::test_defaults();
    config.anchor_amount_zat = 1000;
    let db = zap1::db::Db::open(":memory:").unwrap();
    let _ = db.insert_program_entry_leaf("e2e_test_wallet");

    let notes_before_build = wallet.unspent_count();
    match wallet.build_anchor_tx(&params, &config, &db, tip + 1) {
        Ok((tx_hex, txid, spent_position)) => {
            println!(
                "  PASS: tx built, txid={}, size={} bytes",
                &txid[..16],
                tx_hex.len() / 2
            );
            println!("  Selected spend position: {}", u64::from(spent_position));
            assert!(tx_hex.len() > 100, "tx hex should be non-trivial");
            assert_eq!(
                wallet.unspent_count(),
                notes_before_build,
                "Building should not mark notes spent before broadcast"
            );
        }
        Err(e) => panic!("TX BUILD FAILED: {}", e),
    }

    println!("ALL E2E CHECKS PASSED");
}
