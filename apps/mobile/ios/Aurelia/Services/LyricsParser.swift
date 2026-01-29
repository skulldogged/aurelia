import Foundation

/// Parses LRC-format lyrics into structured synced lyrics data.
enum LyricsParser {
    static func parse(_ lrcContent: String) -> Lyrics {
        guard !lrcContent.isEmpty else {
            return Lyrics(plain: nil, synced: nil, areFromRemote: true)
        }

        var syncedLines: [SyncedLine] = []
        var plainLines: [String] = []

        let lines = lrcContent.components(separatedBy: .newlines)
        let timestampPattern = /\[(\d{1,3}):(\d{2})(?:\.(\d{1,3}))?\]/

        for line in lines {
            let stripped = line.trimmingCharacters(in: .whitespaces)
            guard !stripped.isEmpty else { continue }

            // Skip metadata tags like [ar:Artist]
            if stripped.hasPrefix("[") && !stripped.contains(where: \.isNumber) {
                continue
            }

            if let match = stripped.firstMatch(of: timestampPattern) {
                let minutes = Double(match.1) ?? 0
                let seconds = Double(match.2) ?? 0
                let centiseconds = Double(match.3 ?? Substring("0")) ?? 0
                let divisor = (match.3?.count ?? 1) == 3 ? 1000.0 : 100.0
                let time = minutes * 60.0 + seconds + centiseconds / divisor

                // Extract text after the timestamp
                let text = stripped.replacing(timestampPattern, with: { _ in "" })
                    .trimmingCharacters(in: .whitespaces)

                if !text.isEmpty {
                    syncedLines.append(SyncedLine(time: time, line: text, words: nil))
                    plainLines.append(text)
                }
            } else {
                plainLines.append(stripped)
            }
        }

        let plain = plainLines.isEmpty ? nil : plainLines.joined(separator: "\n")
        let synced = syncedLines.isEmpty ? nil : syncedLines.sorted { $0.time < $1.time }

        return Lyrics(plain: plain, synced: synced, areFromRemote: true)
    }
}
