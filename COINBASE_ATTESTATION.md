# Coinbase Attestation

Status: Draft  
Date: 2026-04-01  
Depends on: Zebra shielded coinbase outputs (commit fa2268f), miner_memo support (commit 6fd711b)

## Overview

Zebra now supports shielded coinbase outputs and miner memos in coinbase transactions. This means a mining pool operator can embed structured data in the block reward itself, at zero marginal cost per block.

ZAP1 attestation memos can ride coinbase transactions. Instead of paying for a separate anchor transaction every 10 events or 24 hours, the pool anchors the current Merkle root inside the next block it mines. The mining produces the proof trail.

## How it works

### Current anchor flow (separate transaction)

```
Events -> Merkle tree -> root
                          |
                     anchor_root binary
                          |
                     shielded tx with ZAP1:09:{root} memo
                          |
                     costs 0.0001 ZEC per anchor
```

### Coinbase anchor flow (zero marginal cost)

```
Events -> Merkle tree -> root
                          |
                     pool operator sets miner_memo
                          |
                     next mined block carries ZAP1:09:{root} in coinbase
                          |
                     costs nothing beyond normal mining
```

The coinbase memo uses the same `ZAP1:09:{root_hex}` encoding. Verification is identical - the proof path resolves to a root that appears in a mined block's coinbase transaction.

## Proposed Event Types: Mining Pool Operations (0x20-0x2F)

These types are specific to mining pool attestation. They complement the existing lifecycle types (0x01-0x0F) and the ZSA types (0x10-0x1F).

| Type | Name | Payload | Trigger |
|------|------|---------|---------|
| `0x20` | `POOL_BLOCK` | `hash(pool_id \|\| block_height \|\| block_hash)` | Pool mines a block |
| `0x21` | `POOL_HASHRATE` | `hash(pool_id \|\| participant_id \|\| hashrate_khs \|\| period)` | Hashrate snapshot for a participant |
| `0x22` | `POOL_PAYOUT` | `hash(pool_id \|\| participant_id \|\| amount_zat \|\| period)` | Payout recorded for a participant |
| `0x23` | `POOL_SHARE` | `hash(pool_id \|\| worker_id \|\| share_hash \|\| difficulty)` | Valid share submitted |
| `0x24` | `POOL_UPTIME` | `hash(pool_id \|\| worker_id \|\| uptime_pct \|\| period)` | Uptime attestation for SLA |
| `0x25` | `POOL_CONFIG` | `hash(pool_id \|\| config_hash \|\| timestamp)` | Pool configuration change committed |
| `0x26` | `POOL_FEE` | `hash(pool_id \|\| fee_bps \|\| period)` | Fee rate attestation |
| `0x27` | `POOL_AUDIT` | `hash(pool_id \|\| audit_root \|\| period)` | Periodic pool audit root |

Types `0x28`-`0x2F` are reserved for future pool operations.

## Hash Constructions

All hashes use BLAKE2b-256 with `NordicShield_` personalization. Variable-length fields are length-prefixed with 2-byte big-endian length. Integer fields use big-endian encoding.

```text
POOL_BLOCK     = BLAKE2b_32(0x20 || len(pool_id) || pool_id || block_height_be || block_hash)
POOL_HASHRATE  = BLAKE2b_32(0x21 || len(pool_id) || pool_id || len(participant_id) || participant_id || hashrate_khs_be || period_be)
POOL_PAYOUT    = BLAKE2b_32(0x22 || len(pool_id) || pool_id || len(participant_id) || participant_id || amount_zat_be || period_be)
POOL_SHARE     = BLAKE2b_32(0x23 || len(pool_id) || pool_id || len(worker_id) || worker_id || share_hash || difficulty_be)
POOL_UPTIME    = BLAKE2b_32(0x24 || len(pool_id) || pool_id || len(worker_id) || worker_id || uptime_pct_be || period_be)
POOL_CONFIG    = BLAKE2b_32(0x25 || len(pool_id) || pool_id || config_hash || timestamp_be)
POOL_FEE       = BLAKE2b_32(0x26 || len(pool_id) || pool_id || fee_bps_be || period_be)
POOL_AUDIT     = BLAKE2b_32(0x27 || len(pool_id) || pool_id || audit_root || period_be)
```

