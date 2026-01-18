package com.aurelia.app.data.network

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import com.aurelia.app.data.model.Lyrics
import com.aurelia.app.utils.LyricsUtils
import org.json.JSONArray

object LrcLibApi {
    private val client = OkHttpClient()

    private const val SEARCH_URL = "https://lrclib.net/api/search"

    suspend fun searchLyrics(artist: String, title: String): Lyrics? = withContext(Dispatchers.IO) {
        try {
            val request = Request.Builder()
                .url("$SEARCH_URL?artist_name=$artist&track_name=$title")
                .addHeader("User-Agent", "Aurelia-Android/1.0")
                .get()
                .build()

            client.newCall(request).execute().use { response ->
                if (!response.isSuccessful) return@withContext null

                val body = response.body?.string() ?: return@withContext null
                val results = JSONArray(body)

                if (results.length() == 0) return@withContext null

                val bestResult = results.getJSONObject(0)
                val syncedLyrics = bestResult.optString("syncedLyrics", null)
                val plainLyrics = bestResult.optString("plainLyrics", null)

                val rawLyrics = syncedLyrics ?: plainLyrics
                if (rawLyrics.isNullOrBlank()) return@withContext null

                LyricsUtils.parseLyrics(rawLyrics).copy(areFromRemote = syncedLyrics != null)
            }
        } catch (e: Exception) {
            e.printStackTrace()
            null
        }
    }
}
