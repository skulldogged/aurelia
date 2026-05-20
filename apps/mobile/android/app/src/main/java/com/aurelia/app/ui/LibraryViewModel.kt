package com.aurelia.app.ui

import android.os.SystemClock
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.aurelia.app.auth.AuthInterceptor
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore
import com.aurelia.app.utils.buildSongIdCache
import com.aurelia.app.utils.jellyfinPrimaryImageUrl
import com.aurelia.app.utils.validateSession
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.aurelia_core.AppException
import uniffi.aurelia_core.fetchSongs
import uniffi.aurelia_core.loadCachedSongs

@OptIn(FlowPreview::class)
class LibraryViewModel(
  private val sessionStore: SessionStore,
  private val playerController: PlayerController,
) : ViewModel() {
  private val mutableState = MutableStateFlow(LibraryState())
  val state: StateFlow<LibraryState> = mutableState

  // Cache for song ID lookup - built once when songs load
  private var songIdByTitleArtist: Map<Pair<String, String>, String> = emptyMap()
  private var loadJob: Job? = null
  private var lastLoadedAtMs: Long = 0L
  private val searchQuery = MutableStateFlow("")
  private val mutableSearchResults = MutableStateFlow<List<SearchResult>>(emptyList())
  val searchResults: StateFlow<List<SearchResult>> = mutableSearchResults

  private val nowPlayingMapper = NowPlayingMapper()

  init {
    viewModelScope.launch {
      playerController.snapshots.collect { snapshot ->
        if (!nowPlayingMapper.shouldUpdate(snapshot)) return@collect
        val (nowPlaying, songId) = nowPlayingMapper.mapToNowPlaying(
          snapshot, songIdByTitleArtist, includeNavigation = true,
        )
        mutableState.update { it.copy(nowPlaying = nowPlaying, currentSongId = songId) }
      }
    }

    viewModelScope.launch {
      combine(
        searchQuery.debounce(SEARCH_DEBOUNCE_MS),
        mutableState.map { it.songs },
      ) { query, songs ->
        computeSearchResults(query, songs)
      }
        .flowOn(Dispatchers.Default)
        .collect { results ->
          mutableSearchResults.value = results
        }
    }
  }

  fun ensureLoaded(force: Boolean = false) {
    if (!force && loadJob?.isActive == true) return
    val hasSongs = mutableState.value.songs.isNotEmpty()
    val isFresh = SystemClock.elapsedRealtime() - lastLoadedAtMs < LOAD_FRESHNESS_MS
    if (!force && hasSongs && isFresh) return

    val session = validateSession(sessionStore)
    if (session == null) {
      mutableState.update { it.copy(error = "Missing session data") }
      return
    }

    mutableState.update { it.copy(isLoading = true, error = null) }

    loadJob = viewModelScope.launch(Dispatchers.IO) {
      if (!session.appDataDir.isNullOrBlank()) {
        try {
          val cachedSongs = loadCachedSongs(session.appDataDir)
          if (cachedSongs.isNotEmpty()) {
            songIdByTitleArtist = buildSongIdCache(cachedSongs)
            mutableState.update { it.copy(songs = cachedSongs, isLoading = false) }
          }
        } catch (e: Exception) {
          Log.w("LibraryViewModel", "Failed to load cached songs", e)
        }
      }

      try {
        val songs = fetchSongs(session.serverUrl, session.token, session.userId, session.appDataDir ?: "")
        songIdByTitleArtist = buildSongIdCache(songs)
        mutableState.update { it.copy(isLoading = false, songs = songs) }
        lastLoadedAtMs = SystemClock.elapsedRealtime()
      } catch (error: AppException) {
        if (!AuthInterceptor.handlePotentialAuthError(error.message)) {
          mutableState.update { it.copy(isLoading = false, error = error.message ?: "Failed to load") }
        }
      } catch (error: Exception) {
        if (!AuthInterceptor.handlePotentialAuthError(error)) {
          mutableState.update { it.copy(isLoading = false, error = "Failed to load") }
        }
      }
    }
  }

  fun loadLibrary() {
    ensureLoaded(force = false)
  }

  fun updateSearchQuery(query: String) {
    searchQuery.value = query
  }

  fun play(songId: String) {
    val serverUrl = sessionStore.getServerUrl() ?: return
    val token = sessionStore.getToken() ?: return
    val song = mutableState.value.songs.firstOrNull { it.id == songId } ?: return

    // Update current song ID immediately for responsive UI
    mutableState.update { it.copy(currentSongId = songId) }

    playerController.play(song, serverUrl, token)
  }

  /**
   * Plays a song from the current song list, setting up the full queue for next/previous navigation.
   * @param songId The ID of the song to start playing
   */
  fun playFromList(songId: String) {
    val serverUrl = sessionStore.getServerUrl() ?: return
    val token = sessionStore.getToken() ?: return
    val songs = mutableState.value.songs

    val startIndex = songs.indexOfFirst { it.id == songId }
    if (startIndex < 0) return

    // Update current song ID immediately for responsive UI
    mutableState.update { it.copy(currentSongId = songId) }

    playerController.setQueue(songs, serverUrl, token, startIndex)
  }

  fun togglePlayPause() {
    val nowPlaying = mutableState.value.nowPlaying ?: return
    if (nowPlaying.isPlaying) {
      playerController.pause()
    } else {
      playerController.resume()
    }
  }

  fun skipPrevious() {
    playerController.skipPrevious()
  }

  fun skipNext() {
    playerController.skipNext()
  }

  private fun computeSearchResults(
    query: String,
    songs: List<uniffi.aurelia_core.Song>,
  ): List<SearchResult> {
    if (query.length < UiConstants.MIN_SEARCH_LENGTH) return emptyList()

    val normalizedQuery = query.lowercase()
    val songResults =
      songs
        .asSequence()
        .filter { song ->
          song.name.lowercase().contains(normalizedQuery) ||
            song.artists?.any { it.lowercase().contains(normalizedQuery) } == true ||
            song.album?.lowercase()?.contains(normalizedQuery) == true
        }
        .take(UiConstants.SEARCH_RESULTS_LIMIT)
        .map { SearchResult.SongResult(it) }
        .toList()

    val albumResults =
      songs
        .asSequence()
        .filter { it.album?.lowercase()?.contains(normalizedQuery) == true }
        .mapNotNull { song ->
          song.albumId?.let { id ->
            Triple(id, song.album ?: "", song.albumArtUrl)
          }
        }
        .distinctBy { it.first }
        .take(UiConstants.SEARCH_ALBUMS_LIMIT)
        .map { (id, name, artUrl) -> SearchResult.Album(id, name, "", artUrl) }
        .toList()

    val artistResults =
      songs
        .asSequence()
        .filter { it.artists?.any { artist -> artist.lowercase().contains(normalizedQuery) } == true }
        .flatMap { song ->
          (song.artists ?: emptyList()).asSequence().mapIndexedNotNull { index, artist ->
            if (artist.lowercase().contains(normalizedQuery)) {
              val artistId = song.artistIds?.getOrNull(index)
              Triple(artistId, artist, song)
            } else {
              null
            }
          }
        }
        .groupBy { it.second }
        .map { (name, entries) ->
          val artistId = entries.firstOrNull()?.first
          SearchResult.Artist(
            id = artistId,
            name = name,
            songCount = entries.size,
            imageUrl = jellyfinPrimaryImageUrl(sessionStore.getServerUrl(), artistId, sessionStore.getToken()),
          )
        }
        .take(UiConstants.SEARCH_ARTISTS_LIMIT)

    return artistResults + albumResults + songResults
  }
}

private const val LOAD_FRESHNESS_MS = 60_000L
private const val SEARCH_DEBOUNCE_MS = 120L
