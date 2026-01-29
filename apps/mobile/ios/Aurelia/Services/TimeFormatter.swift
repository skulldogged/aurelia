import Foundation

enum TimeFormatter {
    /// Formats milliseconds into "m:ss" format.
    static func formatDuration(_ ms: Int64) -> String {
        let totalSeconds = max(0, ms / 1000)
        let minutes = totalSeconds / 60
        let seconds = totalSeconds % 60
        return "\(minutes):\(String(format: "%02d", seconds))"
    }

    /// Formats an ISO date string into a relative time string.
    static func formatRelativeTime(_ isoTime: String?) -> String {
        guard let isoTime, !isoTime.isEmpty else { return "Never" }

        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        guard let date = formatter.date(from: isoTime) ?? ISO8601DateFormatter().date(from: isoTime) else {
            return "Unknown"
        }

        let interval = Date.now.timeIntervalSince(date)
        let minutes = Int(interval / 60)
        let hours = Int(interval / 3600)
        let days = Int(interval / 86400)

        return switch true {
        case minutes < 1: "Just now"
        case minutes < 60: "\(minutes)m ago"
        case hours < 24: "\(hours)h ago"
        case days < 7: "\(days)d ago"
        default: date.formatted(date: .abbreviated, time: .omitted)
        }
    }
}
