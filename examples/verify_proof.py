#!/usr/bin/env python3
"""Verify a ZAP1 attestation proof against Zcash mainnet."""
import hashlib, json, urllib.request, sys

API = "https://pay.frontiercompute.io"
LEAF = sys.argv[1] if len(sys.argv) > 1 else "075b00df286038a7b3f6bb70054df61343e3481fba579591354a00214e9e019b"

# Fetch proof
proof = json.loads(urllib.request.urlopen(f"{API}/verify/{LEAF}/proof.json").read())
check = json.loads(urllib.request.urlopen(f"{API}/verify/{LEAF}/check").read())

print(f"Leaf: {LEAF[:24]}...")
root = proof.get('root', {})
root_hash = root.get('hash', str(root)) if isinstance(root, dict) else str(root)
anchor = proof.get('anchor', {})
txid = anchor.get('txid', 'pending')
print(f"Root: {root_hash[:24]}...")
print(f"Anchor: block {anchor.get('height', 'pending')}")
print(f"Valid: {check.get('valid', False)}")
print(f"Txid: {txid[:24]}...")
