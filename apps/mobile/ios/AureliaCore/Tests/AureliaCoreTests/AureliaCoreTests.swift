@testable import AureliaCore
import XCTest

final class AureliaCoreTests: XCTestCase {
    func testPing() {
        XCTAssertEqual(ping(), "pong")
    }
}
