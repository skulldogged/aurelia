package com.aurelia.app.utils

import com.aurelia.app.data.model.Lyrics
import com.aurelia.app.data.model.SyncedLine
import com.aurelia.app.data.model.SyncedWord
import java.util.regex.Pattern

object LyricsUtils {
    private val LRC_LINE_REGEX = Pattern.compile("^\\[(\\d{2}):(\\d{2})\\.(\\d{2,3})](.*)$")
    private val LRC_WORD_REGEX = Pattern.compile("<(\\d{2}):(\\d{2})\\.(\\d{2,3})>([^<]*)")
    private val LRC_WORD_TAG_REGEX = Regex("<\\d{2}:\\d{2}\\.\\d{2,3}>")
    private val LRC_WORD_SPLIT_REGEX = Regex("(?=<\\d{2}:\\d{2}\\.\\d{2,3}>)")
    private val LRC_TIMESTAMP_TAG_REGEX = Regex("\\[\\d{1,2}:\\d{2}(?:\\.\\d{1,3})?]")

    fun parseLyrics(lyricsText: String?): Lyrics {
        if (lyricsText.isNullOrEmpty()) {
            return Lyrics(plain = emptyList(), synced = emptyList())
        }

        val syncedLines = mutableListOf<SyncedLine>()
        val plainLines = mutableListOf<String>()
        var isSynced = false

        lyricsText.lines().forEach { rawLine ->
            val line = sanitizeLrcLine(rawLine)
            if (line.isEmpty()) return@forEach

            val lineMatcher = LRC_LINE_REGEX.matcher(line)
            if (lineMatcher.matches()) {
                isSynced = true
                val minutes = lineMatcher.group(1)?.toLong() ?: 0
                val seconds = lineMatcher.group(2)?.toLong() ?: 0
                val fraction = lineMatcher.group(3)?.toLong() ?: 0
                val textWithTags = stripFormatCharacters(lineMatcher.group(4)?.trim() ?: "")
                val text = stripLrcTimestamps(textWithTags)

                val millis = if (lineMatcher.group(3)?.length == 2) fraction * 10 else fraction
                val lineTimestamp = minutes * 60 * 1000 + seconds * 1000 + millis

                if (text.contains(LRC_WORD_TAG_REGEX)) {
                    val words = mutableListOf<SyncedWord>()
                    val parts = text.split(LRC_WORD_SPLIT_REGEX)

                    for (part in parts) {
                        if (part.isEmpty()) continue
                        val wordMatcher = LRC_WORD_REGEX.matcher(part)
                        if (wordMatcher.find()) {
                            val wordMinutes = wordMatcher.group(1)?.toLong() ?: 0
                            val wordSeconds = wordMatcher.group(2)?.toLong() ?: 0
                            val wordFraction = wordMatcher.group(3)?.toLong() ?: 0
                            val wordText = stripFormatCharacters(wordMatcher.group(4) ?: "")
                            val wordMillis = if (wordMatcher.group(3)?.length == 2) wordFraction * 10 else wordFraction
                            val wordTimestamp = wordMinutes * 60 * 1000 + wordSeconds * 1000 + wordMillis
                            words.add(SyncedWord(wordTimestamp.toInt(), wordText))
                        } else {
                            val lastTime = words.lastOrNull()?.time ?: lineTimestamp.toInt()
                            words.add(SyncedWord(lastTime, part))
                        }
                    }

                    if (words.isNotEmpty()) {
                        val fullLineText = words.joinToString("") { it.word }
                        syncedLines.add(SyncedLine(lineTimestamp.toInt(), fullLineText, words))
                    } else {
                        syncedLines.add(SyncedLine(lineTimestamp.toInt(), text))
                    }
                } else {
                    syncedLines.add(SyncedLine(lineTimestamp.toInt(), text))
                }
            } else {
                val stripped = stripLrcTimestamps(stripFormatCharacters(line))
                if (isSynced && syncedLines.isNotEmpty()) {
                    val last = syncedLines.removeAt(syncedLines.lastIndex)
                    val mergedLineText =
                        if (last.line.isEmpty()) {
                            stripped
                        } else {
                            last.line + "\n" + stripped
                        }
                    val merged =
                        if (last.words?.isNotEmpty() == true) {
                            SyncedLine(last.time, mergedLineText, last.words)
                        } else {
                            SyncedLine(last.time, mergedLineText)
                        }
                    syncedLines.add(merged)
                } else {
                    plainLines.add(stripped)
                }
            }
        }

        return if (isSynced && syncedLines.isNotEmpty()) {
            val sortedSyncedLines = syncedLines.sortedBy { it.time }
            val plainVersion = sortedSyncedLines.map { it.line }
            Lyrics(synced = sortedSyncedLines, plain = plainVersion)
        } else {
            Lyrics(plain = plainLines)
        }
    }

    fun stripLrcTimestamps(value: String): String {
        if (value.isEmpty()) return value
        val withoutTags = LRC_TIMESTAMP_TAG_REGEX.replace(value, "")
        return withoutTags.trimStart()
    }

    private fun sanitizeLrcLine(rawLine: String): String {
        if (rawLine.isEmpty()) return rawLine

        val withoutTerminators =
            rawLine
                .trimEnd('\r', '\n')
                .filterNot { char ->
                    Character.getType(char).toByte() == Character.FORMAT ||
                        (Character.isISOControl(char) && char != '\t')
                }.trimEnd('\uFEFF')

        val trimmedPrefix = withoutTerminators.trimStart { it.isWhitespace() }
        val firstBracket = trimmedPrefix.indexOf('[')
        return if (firstBracket > 0) {
            trimmedPrefix.substring(firstBracket)
        } else {
            trimmedPrefix
        }
    }

    private fun stripFormatCharacters(value: String): String {
        val cleaned =
            value.filterNot { char ->
                Character.getType(char).toByte() == Character.FORMAT ||
                    (Character.isISOControl(char) && char != '\t')
            }
        return when (cleaned) {
            "\"", "'" -> ""
            else -> cleaned
        }
    }
}
