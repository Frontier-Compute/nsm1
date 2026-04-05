# ZAP1 Evaluator Quickstart

This guide is the reviewer-facing entrypoint. For the full walkthrough, see [`QUICKSTART.md`](QUICKSTART.md).

## Fastest path

```bash
git clone https://github.com/Frontier-Compute/zap1.git
cd zap1
bash scripts/evaluate.sh
```

This runs the live validation path against the public stack and forwards to `scripts/check.sh`.

## What it proves

- the live API is reachable and reports `protocol: ZAP1`
- anchored roots and leaves exist on mainnet
- a live proof verifies
- memo decode returns `zap1` for a known attestation
- explorer and simulator are reachable
- published crates are live
- local Rust checks run when the toolchain is available

## Manual surfaces

- Live protocol info: `https://pay.frontiercompute.io/protocol/info`
- Live stats: `https://pay.frontiercompute.io/stats`
- Anchor history: `https://pay.frontiercompute.io/anchor/history`
- Proof page: `https://pay.frontiercompute.io/verify/075b00df286038a7b3f6bb70054df61343e3481fba579591354a00214e9e019b`
- Proof JSON: `https://pay.frontiercompute.io/verify/075b00df286038a7b3f6bb70054df61343e3481fba579591354a00214e9e019b/proof.json`
- Browser verifier: `https://frontiercompute.io/verify.html`

## Supporting docs

- Full walkthrough: [`QUICKSTART.md`](QUICKSTART.md)
- Evidence snapshot: [`EVIDENCE.md`](EVIDENCE.md)
- Protocol spec: [`ONCHAIN_PROTOCOL.md`](ONCHAIN_PROTOCOL.md)
- Test vectors: [`TEST_VECTORS.md`](TEST_VECTORS.md)
- Operator runbook: [`docs/OPERATOR_RUNBOOK.md`](docs/OPERATOR_RUNBOOK.md)
