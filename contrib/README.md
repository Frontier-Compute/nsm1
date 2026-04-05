# Wallet Integration Contributions

Ready-to-use ZAP1 memo parsers for wallet integration. Drop these into your project to render ZAP1 attestation memos as typed event cards instead of raw protocol strings.

## Contents

### zodl-android/

- `Zap1MemoFormatter.kt` - Kotlin parser and formatter (0 dependencies)
- `Zap1MemoFormatterTest.kt` - Unit tests

Integration point in Zodl Android: `TransactionDetailVM.kt` lines 273-275 and 315-317.

Issue: [zodl-inc/zodl-android#2172](https://github.com/zodl-inc/zodl-android/issues/2172)

PR: [zodl-inc/zodl-android#2173](https://github.com/zodl-inc/zodl-android/pull/2173)

### zodl-ios/

- `Zap1MemoParser.swift` - Swift parser and formatter (0 dependencies)
- `Zap1MemoParserTests.swift` - Unit tests (XCTest)
- `TransactionDetailsStore.patch` - unified diff for the store change

Integration point in Zodl iOS: `TransactionDetailsStore.swift` line 434 in the `.memosLoaded` handler.

Issue: [zodl-inc/zodl-ios#1670](https://github.com/zodl-inc/zodl-ios/issues/1670)

## How it works

Both parsers detect the `ZAP1:{type}:{hash}` and `NSM1:{type}:{hash}` memo format. If a memo matches, the parser returns a structured object with:
- Event name (PROGRAM_ENTRY, DEPLOYMENT, MERKLE_ROOT, etc.)
- Shortened payload hash
- Legacy flag for NSM1 prefix

Non-ZAP1 memos return null - existing behavior is preserved.

## Protocol

- [ONCHAIN_PROTOCOL.md](../ONCHAIN_PROTOCOL.md) - event type registry and hash constructions
- [WALLET_INTEGRATION.md](../WALLET_INTEGRATION.md) - full integration roadmap with file paths
