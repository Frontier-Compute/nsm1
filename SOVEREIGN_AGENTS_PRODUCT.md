# Sovereign Agent Infrastructure

Private custody, verifiable actions, split-key security. For AI agents that hold real capital.

## The problem

Every agent wallet today is either:
- **Custodial** (Coinbase AgentKit) - a company controls your agent's money
- **Transparent** (Eliza, raw EVM wallets) - every trade, payment, and balance is public
- **Breakable** (hot wallets) - compromise the agent, drain the wallet

Agents with real capital need better.

## What we built

Three layers that work together:

### 1. Split-key custody (Ika dWallet)
Agent holds one key share. Ika network holds the other. Neither can sign alone. Policy enforced in Sui Move contracts before any signature happens.

One secp256k1 key signs for:
- Zcash transparent (DoubleSHA256)
- Bitcoin (DoubleSHA256)
- Ethereum / Base / Arbitrum / Hyperliquid (KECCAK256)

### 2. Shielded settlement (Zcash Orchard)
Agent's payments are invisible. Counterparties can't see balances, transaction history, or wallet connections. The agent proves it paid without revealing how much it holds.

### 3. Verifiable action history (ZAP1 attestation)
Every agent action committed to a Merkle tree anchored on Zcash mainnet. Proofs verifiable on 4 EVM chains without revealing:
- Transaction amounts
- Counterparty identities
- Strategy details
- Model weights

Selective disclosure: prove "I took action X" without revealing everything else.

## 10 agent event types

| Type | Name | What it proves |
|------|------|----------------|
| 0x40 | AGENT_REGISTER | Agent identity + model hash + initial policy |
| 0x41 | AGENT_POLICY | Policy update with rules commitment |
| 0x42 | AGENT_ACTION | Tool execution with input/output hashes |
| 0x43 | AGENT_PAYMENT | Payment sent/received (amount hidden) |
| 0x44 | AGENT_DECISION | Decision context + outcome |
| 0x45 | AGENT_CHECKPOINT | State snapshot for audit/recovery |
| 0x46 | AGENT_DELEGATE | Authority delegation to sub-agent |
| 0x47 | AGENT_REVOKE | Delegation revocation |
| 0x48 | AGENT_INFERENCE | Model inference with proof commitment |
| 0x49 | AGENT_AUDIT | Periodic audit root for compliance |

## Integration paths

### MCP Server (any Claude/GPT agent)
```bash
npm install @frontiercompute/zcash-mcp
```
18 tools: create wallet, sign via MPC, send shielded, attest actions, verify proofs, check compliance.

### OpenClaw Plugin (multi-channel agents)
```bash
npm install @frontiercompute/openclaw-zap1
```
8 automatic hooks attest every message, command, and session. 14 query/create tools.

### x402 Micropayments (agent-to-agent)
```bash
npm install @frontiercompute/x402-zec
```
HTTP 402 middleware. Agent hits API, gets payment request, pays in shielded ZEC, proof returned in header.

### Direct SDK
```typescript
import { createWallet, sign } from '@frontiercompute/zcash-ika';
import { attestAction } from '@frontiercompute/silo-zap1';

const wallet = await createWallet(config, 'zcash-transparent');
const sig = await sign(config, { messageHash: sighash, walletId: wallet.id, chain: 'zcash-transparent', encryptionSeed: wallet.encryptionSeed });
await attestAction(config, { actionType: 'PAYMENT', inputHash: txHash, outputHash: recipientHash });
```

## Verification

Any attestation verifiable at:
- Browser: frontiercompute.cash/verify.html
- EVM contract: Arbitrum, Base, Hyperliquid, Sepolia
- CLI: `zap1-verify` crate on crates.io
- API: pay.frontiercompute.io/verify/{leaf}/proof.json

## Who this is for

- **Trading agents** holding real capital on Hyperliquid/DeFi - prove performance without revealing strategy
- **Payment agents** settling between services - pay privately, prove you paid
- **Compliance agents** needing audit trails - selective disclosure to auditors only
- **Multi-agent systems** where agents need to trust each other's track records

## Live infrastructure

- ZAP1 API: pay.frontiercompute.io (Zcash mainnet, 3+ anchors)
- EVM verifier: Arbitrum, Base, Hyperliquid, Sepolia
- Ika MPC: secp256k1 DKG + signing (testnet, mainnet ready)
- MCP server: npm registry + MCP directory
- 00zeven: 3-agent swarm demo with live attestation