Fields:
- `pool_id` - operator-assigned pool identifier
- `participant_id` - hashed participant identifier (no PII on-chain)
- `worker_id` - hashed worker/machine identifier
- `block_hash` - 32-byte hash of the mined block
- `share_hash` - 32-byte hash of the submitted share
- `config_hash` - 32-byte hash of the pool configuration blob
- `audit_root` - 32-byte Merkle root of the pool's internal audit tree
- `hashrate_khs_be` - 8-byte big-endian hashrate in KH/s
- `amount_zat_be` - 8-byte big-endian amount in zatoshis
- `difficulty_be` - 8-byte big-endian share difficulty
- `uptime_pct_be` - 4-byte big-endian uptime percentage (basis points, 10000 = 100%)
- `fee_bps_be` - 4-byte big-endian fee in basis points
- `period_be` - 4-byte big-endian period identifier (e.g., month*100+year or epoch number)
- `block_height_be` - 4-byte big-endian block height
- `timestamp_be` - 8-byte big-endian Unix timestamp

## Coinbase Transport

When carried in a coinbase memo, the ZAP1 attestation uses the same wire format as standard shielded memos:

```text
ZAP1:{type_hex}:{payload_hash_hex}
```

The difference is delivery: the memo is in the coinbase transaction's shielded output, not in a user-initiated shielded transaction.

### Detection

A scanner distinguishes coinbase attestations from regular attestations by checking whether the containing transaction is the first transaction in a block (coinbase position). The ZAP1 memo format is the same either way.

### Verification

Proof verification is unchanged. A coinbase-carried root and a separately-anchored root are cryptographically identical. The Merkle path resolves to the root, and the root appears in a mined transaction.

The only difference for the verifier: the anchor transaction is a coinbase tx, so it appears at height N with coinbase inputs. The `getrawtransaction` RPC returns the same structure.

## Integration with Zebra

Zebra's `miner_memo` config field (commit 6fd711b) accepts a byte string that gets embedded in the coinbase transaction's shielded output. A pool operator sets this to the current ZAP1 Merkle root before mining each block:

```toml
[mining]
miner_address = "u1..."
miner_memo = "ZAP1:09:{current_root_hex}"
```

For dynamic root updates, the pool's block template handler reads the current root from the ZAP1 API and injects it into `miner_memo` before each `getblocktemplate` call.

### Automated flow

```
1. Pool calls GET /anchor/status to get current root
2. Pool sets miner_memo = "ZAP1:09:{root}"
3. Pool mines blocks with that memo until root changes
4. When new leaves arrive and root changes, repeat from 1
```

This eliminates the anchor broadcast path entirely for pool operators. The anchor cost becomes zero.

## Use Cases

### Pool transparency

A mining pool attests hashrate distribution, payout accuracy, and uptime to participants. The attestations ride the blocks the pool mines. A participant verifies their allocation against the Merkle tree without trusting the pool's reporting.

### Institutional mining audit

An institutional miner (Foundry, etc.) needs auditable records of block production, fee structures, and payout schedules. POOL_BLOCK, POOL_FEE, and POOL_PAYOUT events create a verifiable audit trail embedded in the blocks themselves.

### Shielded revenue proof

With shielded coinbase, block rewards arrive privately. A miner proves they received rewards in a period by referencing the POOL_PAYOUT attestation and the corresponding Merkle proof. The proof confirms the event happened without revealing the exact amount.

### Cross-chain mining proof

Using the Solidity verifier (zap1-verify-sol), a DeFi protocol on Ethereum can verify that a Zcash miner produced blocks, maintained uptime, or received payouts - all from on-chain Merkle proofs without touching Zcash directly.

## Activation

Coinbase attestation depends on:
1. Zebra shielded coinbase outputs reaching stable release
2. The `miner_memo` field being exposed in production Zebra config
3. Pool software supporting dynamic memo injection via the block template API

The event types (0x20-0x2F) are reserved in the ZAP1 registry. The coinbase transport mode works with existing ZAP1 verification tooling - no changes to the verifier SDK, memo decoder, or Solidity contract.

## Registry Update

With this spec, the ZAP1 event type registry is:

| Range | Family | Count | Status |
|-------|--------|-------|--------|
| 0x01-0x09 | Lifecycle | 9 | Active |
| 0x0A-0x0C | Staking | 3 | Implemented, pending Crosslink |
| 0x0D-0x0F | Governance | 3 | Active |
| 0x10-0x1F | ZSA | 16 | Reserved (spec published) |
| 0x20-0x2F | Mining pool | 16 | Reserved (this document) |
| 0x30-0xFF | Unallocated | 208 | Future use |

Total defined: 47 event types across 5 families.
