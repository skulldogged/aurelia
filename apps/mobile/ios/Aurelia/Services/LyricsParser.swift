import Foundation
import AureliaCore

enum LyricsParser {
    static func fromParsed(_ parsed: ParsedLyrics) -> Lyrics {
        let synced = parsed.synced.isEmpty
            ? nil
            : parsed.synced.map {
                SyncedLine(
                    time: Double($0.timeMs) / 1000.0,
                    endTime: $0.endTimeMs.map { Double($0) / 1000.0 },
                    line: $0.line,
                    words: $0.words?.map {
                        SyncedWord(
                            time: Double($0.timeMs) / 1000.0,
                            endTime: $0.endTimeMs.map { Double($0) / 1000.0 },
                            word: $0.word
                        )
                    },
                    agentId: $0.agentId,
                    translation: $0.translation
                )
            }

        let sections: [LyricsSection]? = parsed.sections.map { secs in
            secs.map { sec in
                LyricsSection(
                    name: sec.name,
                    startTime: Double(sec.startTimeMs) / 1000.0,
                    endTime: Double(sec.endTimeMs) / 1000.0,
                    lines: sec.lines.map {
                        SyncedLine(
                            time: Double($0.timeMs) / 1000.0,
                            endTime: $0.endTimeMs.map { Double($0) / 1000.0 },
                            line: $0.line,
                            words: $0.words?.map {
                                SyncedWord(
                                    time: Double($0.timeMs) / 1000.0,
                                    endTime: $0.endTimeMs.map { Double($0) / 1000.0 },
                                    word: $0.word
                                )
                            },
                            agentId: $0.agentId,
                            translation: $0.translation
                        )
                    },
                    agentId: sec.agentId
                )
            }
        }

        let agents: [LyricsAgent]? = parsed.agents.map { agts in
            agts.map { LyricsAgent(id: $0.id, agentType: $0.agentType) }
        }

        let plain = parsed.plain.isEmpty ? nil : parsed.plain.joined(separator: "\n")

        return Lyrics(
            plain: plain,
            synced: synced,
            sections: sections,
            agents: agents,
            songwriters: parsed.songwriters,
            language: parsed.language,
            areFromRemote: parsed.areFromRemote
        )
    }
}
