import Foundation
import AureliaCore

enum LyricsParser {
    static func fromParsed(_ parsed: ParsedLyrics) -> Lyrics {
        let synced = parsed.synced.isEmpty
            ? nil
            : parsed.synced.map {
                SyncedLine(
                    time: Double($0.timeMs) / 1000.0,
                    line: $0.line,
                    words: $0.words?.map {
                        SyncedWord(
                            time: Double($0.timeMs) / 1000.0,
                            word: $0.word
                        )
                    }
                )
            }

        let plain = parsed.plain.isEmpty ? nil : parsed.plain.joined(separator: "\n")

        return Lyrics(plain: plain, synced: synced, areFromRemote: parsed.areFromRemote)
    }
}
