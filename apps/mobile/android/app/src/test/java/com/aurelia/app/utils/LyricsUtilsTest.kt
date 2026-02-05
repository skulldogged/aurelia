package com.aurelia.app.utils

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class LyricsUtilsTest {
  @Test
  fun parseLyrics_handlesSyncedLines() {
    val input = """
      [00:10.00]Hello
      [00:20.50]World
    """.trimIndent()

    val lyrics = LyricsUtils.parseLyrics(input)

    assertTrue(lyrics.isValid())
    assertEquals(listOf("Hello", "World"), lyrics.plain)
    assertEquals(2, lyrics.synced?.size)
    assertEquals(10_000, lyrics.synced?.get(0)?.time)
    assertEquals("Hello", lyrics.synced?.get(0)?.line)
  }

  @Test
  fun parseLyrics_parsesWordLevelTags() {
    val input = "[00:01.00]<00:01.00>Hi <00:01.50>there"
    val lyrics = LyricsUtils.parseLyrics(input)

    assertTrue(lyrics.isValid())
    val line = lyrics.synced?.first()
    assertEquals("Hi there", line?.line)
    assertEquals(2, line?.words?.size)
    assertEquals(1_000, line?.words?.get(0)?.time)
    assertEquals("Hi ", line?.words?.get(0)?.word)
  }

  @Test
  fun stripLrcTimestamps_removesTags() {
    val stripped = LyricsUtils.stripLrcTimestamps("[01:23.45] Hello")
    assertEquals("Hello", stripped)
  }
}
