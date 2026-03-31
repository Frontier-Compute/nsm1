# Changelog

## 3.0.0-draft (2026-03-31)

- Added type byte prefix to leaf hash construction (spec Section 3)
- Renumbered spec sections to close gap from removed sections
- Expanded test vectors: Merkle tree, memo encoding, conformance cross-check
- FROST 2-of-3 threshold signing design document
- Keygen binary for operator key generation
- Operator setup script (generates keys, .env, docker-compose)
- Admin anchor QR endpoint for manual Zodl anchoring
- Anchor badge SVG endpoint
- Event descriptions in /events response
- Zaino gRPC scanner backend (production)
- 29 API conformance checks (was 21)

## 2.2.0 (2026-03-28)

- Conformance kit: 14 protocol checks, 21 API schema checks
- OpenAPI 3.0 spec for read-only surfaces
- Reference clients (Python, TypeScript)
- Consumer contracts (wallet, explorer, indexer, operator)
- Versioning policy with stability guarantees
- ZIP 302 TVLV reference encoder/decoder
- Compatibility vectors between ZAP1 and legacy NSM1

## 2.1.0 (2026-03-27)

- zap1-verify v0.2.0 on crates.io (Rust + 83KB WASM)
- zap1-js v0.1.0 on npm
- zcash-memo-decode v0.1.0 on crates.io
- Browser verifier, attestation explorer, interactive simulator
- ZIP draft PR #1243

## 2.0.0 (2026-03-27)

- Renamed protocol from NSM1 to ZAP1
- 3 mainnet anchors, 12 leaves
- 9 event types deployed, 3 reserved for Crosslink staking
- BLAKE2b-256 leaf hashing with domain-separated personalization
- Merkle tree aggregation with NordicShield_MRK node personalization

## 1.0.0 (2026-03-27)

- Initial deployment on Zcash mainnet
- First anchor at block 3,286,631
