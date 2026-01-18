package com.aurelia.app.data.model

data class SyncedWord(val time: Int, val word: String)

data class SyncedLine(
    val time: Int,
    val line: String,
    val words: List<SyncedWord>? = null
)

data class Lyrics(
    val plain: List<String>? = null,
    val synced: List<SyncedLine>? = null,
    val areFromRemote: Boolean = false
) {
    fun isValid(): Boolean = !synced.isNullOrEmpty() || !plain.isNullOrEmpty()
}
