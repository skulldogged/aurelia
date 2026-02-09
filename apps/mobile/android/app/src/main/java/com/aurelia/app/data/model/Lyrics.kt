package com.aurelia.app.data.model

data class SyncedWord(
  val time: Int,
  val endTime: Int? = null,
  val word: String,
)

data class SyncedLine(
  val time: Int,
  val endTime: Int? = null,
  val line: String,
  val words: List<SyncedWord>? = null,
  val agentId: String? = null,
)

data class LyricsSection(
  val name: String,
  val startTime: Int,
  val endTime: Int,
  val lines: List<SyncedLine>,
  val agentId: String? = null,
)

data class LyricsAgent(
  val id: String,
  val agentType: String,
)

data class Lyrics(
  val plain: List<String>? = null,
  val synced: List<SyncedLine>? = null,
  val sections: List<LyricsSection>? = null,
  val agents: List<LyricsAgent>? = null,
  val songwriters: List<String>? = null,
  val language: String? = null,
  val areFromRemote: Boolean = false,
) {
  fun isValid(): Boolean = !synced.isNullOrEmpty() || !plain.isNullOrEmpty()

  /** Check if an agent ID refers to a background/other voice. */
  fun isBackgroundVocal(agentId: String?): Boolean {
    if (agentId == null || agents == null) return false
    return agents.find { it.id == agentId }?.agentType == "other"
  }
}
