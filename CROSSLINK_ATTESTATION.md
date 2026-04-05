# Crosslink Validator Attestation

Status: Draft  
Date: 2026-04-01  
Depends on: Crosslink PoS hybrid protocol activation

## Overview

Crosslink introduces proof-of-stake to Zcash as a hybrid consensus mechanism. Validators stake ZEC and participate in block finalization. This creates a new class of operations that need attestation: deposits, withdrawals, rewards, slashing, delegation, and validator lifecycle.

ZAP1 types 0x0A-0x0C (STAKING_DEPOSIT, STAKING_WITHDRAW, STAKING_REWARD) were reserved from v2.0.0 and are implemented in the API with preliminary hash constructions. This document extends the staking attestation model for the full Crosslink validator lifecycle.

## Existing Staking Types (0x0A-0x0C)

These are already implemented. Hash constructions will be finalized when Crosslink specifies the staking transaction format.

| Type | Name | Current payload |
|------|------|-----------------|
| `0x0A` | `STAKING_DEPOSIT` | `hash(wallet_hash \|\| amount_zat_be \|\| validator_id)` |
| `0x0B` | `STAKING_WITHDRAW` | `hash(wallet_hash \|\| amount_zat_be \|\| validator_id)` |
| `0x0C` | `STAKING_REWARD` | `hash(wallet_hash \|\| epoch_be \|\| reward_zat_be)` |

## Proposed Validator Operations (0x30-0x3F)

Extended event types for the full validator lifecycle. These activate when Crosslink launches.

| Type | Name | Payload | Trigger |
|------|------|---------|---------|
| `0x30` | `VALIDATOR_REGISTER` | `hash(validator_id \|\| pubkey_hash \|\| stake_amount)` | Validator joins the active set |
| `0x31` | `VALIDATOR_EXIT` | `hash(validator_id \|\| exit_epoch \|\| reason_code)` | Validator exits the active set |
| `0x32` | `VALIDATOR_SLASH` | `hash(validator_id \|\| slash_amount \|\| evidence_hash)` | Validator slashed for misbehavior |
| `0x33` | `VALIDATOR_ATTEST` | `hash(validator_id \|\| epoch \|\| checkpoint_hash)` | Validator attests a finality checkpoint |
| `0x34` | `DELEGATION_SET` | `hash(delegator_hash \|\| validator_id \|\| amount)` | Delegation to a validator |
| `0x35` | `DELEGATION_UNSET` | `hash(delegator_hash \|\| validator_id \|\| amount)` | Delegation removed |
| `0x36` | `EPOCH_SUMMARY` | `hash(epoch \|\| active_validators \|\| total_stake \|\| rewards_paid)` | Per-epoch rollup |
| `0x37` | `VALIDATOR_UPTIME` | `hash(validator_id \|\| epoch \|\| attestation_count \|\| expected_count)` | Uptime report for a validator |

Types `0x38`-`0x3F` reserved for future Crosslink operations.

## Hash Constructions

```text
VALIDATOR_REGISTER = BLAKE2b_32(0x30 || len(validator_id) || validator_id || pubkey_hash || stake_amount_be)
VALIDATOR_EXIT     = BLAKE2b_32(0x31 || len(validator_id) || validator_id || exit_epoch_be || reason_code_be)
VALIDATOR_SLASH    = BLAKE2b_32(0x32 || len(validator_id) || validator_id || slash_amount_be || evidence_hash)
VALIDATOR_ATTEST   = BLAKE2b_32(0x33 || len(validator_id) || validator_id || epoch_be || checkpoint_hash)
DELEGATION_SET     = BLAKE2b_32(0x34 || len(delegator_hash) || delegator_hash || len(validator_id) || validator_id || amount_be)
DELEGATION_UNSET   = BLAKE2b_32(0x35 || len(delegator_hash) || delegator_hash || len(validator_id) || validator_id || amount_be)
EPOCH_SUMMARY      = BLAKE2b_32(0x36 || epoch_be || active_validators_be || total_stake_be || rewards_paid_be)
VALIDATOR_UPTIME   = BLAKE2b_32(0x37 || len(validator_id) || validator_id || epoch_be || attestation_count_be || expected_count_be)
```

All hashes use BLAKE2b-256 with `NordicShield_` personalization. Fixed-size fields (32-byte hashes, 8-byte amounts, 4-byte epochs/counts) are not length-prefixed. Variable-length identifiers are length-prefixed with 2-byte big-endian length.

## Use Cases

### Validator accountability

A validator registers, stakes, attests checkpoints, and earns rewards. Every operation is committed to the Merkle tree. A delegator verifies the validator's full history from proof paths without trusting the validator's self-reported stats.

### Slashing evidence

When a validator is slashed, the VALIDATOR_SLASH event commits the evidence hash. The slashing is provable from the Merkle tree even if the validator's node goes offline.

### Delegation audit

A delegator proves their delegation history - when they delegated, how much, to which validator, and when they undelegated. This is useful for tax reporting, portfolio proof, and cross-chain credential systems.

### Cross-chain staking proof

Using the Solidity verifier, an Ethereum smart contract can verify that a Zcash validator is active, has a certain stake amount, or has maintained uptime - enabling cross-chain staking derivatives or insurance products.

### Epoch reporting

EPOCH_SUMMARY provides a per-epoch rollup: active validator count, total stake, rewards paid. This is the validator set's equivalent of a financial statement, anchored on-chain and verifiable from the Merkle tree.

## Relationship to Crosslink Protocol

ZAP1 validator attestations are application-layer records, not consensus operations. They do not modify Crosslink staking mechanics, finality rules, or reward distribution. They sit above the consensus protocol and provide a verifiable audit surface.

Crosslink handles: staking, finality, slashing, rewards at the consensus layer.  
ZAP1 handles: attestation, accountability, delegation audit, cross-chain proof at the application layer.

## Coinbase Integration

Validators who also mine (or operate pool infrastructure) can embed attestations in coinbase memos at zero marginal cost (see COINBASE_ATTESTATION.md). A validator mining a block includes both their block attestation (POOL_BLOCK, 0x20) and their finality attestation (VALIDATOR_ATTEST, 0x33) in the same coinbase memo sequence.

## Activation

These types activate when:
1. Crosslink protocol is specified and the validator registration/exit flow is finalized
2. The staking transaction format is documented
3. Hash constructions are validated against Crosslink's on-chain data structures

Until then, types 0x30-0x3F are reserved. Types 0x0A-0x0C remain implemented with preliminary constructions.
