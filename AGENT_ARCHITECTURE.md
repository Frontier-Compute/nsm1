# 00zeven: Zcash Agent Architecture

Status: Draft  
Date: 2026-04-02

## Problem

AI agents need three things: capability (do stuff), privacy (hide the money), and accountability (prove what happened). Current agent frameworks give capability. Zcash gives privacy. Nothing connects accountability to both.

An agent running through OpenClaw can search the web, write code, send messages, and execute shell commands. An agent with a Zcash wallet (via OWS) can hold and spend ZEC privately. But nobody can verify what the agent did without either trusting the operator or exposing the transaction graph.

## 00zeven agents

A 00zeven agent is an OpenClaw agent operating through Zcash's Orchard pool with ZAP1 attestation. Three properties:

1. **Capable** - full OpenClaw toolset (shell, browser, code, search, messaging, file ops)
2. **Private** - financial operations through Orchard shielded wallet
3. **Accountable** - every significant action attested via ZAP1 Merkle commitment

The name comes from having a license to operate autonomously - but with a verifiable trail.

## Stack

```
OpenClaw runtime
  |-- tools: shell, browser, search, files, messaging
  |-- channels: Telegram, Discord, Signal, Slack, Matrix, IRC, web
  |-- plugins: ZAP1 (attestation), wallet (OWS/PCZT), custom
  |
  v
ZAP1 attestation layer
  |-- AGENT_REGISTER: identity + model + policy committed
  |-- AGENT_POLICY: decision rules committed
  |-- AGENT_ACTION: every tool call attested
  |-- AGENT_PAYMENT: every transaction attested
  |-- AGENT_DECISION: every judgment attested
  |
  v
Zcash Orchard pool
  |-- shielded wallet (OWS/PCZT signing)
  |-- financial operations invisible
  |-- memo field carries ZAP1 attestations
  |
  v
Verification surfaces
  |-- ZAP1 API: proof bundles, lifecycle views
  |-- Solidity verifier: cross-chain proof checks
  |-- Zodl wallet: memo rendering
  |-- OpenClaw memory: agent knows its own history
```

## Agent Lifecycle

### Registration

The agent commits its identity to ZAP1 at startup:

```
AGENT_REGISTER(agent_id, pubkey_hash, model_hash, policy_hash)
```

This creates a leaf in the Merkle tree. The agent's identity is anchored to Zcash. Anyone with the agent_id can look up the registration and verify the model and policy hashes.

### Policy commitment

Before autonomous operation, the agent commits its rules:

```
AGENT_POLICY(agent_id, policy_version, rules_hash)
```

The rules hash covers: spending limits, approved counterparty hashes, allowed tool set, decision thresholds. The policy is committed before any action - the order is provable from leaf insertion sequence.

### Operation

Every significant action creates a ZAP1 event:

| Agent does | ZAP1 event | What gets committed |
|---|---|---|
| Calls a tool | AGENT_ACTION | tool name + input/output hashes |
| Sends ZEC | AGENT_PAYMENT | payment hash + counterparty hash |
| Makes a judgment call | AGENT_DECISION | context hash + decision hash |
| Delegates to sub-agent | AGENT_DELEGATE | delegate ID + scope hash |
| Hits a checkpoint | AGENT_CHECKPOINT | full state hash |

The agent's wallet activity stays shielded. The attestation layer proves what happened without revealing amounts or counterparties.

### Audit

Periodically or on demand:

```
AGENT_AUDIT(agent_id, audit_root, period)
```

The audit root is a Merkle root over the agent's internal audit tree for the period. A verifier can request the full audit tree and check every action against the committed root.

## OpenClaw Integration

### Plugin architecture

The `openclaw-zap1` plugin provides 10 tools to any OpenClaw agent. An agent configured with the ZAP1 plugin can:

1. Query its own attestation history
2. Verify proofs for other agents
3. Create attestation events for its own actions
4. Export proof bundles for external verification

### Memory integration

OpenClaw's memory system stores long-term agent context. ZAP1 attestation is the external, anchored complement:

- **Memory**: what the agent remembers (private, mutable, local)
- **Attestation**: what the agent provably did (public, immutable, on-chain)

An agent can reference its own ZAP1 attestation history in memory to maintain continuity across sessions. The attestation provides ground truth that memory alone cannot guarantee.

### Channel integration

A 00zeven agent operates across channels (Telegram, Discord, Signal, web). Actions taken on any channel get attested to the same Merkle tree. The attestation layer is channel-agnostic.

A user interacting with the agent on Telegram can verify its behavior by checking the ZAP1 proof, regardless of which channel the action originated from.

### Sub-agent coordination

OpenClaw supports thread ownership and multi-agent setups. When a 00zeven agent delegates to a sub-agent:

1. Parent commits AGENT_DELEGATE with scope hash
2. Sub-agent operates within its delegated scope
3. Sub-agent's actions are attested in the same or a linked Merkle tree
4. Parent can revoke delegation via AGENT_REVOKE

The delegation chain is provable from the Merkle tree.

## Privacy model

| Layer | What's visible | What's hidden |
|---|---|---|
| Zcash Orchard | Nothing (shielded) | Balances, amounts, counterparties |
| ZAP1 attestation | Event types, hashed identifiers, proof paths | Raw field values, amounts, PII |
| Selective disclosure | Whatever the agent's principal chooses to reveal | Everything else |

The default is maximum privacy. The agent's financial activity is shielded. The attestation layer proves behavior without revealing specifics. Selective disclosure lets the principal open specific events to specific verifiers.

## Cross-chain agent proofs

A DeFi protocol on Ethereum needs to know if an agent is trustworthy. The agent presents ZAP1 proofs to the Solidity verifier:

1. AGENT_REGISTER proof: the agent has a committed identity
2. AGENT_POLICY proof: the agent follows a known policy
3. N AGENT_ACTION proofs: the agent has a track record

The Ethereum protocol verifies these proofs on-chain. The agent's Zcash transaction graph stays shielded.

## Operator model

A 00zeven agent is deployed and controlled by a human operator. The operator:

- Chooses which AI model to use
- Defines the agent's policy (spending limits, tool access, scope)
- Holds the API key that authorizes attestation writes
- Reviews and approves the agent's actions where required
- Owns the attestation history and decides what to disclose

The agent is a tool the operator runs, not an autonomous entity. ZAP1 attestation proves what the tool did under the operator's direction. The operator is accountable. The protocol provides the evidence.

AI-assisted development tools (code generators, copilots, agent frameworks) are instruments used by the operator to build and operate the system. The operator authors the protocol, the specs, and the deployments. The tools accelerate execution. This distinction is foundational: attestation proves what happened, not who or what executed it. The operator signs the policy. The proof trail is the operator's record.

## Implementation path

| Component | Status | Where |
|---|---|---|
| OpenClaw plugin | Published (npm) | Frontier-Compute/openclaw-zap1 |
| Agent event types (0x40-0x42) | Deployed, mainnet | SOVEREIGN_AGENTS.md |
| ZAP1 protocol (0x01-0x0F) | Deployed, mainnet | Frontier-Compute/zap1 |
| Solidity verifier | Deployed, Sepolia | Frontier-Compute/zap1-verify-sol |
| Zodl rendering | PR submitted | zodl-inc/zodl-android#2173 |
| OWS wallet integration | In progress | External |
| Remaining agent types (0x43-0x49) | Spec published | SOVEREIGN_AGENTS.md |

The agent architecture is ready for operator deployment. The operator:
1. Installs the OpenClaw plugin
2. Configures agentId and apiKey
3. Defines a policy
4. Runs the agent under their supervision
