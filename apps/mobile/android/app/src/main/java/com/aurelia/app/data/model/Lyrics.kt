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
  val translation: String? = null,
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

  /** Check if an agent ID refers to a secondary vocalist — a `person` agent
   *  that is NOT the first (primary) person in the agents list.
   *  Apple Music right-aligns these lines to visually distinguish duet parts. */
  fun isSecondaryVocalist(agentId: String?): Boolean {
    if (agentId == null || agents == null) return false
    val firstPerson = agents.firstOrNull { it.agentType == "person" } ?: return false
    if (agentId == firstPerson.id) return false
    return agents.any { it.id == agentId && it.agentType == "person" }
  }
}
