package com.aurelia.app.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.aurelia.app.player.PlayerController
import com.aurelia.app.player.PlayerSnapshot
import com.aurelia.app.player.QueueItem
import com.aurelia.app.storage.SessionStore
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.aurelia_core.AppException
import uniffi.aurelia_core.Song
import uniffi.aurelia_core.buildStreamUrl
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
  
  // Cache for song ID lookup
  private var songIdByTitleArtist: Map<Pair<String, String>, String> = emptyMap()
  
  // Track last snapshot to avoid redundant updates
  private var lastTitle: String = ""
  private var lastArtist: String = ""
  private var lastIsPlaying: Boolean = false

  init {
    playerController.observe { snapshot ->
      handlePlayerUpdate(snapshot)
    }
  }

  private fun handlePlayerUpdate(snapshot: PlayerSnapshot) {
    val titleChanged = snapshot.title != lastTitle
    val artistChanged = snapshot.artist != lastArtist
    val playingChanged = snapshot.isPlaying != lastIsPlaying

    if (!titleChanged && !artistChanged && !playingChanged) {
      return
    }

    lastTitle = snapshot.title
    lastArtist = snapshot.artist
    lastIsPlaying = snapshot.isPlaying

    if (snapshot.title.isBlank()) {
      mutableState.update { it.copy(nowPlaying = null, currentSongId = null) }
      return
    }

    val songId = songIdByTitleArtist[Pair(snapshot.title, snapshot.artist)]

    mutableState.update {
      it.copy(
        nowPlaying = NowPlayingState(snapshot.title, snapshot.artist, snapshot.albumArtUrl, snapshot.isPlaying),
        currentSongId = songId
      )
    }
  }

  private fun buildSongIdCache(songs: List<Song>) {
    songIdByTitleArtist = songs.associate { song ->
      val artist = song.artists?.joinToString(", ") ?: ""
      Pair(song.name, artist) to song.id
    }
  }

  fun loadHomeData() {
    val serverUrl = sessionStore.getServerUrl()
    val userId = sessionStore.getUserId()
    val token = sessionStore.getToken()
    val appDataDir = sessionStore.getAppDataDir()

    if (serverUrl.isNullOrBlank() || userId.isNullOrBlank() || token.isNullOrBlank()) {
      mutableState.update { it.copy(error = "Missing session data") }
      return
    }

    mutableState.update { it.copy(isLoading = true, error = null) }

    // Try loading from cache first
    if (!appDataDir.isNullOrBlank()) {
      viewModelScope.launch(Dispatchers.IO) {
        try {
          val cachedSongs = loadCachedSongs(appDataDir)
          if (cachedSongs.isNotEmpty()) {
            allSongs = cachedSongs
            buildSongIdCache(cachedSongs)
            processHomeData(cachedSongs)
          }
        } catch (_: Exception) {
          // Ignore cache errors
        }
      }
    }

    // Fetch fresh data
    viewModelScope.launch(Dispatchers.IO) {
      try {
        val songs = fetchSongs(serverUrl, token, userId, appDataDir ?: "")
        allSongs = songs
        buildSongIdCache(songs)
        processHomeData(songs)
      } catch (error: AppException) {
        mutableState.update { it.copy(isLoading = false, error = error.message ?: "Failed to load") }
      } catch (_: Exception) {
        mutableState.update { it.copy(isLoading = false, error = "Failed to load") }
      }
    }
  }

  private fun processHomeData(songs: List<Song>) {
    // Most played - top 10 by playCount
    val mostPlayed = songs
      .filter { (it.playCount ?: 0) > 0 }
      .sortedByDescending { it.playCount ?: 0 }
      .take(10)

    // Recently played - top 10 by datePlayed
    val recentlyPlayed = songs
      .filter { !it.datePlayed.isNullOrBlank() }
      .sortedByDescending { it.datePlayed ?: "" }
      .take(10)

    // Group songs by album for album-based sections
    val albumsMap = songs
      .filter { !it.albumId.isNullOrBlank() }
      .groupBy { it.albumId!! }

    // Recently added albums - by dateCreated of first song
    val recentlyAddedAlbums = albumsMap
      .map { (albumId, albumSongs) ->
        val firstSong = albumSongs.maxByOrNull { it.dateCreated ?: "" } ?: albumSongs.first()
        AlbumItemWithDate(
          album = AlbumItem(
            id = albumId,
            name = firstSong.album ?: "Unknown Album",
            artist = firstSong.artists?.firstOrNull() ?: "Unknown Artist",
            albumArtUrl = firstSong.albumArtUrl,
            songCount = albumSongs.size
          ),
          dateCreated = firstSong.dateCreated ?: ""
        )
      }
      .sortedByDescending { it.dateCreated }
      .take(12)
      .map { it.album }

    // Random albums for "From Your Library"
    val randomAlbums = albumsMap
      .map { (albumId, albumSongs) ->
        val firstSong = albumSongs.first()
        AlbumItem(
          id = albumId,
          name = firstSong.album ?: "Unknown Album",
          artist = firstSong.artists?.firstOrNull() ?: "Unknown Artist",
          albumArtUrl = firstSong.albumArtUrl,
          songCount = albumSongs.size
        )
      }
      .shuffled()
      .take(12)

    // Featured albums - random selection with album art
    val featuredAlbums = albumsMap
      .filter { (_, albumSongs) -> albumSongs.any { !it.albumArtUrl.isNullOrBlank() } }
      .map { (albumId, albumSongs) ->
        val firstSong = albumSongs.first()
        FeaturedAlbum(
          id = albumId,
          name = firstSong.album ?: "Unknown Album",
          artist = firstSong.artists?.joinToString(", ") ?: "Unknown Artist",
          albumArtUrl = firstSong.albumArtUrl,
          songCount = albumSongs.size
        )
      }
      .shuffled()
      .take(5)

    mutableState.update {
      it.copy(
        isLoading = false,
        featuredAlbums = featuredAlbums,
        mostPlayed = mostPlayed,
        recentlyPlayed = recentlyPlayed,
        recentlyAddedAlbums = recentlyAddedAlbums,
        randomAlbums = randomAlbums
      )
    }
  }

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
      val prevIndex = if (current.currentFeaturedIndex > 0) {
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
  fun playSongFromList(songId: String, songList: List<Song>) {
    val serverUrl = sessionStore.getServerUrl() ?: return
    val token = sessionStore.getToken() ?: return
    
    val startIndex = songList.indexOfFirst { it.id == songId }
    if (startIndex < 0) return
    
    val queueItems = songList.map { song ->
      QueueItem(
        id = song.id,
        uri = buildStreamUrl(serverUrl, token, song.id, song.container),
        title = song.name,
        artist = song.artists?.joinToString(", ") ?: "Unknown Artist",
        albumArtUrl = song.albumArtUrl,
        durationMs = (song.duration ?: 0.0).let { (it * 1000).toLong() }
      )
    }
    
    mutableState.update { it.copy(currentSongId = songId) }
    playerController.setQueue(queueItems, startIndex)
  }

  /**
   * Play all songs from an album
   */
  fun playAlbum(albumId: String) {
    val serverUrl = sessionStore.getServerUrl() ?: return
    val token = sessionStore.getToken() ?: return
    
    val albumSongs = allSongs
      .filter { it.albumId == albumId }
      .sortedBy { it.trackNumber ?: 0 }
    
    if (albumSongs.isEmpty()) return
    
    val queueItems = albumSongs.map { song ->
      QueueItem(
        id = song.id,
        uri = buildStreamUrl(serverUrl, token, song.id, song.container),
        title = song.name,
        artist = song.artists?.joinToString(", ") ?: "Unknown Artist",
        albumArtUrl = song.albumArtUrl,
        durationMs = (song.duration ?: 0.0).let { (it * 1000).toLong() }
      )
    }
    
    mutableState.update { it.copy(currentSongId = albumSongs.first().id) }
    playerController.setQueue(queueItems, 0)
  }

  /**
   * Shuffle play an album
   */
  fun shuffleAlbum(albumId: String) {
    val serverUrl = sessionStore.getServerUrl() ?: return
    val token = sessionStore.getToken() ?: return
    
    val albumSongs = allSongs
      .filter { it.albumId == albumId }
      .shuffled()
    
    if (albumSongs.isEmpty()) return
    
    val queueItems = albumSongs.map { song ->
      QueueItem(
        id = song.id,
        uri = buildStreamUrl(serverUrl, token, song.id, song.container),
        title = song.name,
        artist = song.artists?.joinToString(", ") ?: "Unknown Artist",
        albumArtUrl = song.albumArtUrl,
        durationMs = (song.duration ?: 0.0).let { (it * 1000).toLong() }
      )
    }
    
    mutableState.update { it.copy(currentSongId = albumSongs.first().id) }
    playerController.setQueue(queueItems, 0)
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

  override fun onCleared() {
    super.onCleared()
    playerController.release()
  }
}

private data class AlbumItemWithDate(
  val album: AlbumItem,
  val dateCreated: String
)
