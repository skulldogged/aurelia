import XCTest
@testable import AureliaMac

final class TimeFormatterTests: XCTestCase {
    func testFormatDurationClampsToZero() {
        XCTAssertEqual(TimeFormatter.formatDuration(-1_000), "0:00")
    }

    func testFormatDurationFormatsMinutesAndSeconds() {
        XCTAssertEqual(TimeFormatter.formatDuration(185_000), "3:05")
    }

    func testFormatRelativeTimeReturnsNeverForMissingInput() {
        XCTAssertEqual(TimeFormatter.formatRelativeTime(nil), "Never")
        XCTAssertEqual(TimeFormatter.formatRelativeTime(""), "Never")
    }

    func testFormatRelativeTimeReturnsUnknownForInvalidInput() {
        XCTAssertEqual(TimeFormatter.formatRelativeTime("not-a-date"), "Unknown")
    }
}
