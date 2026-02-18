using Aurelia.Services;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Runtime.CompilerServices;

namespace Aurelia.ViewModels;

public class LibraryViewModel : INotifyPropertyChanged {
  private readonly ApiService _apiService;
  private readonly PlayerService _playerService;

  private bool _isLoading;
  private int _selectedTab;

  public event PropertyChangedEventHandler? PropertyChanged;

  public bool IsLoading {
    get => _isLoading;
    set { _isLoading = value; OnPropertyChanged(); }
  }

  public int SelectedTab {
    get => _selectedTab;
    set { _selectedTab = value; OnPropertyChanged(); }
  }

  public ObservableCollection<Song> Songs { get; } = [];
  public ObservableCollection<Album> Albums { get; } = [];
  public ObservableCollection<Artist> Artists { get; } = [];
  public ObservableCollection<Playlist> Playlists { get; } = [];

  public LibraryViewModel(ApiService apiService, PlayerService playerService) {
    _apiService = apiService;
    _playerService = playerService;
  }

  public async Task LoadAsync() {
    IsLoading = true;
    try {
      await Task.WhenAll(LoadSongsAsync(), LoadAlbumsAsync(), LoadArtistsAsync(), LoadPlaylistsAsync());
    } finally {
      IsLoading = false;
    }
  }

  private async Task LoadSongsAsync() {
    Songs.Clear();
    List<Song> songs = await _apiService.GetSongsAsync();
    foreach (Song song in songs)
      Songs.Add(song);
  }

  private async Task LoadAlbumsAsync() {
    Albums.Clear();
    List<Album> albums = await _apiService.GetAlbumsAsync();
    foreach (Album album in albums)
      Albums.Add(album);
  }

  private async Task LoadArtistsAsync() {
    Artists.Clear();
    List<Artist> artists = await _apiService.GetArtistsAsync();
    foreach (Artist artist in artists)
      Artists.Add(artist);
  }

  private async Task LoadPlaylistsAsync() {
    Playlists.Clear();
    List<Playlist> playlists = await _apiService.GetPlaylistsAsync();
    foreach (Playlist playlist in playlists)
      Playlists.Add(playlist);
  }

  public void PlaySong(Song song) {
    int index = Songs.IndexOf(song);
    _playerService.SetQueue(Songs.ToList(), index >= 0 ? index : 0);
    _ = _playerService.PlayAsync();
  }

  public void PlayAlbum(Album album) {
    _ = PlayAlbumInternal(album);
  }

  private async Task PlayAlbumInternal(Album album) {
    List<Song> songs = await _apiService.GetSongsAsync(albumId: album.id);
    if (songs.Count > 0) {
      _playerService.SetQueue(songs, 0);
      await _playerService.PlayAsync();
    }
  }

  public void PlayArtist(Artist artist) {
    _ = PlayArtistInternal(artist);
  }

  private async Task PlayArtistInternal(Artist artist) {
    List<Song> songs = await _apiService.GetSongsAsync(artistId: artist.id);
    if (songs.Count > 0) {
      _playerService.SetQueue(songs, 0);
      await _playerService.PlayAsync();
    }
  }

  public void PlayPlaylist(Playlist playlist) {
    _ = PlayPlaylistInternal(playlist);
  }

  private async Task PlayPlaylistInternal(Playlist playlist) {
    Playlist? fullPlaylist = await _apiService.GetPlaylistAsync(playlist.id);
    if (fullPlaylist?.songs != null && fullPlaylist.songs.Length > 0) {
      _playerService.SetQueue(fullPlaylist.songs.ToList(), 0);
      await _playerService.PlayAsync();
    }
  }

  protected void OnPropertyChanged([CallerMemberName] string? propertyName = null) {
    PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
  }
}
