import XCTest

class Zap1MemoParserTests: XCTestCase {
    func testParseProgramEntry() {
        let att = Zap1Attestation.parse(
            "ZAP1:01:075b00df286038a7b3f6bb70054df61343e3481fba579591354a00214e9e019b"
        )

        XCTAssertNotNil(att)
        XCTAssertEqual(att?.prefix, "ZAP1")
        XCTAssertEqual(att?.typeHex, "01")
        XCTAssertEqual(att?.event, "PROGRAM_ENTRY")
        XCTAssertEqual(att?.shortHash, "075b00df2860...")
        XCTAssertEqual(att?.isLegacy, false)
    }

    func testParseLegacyNSM1() {
        let att = Zap1Attestation.parse(
            "NSM1:04:f265b9a06a61b2b8c6eeed7fc00c7aa686ad511053467815bf1f1037d460e1f1"
        )

        XCTAssertNotNil(att)
        XCTAssertEqual(att?.prefix, "NSM1")
        XCTAssertEqual(att?.event, "DEPLOYMENT")
        XCTAssertEqual(att?.isLegacy, true)
    }

    func testParseGovernanceUppercaseHex() {
        let att = Zap1Attestation.parse(
            "ZAP1:0D:A487C25F5867A9E3760C45AE7EED24D84E771568F1826A889CCD94B3C7C3A5B5"
        )

        XCTAssertNotNil(att)
        XCTAssertEqual(att?.typeHex, "0d")
        XCTAssertEqual(att?.event, "GOVERNANCE_PROPOSAL")
    }

    func testRejectsInvalidMemos() {
        XCTAssertNil(Zap1Attestation.parse("Hello world"))
        XCTAssertNil(Zap1Attestation.parse("ZAP1:xx:notahash"))
        XCTAssertNil(Zap1Attestation.parse("ZAP1:01:tooshort"))
        XCTAssertNil(Zap1Attestation.parse(""))
        XCTAssertNil(Zap1Attestation.parse(
            "ZAP2:01:075b00df286038a7b3f6bb70054df61343e3481fba579591354a00214e9e019b"
        ))
    }

    func testFormatReturnsReadableString() {
        let formatted = Zap1Attestation.format(
            "ZAP1:09:024e36515ea30efc15a0a7962dd8f677455938079430b9eab174f46a4328a07a"
        )

        XCTAssertEqual(formatted, "ZAP1: MERKLE_ROOT  024e36515ea3...")
    }

    func testFormatReturnsNilForNonZap1() {
        XCTAssertNil(Zap1Attestation.format("Just a regular memo"))
    }

    func testTrimsWhitespaceAndNewlines() {
        let att = Zap1Attestation.parse(
            "  ZAP1:01:075b00df286038a7b3f6bb70054df61343e3481fba579591354a00214e9e019b\n"
        )

        XCTAssertNotNil(att)
        XCTAssertEqual(att?.event, "PROGRAM_ENTRY")
    }
}
