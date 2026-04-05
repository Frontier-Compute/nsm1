//! Quick scan of last 200 blocks to find funding note.
//! Run: cargo test --test quick_fund_scan -- --ignored --nocapture

use zap1::wallet::AnchorWallet;
use zcash_protocol::consensus::MainNetwork;

const ZEBRA_URL: &str = "http://127.0.0.1:8232";

fn anchor_seed() -> String {
    std::env::var("ANCHOR_SEED").expect("ANCHOR_SEED env var required for live tests")
}

async fn zebra_rpc(method: &str, params: serde_json::Value) -> serde_json::Value {
    let client = reqwest::Client::new();
    let body = serde_json::json!({"jsonrpc":"2.0","id":1,"method":method,"params":params});
    client
        .post(ZEBRA_URL)
        .json(&body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

#[tokio::test]
#[ignore]
async fn find_funding_note() {
    let params = MainNetwork;
    let tip = zebra_rpc("getblockcount", serde_json::json!([])).await["result"]
        .as_u64()
        .unwrap() as u32;
    let scan_start = tip - 300;

    // Init wallet with frontier at scan_start
    let seed = anchor_seed();
    let wallet = AnchorWallet::new(&params, &seed).unwrap();
    wallet.init_from_zebra(ZEBRA_URL, scan_start).await.unwrap();
    println!(
        "Init at {}, pos={}, scanning to {}",
        scan_start,
        wallet.next_position_value(),
        tip
    );

    for height in scan_start..=tip {
        let block = zebra_rpc("getblock", serde_json::json!([height.to_string(), 1])).await;
        let txids: Vec<String> = block["result"]["tx"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        let mut raw_txs = Vec::new();
        for txid in &txids {
            let r = zebra_rpc("getrawtransaction", serde_json::json!([txid, 0])).await;
            if let Some(h) = r["result"].as_str() {
                if let Ok(b) = hex::decode(h) {
                    raw_txs.push((txid.clone(), b));
                }
            }
        }

        let pre = wallet.balance();
        wallet
            .process_block_commitments(height, &raw_txs, &params)
            .unwrap();
        let post = wallet.balance();
        if post > pre {
            println!(
                "NOTE at height {}: +{} zat (balance: {} = {:.4} ZEC, {} notes)",
                height,
                post - pre,
                post,
                post as f64 / 1e8,
                wallet.unspent_count()
            );
        }
    }

    println!(
        "\nFinal: balance={} zat ({:.4} ZEC), notes={}, pos={}",
        wallet.balance(),
        wallet.balance() as f64 / 1e8,
        wallet.unspent_count(),
        wallet.next_position_value()
    );

    if wallet.balance() > 0 {
        println!("Testing witness...");
        wallet.try_witness_first_note().expect("witness must work");
        println!("WITNESS: OK");

        println!("Building anchor tx...");
        let mut config = zap1::config::Config::test_defaults();
        config.anchor_amount_zat = 1000;
        let db = zap1::db::Db::open(":memory:").unwrap();
        let _ = db.insert_program_entry_leaf("e2e_test");
        let (tx_hex, txid, pos) = wallet
            .build_anchor_tx(&params, &config, &db, tip + 1)
            .unwrap();
        println!(
            "TX BUILD: OK txid={} size={} bytes spend_pos={}",
            &txid[..16],
            tx_hex.len() / 2,
            u64::from(pos)
        );
        println!("ALL E2E PASSED");
    } else {
        println!("NO NOTES FOUND - funding tx not in last 300 blocks");
    }
}
