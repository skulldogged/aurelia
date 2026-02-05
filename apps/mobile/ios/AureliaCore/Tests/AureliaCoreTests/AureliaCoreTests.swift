import XCTest
@testable import AureliaCore

final class AureliaCoreTests: XCTestCase {
    func testPing() {
        XCTAssertEqual(ping(), "pong")
    }
}
