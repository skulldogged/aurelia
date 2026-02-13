package com.aurelia.app.ui

import android.os.SystemClock
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.aurelia.app.auth.AuthInterceptor
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore
import com.aurelia.app.utils.buildSongIdCache
import com.aurelia.app.utils.validateSession
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.aurelia_core.AppException
import uniffi.aurelia_core.Song
import uniffi.aurelia_core.deriveMobileHomeData
import uniffi.aurelia_core.fetchSongs
import uniffi.aurelia_core.loadCachedSongs

class HomeViewModel(
  private val sessionStore: SessionStore,
  private val playerController: PlayerController,
) : ViewModel() {
  private val mutableState = MutableStateFlow(HomeState())
  val state: StateFlow<HomeState> = mutableState

  // All songs cache for queue building
  private var allSongs: List<Song> = emptyList()
  private val useSharedHomeDerivation = true

  // Cache for song ID lookup
  private var songIdByTitleArtist: Map<Pair<String, String>, String> = emptyMap()
  private var loadJob: Job? = null
  private var lastLoadedAtMs: Long = 0L

  private val nowPlayingMapper = NowPlayingMapper()

  init {
    viewModelScope.launch {
      playerController.snapshots.collect { snapshot ->
        if (!nowPlayingMapper.shouldUpdate(snapshot)) return@collect
        val (nowPlaying, songId) = nowPlayingMapper.mapToNowPlaying(snapshot, songIdByTitleArtist)
        mutableState.update { it.copy(nowPlaying = nowPlaying, currentSongId = songId) }
      }
    }
  }

  fun ensureLoaded(force: Boolean = false) {
    if (!force && loadJob?.isActive == true) return
    val hasData =
      mutableState.value.featuredAlbums.isNotEmpty() ||
        mutableState.value.mostPlayed.isNotEmpty() ||
        mutableState.value.recentlyPlayed.isNotEmpty()
    val isFresh = SystemClock.elapsedRealtime() - lastLoadedAtMs < LOAD_FRESHNESS_MS
    if (!force && hasData && isFresh) return

    val session = validateSession(sessionStore)
    if (session == null) {
      mutableState.update { it.copy(error = "Missing session data") }
      return
    }

    mutableState.update { it.copy(isLoading = true, error = null) }

    loadJob = viewModelScope.launch(Dispatchers.IO) {
      // Try loading from cache first
      if (!session.appDataDir.isNullOrBlank()) {
        try {
          val cachedSongs = loadCachedSongs(session.appDataDir)
          if (cachedSongs.isNotEmpty()) {
            allSongs = cachedSongs
            songIdByTitleArtist = buildSongIdCache(cachedSongs)
            processHomeData(cachedSongs)
          }
        } catch (e: Exception) {
          Log.w("HomeViewModel", "Failed to load cached songs", e)
        }
      }

      // Fetch fresh data
      try {
        val songs = fetchSongs(session.serverUrl, session.token, session.userId, session.appDataDir ?: "")
        allSongs = songs
        songIdByTitleArtist = buildSongIdCache(songs)
        processHomeData(songs)
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

  fun loadHomeData() {
    ensureLoaded(force = false)
  }

  private fun processHomeData(songs: List<Song>) {
    if (useSharedHomeDerivation) {
      processHomeDataWithSharedDerivation(songs)
      return
    }

    processHomeDataLegacy(songs)
  }

  private fun processHomeDataWithSharedDerivation(songs: List<Song>) {
    val derived =
      deriveMobileHomeData(
        songs = songs,
        mostPlayedLimit = UiConstants.MOST_PLAYED_LIMIT.toLong(),
        recentlyPlayedLimit = UiConstants.RECENTLY_PLAYED_LIMIT.toLong(),
        albumSectionLimit = UiConstants.ALBUM_SECTION_LIMIT.toLong(),
        featuredAlbumsLimit = UiConstants.FEATURED_ALBUMS_LIMIT.toLong(),
      )

    mutableState.update {
      it.copy(
        isLoading = false,
        featuredAlbums = derived.featuredAlbums.map(::toFeaturedAlbum),
        mostPlayed = derived.mostPlayed,
        recentlyPlayed = derived.recentlyPlayed,
        recentlyAddedAlbums = derived.recentlyAdded.map(::toAlbumItem),
        randomAlbums = derived.randomAlbums.map(::toAlbumItem),
      )
    }
  }

  private fun processHomeDataLegacy(songs: List<Song>) {
    // Most played - top 10 by playCount
    val mostPlayed =
      songs
        .filter { (it.playCount ?: 0) > 0 }
        .sortedByDescending { it.playCount ?: 0 }
        .take(UiConstants.MOST_PLAYED_LIMIT)

    // Recently played
    val recentlyPlayed =
      songs
        .filter { !it.datePlayed.isNullOrBlank() }
        .sortedByDescending { it.datePlayed ?: "" }
        .take(UiConstants.RECENTLY_PLAYED_LIMIT)

    // Group songs by album for album-based sections
    val albumsMap =
      songs
        .filter { !it.albumId.isNullOrBlank() }
        .groupBy { it.albumId.orEmpty() }

    // Recently added albums - by dateCreated of first song
    val recentlyAddedAlbums =
      albumsMap
        .map { (albumId, albumSongs) ->
          val firstSong = albumSongs.maxByOrNull { it.dateCreated ?: "" } ?: albumSongs.first()
          AlbumItemWithDate(
            album =
              AlbumItem(
                id = albumId,
                name = firstSong.album ?: "Unknown Album",
                artist = firstSong.artists?.firstOrNull() ?: "Unknown Artist",
                albumArtUrl = firstSong.albumArtUrl,
                songCount = albumSongs.size,
              ),
            dateCreated = firstSong.dateCreated ?: "",
          )
        }.sortedByDescending { it.dateCreated }
        .take(UiConstants.ALBUM_SECTION_LIMIT)
        .map { it.album }

    // Random albums for "From Your Library"
    val randomAlbums =
      albumsMap
        .map { (albumId, albumSongs) ->
          val firstSong = albumSongs.first()
          AlbumItem(
            id = albumId,
            name = firstSong.album ?: "Unknown Album",
            artist = firstSong.artists?.firstOrNull() ?: "Unknown Artist",
            albumArtUrl = firstSong.albumArtUrl,
            songCount = albumSongs.size,
          )
        }.shuffled()
        .take(UiConstants.ALBUM_SECTION_LIMIT)

    // Featured albums - random selection with album art
    val featuredAlbums =
      albumsMap
        .filter { (_, albumSongs) -> albumSongs.any { !it.albumArtUrl.isNullOrBlank() } }
        .map { (albumId, albumSongs) ->
          val firstSong = albumSongs.first()
          FeaturedAlbum(
            id = albumId,
            name = firstSong.album ?: "Unknown Album",
            artist = firstSong.artists?.joinToString(", ") ?: "Unknown Artist",
            albumArtUrl = firstSong.albumArtUrl,
            songCount = albumSongs.size,
          )
        }.shuffled()
        .take(UiConstants.FEATURED_ALBUMS_LIMIT)

    mutableState.update {
      it.copy(
        isLoading = false,
        featuredAlbums = featuredAlbums,
        mostPlayed = mostPlayed,
        recentlyPlayed = recentlyPlayed,
        recentlyAddedAlbums = recentlyAddedAlbums,
        randomAlbums = randomAlbums,
      )
    }
  }

  private fun toAlbumItem(album: uniffi.aurelia_core.Album): AlbumItem =
    AlbumItem(
      id = album.id ?: "",
      name = album.name,
      artist = album.artist,
      albumArtUrl = album.albumArtUrl,
      songCount = album.songCount.toInt(),
    )

  private fun toFeaturedAlbum(album: uniffi.aurelia_core.Album): FeaturedAlbum =
    FeaturedAlbum(
      id = album.id ?: "",
      name = album.name,
      artist = album.artist,
      albumArtUrl = album.albumArtUrl,
      songCount = album.songCount.toInt(),
    )

  fun nextFeaturedAlbum() {
    val current = mutableState.value
    if (current.featuredAlbums.isNotEmpty()) {
      val nextIndex = (current.currentFeaturedIndex + 1) % current.featuredAlbums.size
      mutableState.update { it.copy(currentFeaturedIndex = nextIndex) }
    }
  }

  fun previousFeaturedAlbum() {
    val current = mutableState.value
    if (current.featuredAlbums.isNotEmpty()) {
      val prevIndex =
        if (current.currentFeaturedIndex > 0) {
          current.currentFeaturedIndex - 1
        } else {
          current.featuredAlbums.size - 1
        }
      mutableState.update { it.copy(currentFeaturedIndex = prevIndex) }
    }
  }

  fun setFeaturedIndex(index: Int) {
    val current = mutableState.value
    if (index in current.featuredAlbums.indices) {
      mutableState.update { it.copy(currentFeaturedIndex = index) }
    }
  }

  /**
   * Play a song from a specific list, setting up the queue from that list
   */
  fun playSongFromList(
    songId: String,
    songList: List<Song>,
  ) {
    val serverUrl = sessionStore.getServerUrl() ?: return
    val token = sessionStore.getToken() ?: return

    val startIndex = songList.indexOfFirst { it.id == songId }
    if (startIndex < 0) return

    mutableState.update { it.copy(currentSongId = songId) }
    playerController.setQueue(songList, serverUrl, token, startIndex)
  }

  /**
   * Play all songs from an album
   */
  fun playAlbum(albumId: String) {
    val serverUrl = sessionStore.getServerUrl() ?: return
    val token = sessionStore.getToken() ?: return

    val albumSongs =
      allSongs
        .filter { it.albumId == albumId }
        .sortedBy { it.trackNumber ?: 0 }

    if (albumSongs.isEmpty()) return

    mutableState.update { it.copy(currentSongId = albumSongs.first().id) }
    playerController.setQueue(albumSongs, serverUrl, token)
  }

  /**
   * Shuffle play an album
   */
  fun shuffleAlbum(albumId: String) {
    val serverUrl = sessionStore.getServerUrl() ?: return
    val token = sessionStore.getToken() ?: return

    val albumSongs =
      allSongs
        .filter { it.albumId == albumId }
        .shuffled()

    if (albumSongs.isEmpty()) return

    mutableState.update { it.copy(currentSongId = albumSongs.first().id) }
    playerController.setQueue(albumSongs, serverUrl, token)
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
}

private data class AlbumItemWithDate(
  val album: AlbumItem,
  val dateCreated: String,
)

private const val LOAD_FRESHNESS_MS = 60_000L
