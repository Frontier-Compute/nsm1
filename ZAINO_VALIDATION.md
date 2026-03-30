# Zaino gRPC Validation

Date: March 30, 2026
Status: Validated on mainnet

## Infrastructure

- Zaino 0.2.0 (ZingoLabs ZainoD)
- gRPC on 127.0.0.1:8137
- ZainoDB: 96GB at /mnt/zebra/zaino-db
- Connected to Zebra 4.3.0 at 127.0.0.1:8232
- Chain tip: 3,289,945 (synced)

## Endpoints Tested

| Method | Result |
|--------|--------|
| GetLightdInfo | Version 0.2.0, chain main, block 3,289,945 |
| GetLatestBlock | Height 3,289,945, hash returned |
| GetBlock(3286631) | First anchor block, compact tx data present |
| GetBlockRange(3286631-3286633) | 3 blocks streamed |
| GetTransaction(9eb952bb...) | 5th anchor tx, height 3,289,870, full raw data |
| GetLatestTreeState | Sapling + Orchard tree state at tip |

## Anchor Verification via Zaino

Our 5th anchor (txid 9eb952bb180f34bc4ecf4390c4e5c139ccccd9cc03036e1c467c4ceab1dd55f1) was retrieved via Zaino gRPC, confirming the dual-backend path works for NSM1 anchor verification.

## Dual Backend Summary

| Backend | Port | Protocol | Scanner Use |
|---------|------|----------|------------|
| Zebra RPC | 8232 | JSON-RPC | Current production scanner (polling getblock) |
| Zaino gRPC | 8137 | CompactTxStreamer | Compact block streaming (validated, integration target) |

The NodeBackend trait in nsm1/src/node.rs abstracts both paths. Switching from Zebra RPC to Zaino gRPC requires changing the backend config, not the scanner logic.
