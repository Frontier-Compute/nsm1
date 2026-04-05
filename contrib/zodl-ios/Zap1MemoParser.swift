import Foundation

/// Parses ZAP1 and legacy NSM1 attestation memos into structured data.
struct Zap1Attestation {
    let prefix: String
    let typeHex: String
    let event: String
    let hash: String

    var shortHash: String { String(hash.prefix(12)) + "..." }
    var isLegacy: Bool { prefix == "NSM1" }

    // Compiled once, reused across all parse calls
    private static let pattern = try! NSRegularExpression(
        pattern: #"^(ZAP1|NSM1):([0-9a-fA-F]{2}):([0-9a-fA-F]{64})$"#
    )

    private static let events: [String: String] = [
        "01": "PROGRAM_ENTRY", "02": "OWNERSHIP_ATTEST",
        "03": "CONTRACT_ANCHOR", "04": "DEPLOYMENT",
        "05": "HOSTING_PAYMENT", "06": "SHIELD_RENEWAL",
        "07": "TRANSFER", "08": "EXIT",
        "09": "MERKLE_ROOT", "0a": "STAKING_DEPOSIT",
        "0b": "STAKING_WITHDRAW", "0c": "STAKING_REWARD",
        "0d": "GOVERNANCE_PROPOSAL", "0e": "GOVERNANCE_VOTE",
        "0f": "GOVERNANCE_RESULT"
    ]

    static func parse(_ memo: String) -> Zap1Attestation? {
        let trimmed = memo.trimmingCharacters(in: .whitespacesAndNewlines)
        let range = NSRange(trimmed.startIndex..., in: trimmed)
        guard let match = pattern.firstMatch(in: trimmed, range: range),
              let prefixRange = Range(match.range(at: 1), in: trimmed),
              let typeRange = Range(match.range(at: 2), in: trimmed),
              let hashRange = Range(match.range(at: 3), in: trimmed)
        else { return nil }

        let prefix = String(trimmed[prefixRange])
        let typeHex = String(trimmed[typeRange]).lowercased()
        let hash = String(trimmed[hashRange])

        return Zap1Attestation(
            prefix: prefix,
            typeHex: typeHex,
            event: events[typeHex] ?? "TYPE_0x\(typeHex)",
            hash: hash
        )
    }

    static func format(_ memo: String) -> String? {
        guard let attestation = parse(memo) else { return nil }
        return "ZAP1: \(attestation.event)  \(attestation.shortHash)"
    }
}
