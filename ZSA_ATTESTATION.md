# ZSA Attestation Event Types

Status: Draft  
Date: 2026-04-01  
Depends on: Zcash Shielded Assets (ZIP 226, ZIP 227) protocol activation

## Overview

When Zcash Shielded Assets ship, asset issuers will need a way to attest properties of their assets on-chain without revealing holder identities or transaction details. ZAP1 provides this layer: structured attestation events committed to a Merkle tree and anchored to Zcash via shielded memos.

This document defines ZAP1 event types for the ZSA lifecycle, extending the registry into the `0x10`-`0x1F` range. These types are reserved and will activate when the ZSA protocol is deployed on mainnet.

Grounding: all types below map directly to ZIP 226/227 consensus objects. `AssetId = (issuer, assetDescHash)` per ZIP 227. Issuance is transparent and issuer-authorized. Burn is the provable supply-reduction mechanism (ZIP 226). Key rotation is not supported - compromise handling is "finalize old issuer, move to new issuer" (ZIP 227).

## Proposed Event Types

### Protocol-core (maps to ZIP 226/227 consensus concepts)

| Type | Name | Payload focus | ZIP reference |
|------|------|---------------|---------------|
| `0x10` | `ZSA_ISSUER_BIND` | `H(issuer \|\| issuer_meta_root)` | ZIP 227 issuer registration |
| `0x11` | `ZSA_ASSET_BIND` | `H(issuer \|\| assetDescHash)` | ZIP 227 AssetId derivation |
| `0x12` | `ZSA_REFERENCE_NOTE` | `H(issuer \|\| assetDescHash \|\| txid \|\| output_index)` | ZIP 227 reference note |
| `0x13` | `ZSA_ISSUE` | `H(issuer \|\| assetDescHash \|\| amount_be \|\| txid \|\| issue_action_index)` | ZIP 227 issuance action |
| `0x14` | `ZSA_FINALIZE` | `H(issuer \|\| assetDescHash \|\| txid)` | ZIP 227 finalization |
| `0x15` | `ZSA_ZERO_ISSUE_REVOKE` | `H(issuer \|\| assetDescHash \|\| txid \|\| reason_code)` | ZIP 227 zero-value revocation |
| `0x16` | `ZSA_BURN` | `H(asset_base \|\| amount_be \|\| txid)` | ZIP 226 burn mechanism |
| `0x17` | `ZSA_SUPPLY_SNAPSHOT` | `H(issuer \|\| assetDescHash \|\| supply_be \|\| final_flag)` | ZIP 227 global state |

### Bridge and operations (issuer-side attestations implied by ZIP 227)

| Type | Name | Payload focus | Use case |
|------|------|---------------|----------|
| `0x18` | `ZSA_RESERVE_LOCK` | `H(issuer \|\| assetDescHash \|\| ext_chain_id \|\| ext_txid_hash \|\| amount_be)` | Cross-chain reserve lock |
| `0x19` | `ZSA_RESERVE_RELEASE` | `H(issuer \|\| assetDescHash \|\| ext_chain_id \|\| ext_txid_hash \|\| amount_be)` | Cross-chain reserve release |
| `0x1A` | `ZSA_REDEEM_REQUEST` | `H(issuer \|\| assetDescHash \|\| burn_txid \|\| amount_be \|\| destination_hash)` | Burn-and-redeem request |
| `0x1B` | `ZSA_REDEEM_COMPLETE` | `H(issuer \|\| assetDescHash \|\| burn_txid \|\| settlement_txid_hash)` | Redemption settlement |
| `0x1C` | `ZSA_COMPROMISE_NOTICE` | `H(issuer \|\| incident_id_hash \|\| timestamp_be)` | Issuer key compromise |
| `0x1D` | `ZSA_SUCCESSOR_ISSUER` | `H(old_issuer \|\| new_issuer \|\| assetDescHash)` | Key rotation via new issuer |
| `0x1E` | `ZSA_METADATA_BIND` | `H(issuer \|\| assetDescHash \|\| metadata_root)` | Metadata commitment |
| `0x1F` | `ZSA_AUDIT_ROOT` | `H(issuer \|\| audit_root \|\| period_be)` | Periodic audit attestation |

## Hash Constructions

All hashes use BLAKE2b-256 with `NordicShield_` personalization, consistent with the base profile. All variable-length fields are length-prefixed with 2-byte big-endian length. Integer fields use big-endian encoding (8 bytes for amounts, 4 bytes for indices and reason codes).

