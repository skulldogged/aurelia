using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Runtime.CompilerServices;
using Aurelia.Models;
using Aurelia.Services;

namespace Aurelia.ViewModels;

public class HomeViewModel : INotifyPropertyChanged
{
    private readonly ApiService _apiService;
    private readonly PlayerService _playerService;

    private bool _isLoading;
    private HomeData? _homeData;

    public event PropertyChangedEventHandler? PropertyChanged;

    public bool IsLoading
    {
        get => _isLoading;
        set { _isLoading = value; OnPropertyChanged(); }
    }

    public HomeData? HomeData
    {
        get => _homeData;
        set { _homeData = value; OnPropertyChanged(); }
    }

    public string Greeting
    {
        get
        {
            var hour = DateTime.Now.Hour;
            return hour switch
            {
                < 12 => "Good morning",
                < 17 => "Good afternoon",
                _ => "Good evening"
            };
        }
    }

    public ObservableCollection<AureliaCore.Song> MostPlayed { get; } = [];
    public ObservableCollection<AureliaCore.Song> RecentlyPlayed { get; } = [];
    public ObservableCollection<AureliaCore.Album> RecentlyAddedAlbums { get; } = [];
    public ObservableCollection<AureliaCore.Album> RandomAlbums { get; } = [];
    public ObservableCollection<AureliaCore.Album> FeaturedAlbums { get; } = [];

    public HomeViewModel(ApiService apiService, PlayerService playerService)
    {
        _apiService = apiService;
        _playerService = playerService;
    }

    public async Task LoadAsync()
    {
        IsLoading = true;
        try
        {
            HomeData = await _apiService.GetHomeDataAsync();

            MostPlayed.Clear();
            RecentlyPlayed.Clear();
            RecentlyAddedAlbums.Clear();
            RandomAlbums.Clear();
            FeaturedAlbums.Clear();

            if (HomeData?.MostPlayed != null)
            {
                foreach (var song in HomeData.MostPlayed)
                    MostPlayed.Add(song);
            }

            if (HomeData?.RecentlyPlayed != null)
            {
                foreach (var song in HomeData.RecentlyPlayed)
                    RecentlyPlayed.Add(song);
            }

            if (HomeData?.RecentlyAddedAlbums != null)
            {
                foreach (var album in HomeData.RecentlyAddedAlbums)
                    RecentlyAddedAlbums.Add(album);
            }

            if (HomeData?.RandomAlbums != null)
            {
                foreach (var album in HomeData.RandomAlbums)
                    RandomAlbums.Add(album);
            }

            if (HomeData?.FeaturedAlbums != null)
            {
                foreach (var album in HomeData.FeaturedAlbums)
                    FeaturedAlbums.Add(album);
            }
        }
        finally
        {
            IsLoading = false;
        }
    }

    public void PlayAlbum(AureliaCore.Album album)
    {
        _ = PlayAlbumInternal(album);
    }

    private async Task PlayAlbumInternal(AureliaCore.Album album)
    {
        var songs = await _apiService.GetSongsAsync(albumId: album.id);
        if (songs.Count > 0)
        {
            _playerService.SetQueue(songs, 0);
            await _playerService.PlayAsync();
        }
    }

    public void PlaySong(AureliaCore.Song song, ObservableCollection<AureliaCore.Song> sourceList)
    {
        var index = sourceList.IndexOf(song);
        _playerService.SetQueue(sourceList.ToList(), index >= 0 ? index : 0);
        _ = _playerService.PlayAsync();
    }

    protected void OnPropertyChanged([CallerMemberName] string? propertyName = null)
    {
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
    }
}
