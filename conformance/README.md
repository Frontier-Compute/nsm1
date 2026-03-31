# ZAP1 Conformance Kit

Test your implementation against the ZAP1 protocol contract. If your code produces the same hashes and accepts the same proof bundles, it is conformant.

## Quick check

```bash
python3 conformance/check.py
```

## What it tests

1. **Hash vectors**: BLAKE2b-256 leaf hashes for all event types with known inputs
2. **Merkle tree**: root computation from known leaves using NordicShield_MRK personalization
3. **Proof verification**: accept valid proof bundles, reject invalid ones
4. **Memo wire format**: encode/decode ZAP1:{type}:{hash} strings
5. **Export packages**: parse and verify audit export packages

## Fixtures

| File | Purpose |
|---|---|
| `hash_vectors.json` | event type hash inputs and expected outputs |
| `tree_vectors.json` | Merkle tree construction from leaves to root |
| `valid_bundle.json` | proof bundle that must verify |
| `invalid_bundle.json` | proof bundle that must fail |
| `valid_export.json` | export package that must verify |
| `memo_vectors.json` | wire format encode/decode cases |

## For implementers

If you are building a ZAP1 implementation in any language:

1. Implement BLAKE2b-256 with 16-byte personalization
2. Use the hash vectors to verify your leaf construction
3. Use the tree vectors to verify your Merkle root computation
4. Use the bundle fixtures to verify your proof walker
5. Run `check.py` to confirm everything matches

## Adding vectors

New vectors should include mainnet anchor data where possible. The hash vectors from block 3,286,631 are the canonical reference.