```text
ZSA_ISSUER_BIND       = BLAKE2b_32(0x10 || len(issuer) || issuer || issuer_meta_root)
ZSA_ASSET_BIND        = BLAKE2b_32(0x11 || len(issuer) || issuer || assetDescHash)
ZSA_REFERENCE_NOTE    = BLAKE2b_32(0x12 || len(issuer) || issuer || assetDescHash || txid || output_index_be)
ZSA_ISSUE             = BLAKE2b_32(0x13 || len(issuer) || issuer || assetDescHash || amount_be || txid || issue_action_index_be)
ZSA_FINALIZE          = BLAKE2b_32(0x14 || len(issuer) || issuer || assetDescHash || txid)
ZSA_ZERO_ISSUE_REVOKE = BLAKE2b_32(0x15 || len(issuer) || issuer || assetDescHash || txid || reason_code_be)
ZSA_BURN              = BLAKE2b_32(0x16 || asset_base || amount_be || txid)
ZSA_SUPPLY_SNAPSHOT   = BLAKE2b_32(0x17 || len(issuer) || issuer || assetDescHash || supply_be || final_flag)
ZSA_RESERVE_LOCK      = BLAKE2b_32(0x18 || len(issuer) || issuer || assetDescHash || ext_chain_id_be || ext_txid_hash || amount_be)
ZSA_RESERVE_RELEASE   = BLAKE2b_32(0x19 || len(issuer) || issuer || assetDescHash || ext_chain_id_be || ext_txid_hash || amount_be)
ZSA_REDEEM_REQUEST    = BLAKE2b_32(0x1A || len(issuer) || issuer || assetDescHash || burn_txid || amount_be || destination_hash)
ZSA_REDEEM_COMPLETE   = BLAKE2b_32(0x1B || len(issuer) || issuer || assetDescHash || burn_txid || settlement_txid_hash)
ZSA_COMPROMISE_NOTICE = BLAKE2b_32(0x1C || len(issuer) || issuer || incident_id_hash || timestamp_be)
ZSA_SUCCESSOR_ISSUER  = BLAKE2b_32(0x1D || len(old_issuer) || old_issuer || len(new_issuer) || new_issuer || assetDescHash)
ZSA_METADATA_BIND     = BLAKE2b_32(0x1E || len(issuer) || issuer || assetDescHash || metadata_root)
ZSA_AUDIT_ROOT        = BLAKE2b_32(0x1F || len(issuer) || issuer || audit_root || period_be)
```

Fields:
- `issuer` - the issuance validating key encoding (per ZIP 227)
- `assetDescHash` - 32-byte hash of the asset description (per ZIP 227 AssetId derivation)
- `asset_base` - the asset base point for burn operations (per ZIP 226)
- `amount_be` - 8-byte big-endian amount in base units
- `txid` - 32-byte transaction identifier
- `output_index_be` / `issue_action_index_be` - 4-byte big-endian index
- `ext_chain_id_be` - 4-byte big-endian external chain identifier
- `ext_txid_hash` - 32-byte hash of the external chain transaction
- `destination_hash` - 32-byte hash of the redemption destination
- `settlement_txid_hash` - 32-byte hash of the settlement transaction
- `incident_id_hash` - 32-byte hash of the compromise incident record
- `issuer_meta_root` / `metadata_root` / `audit_root` - 32-byte Merkle roots of metadata or audit trees
- `reason_code_be` - 4-byte big-endian reason code
- `final_flag` - 1-byte (0x00 = not finalized, 0x01 = finalized)
- `period_be` - 4-byte big-endian audit period identifier

## Use Cases

### Supply Tracking

An institutional issuer creates a ZSA and commits every issuance and burn to ZAP1. The supply snapshot at any point is provable from the Merkle tree. Auditors verify the supply trail from the proof path without seeing individual holder balances.

### Bridge Operations

A cross-chain bridge locks collateral on an external chain and mints ZSA tokens on Zcash. Each lock and release is attested via `ZSA_RESERVE_LOCK` and `ZSA_RESERVE_RELEASE`. The Solidity verifier (`zap1-verify-sol`) can verify these attestations on the external chain, creating a two-way audit surface.

### Issuer Key Compromise

ZIP 227 does not support key rotation. If an issuer key is compromised, the response is to finalize the old asset and issue under a new key. `ZSA_COMPROMISE_NOTICE` and `ZSA_SUCCESSOR_ISSUER` create a permanent, verifiable record of this transition in the Merkle tree.

### Cross-Chain Proof

Using `zap1-verify-sol`, an EVM smart contract can verify that a ZSA was issued, that its supply matches a snapshot, or that a bridge reserve was locked - all without a custodian, a guardian set, or exposing the Zcash transaction graph.

## Relationship to ZSA Protocol

ZAP1 ZSA events are application-layer attestations, not consensus-layer operations. They do not modify ZSA issuance or transfer mechanics. They sit above the asset protocol and provide a verifiable audit surface.

The ZSA protocol handles: issuance, transfer, burn at the consensus layer.  
ZAP1 handles: attestation, supply proof, compliance, audit at the application layer.

## Activation

These event types will activate when:
1. ZSA protocol is deployed on Zcash mainnet
2. The `AssetId` and `issuer` key formats are finalized in ZIP 226/227
3. Hash constructions are validated against the ZSA issuance key derivation
4. Test vectors are published matching live ZSA issuance transactions

Until then, types `0x10`-`0x1F` are reserved in the ZAP1 registry. The API will reject events in this range.
