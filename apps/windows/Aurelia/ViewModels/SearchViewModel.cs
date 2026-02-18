using Aurelia.Services;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Runtime.CompilerServices;

namespace Aurelia.ViewModels;

public class SearchViewModel : INotifyPropertyChanged {
  private readonly ApiService _apiService;
  private readonly PlayerService _playerService;

  private bool _isLoading;
  private string _searchQuery = string.Empty;

  public event PropertyChangedEventHandler? PropertyChanged;

  public bool IsLoading {
    get => _isLoading;
    set { _isLoading = value; OnPropertyChanged(); }
  }

  public string SearchQuery {
    get => _searchQuery;
    set { _searchQuery = value; OnPropertyChanged(); }
  }

  public ObservableCollection<Song> SearchResults { get; } = [];

  public SearchViewModel(ApiService apiService, PlayerService playerService) {
    _apiService = apiService;
    _playerService = playerService;
  }

  public async Task SearchAsync(string query) {
    if (string.IsNullOrWhiteSpace(query)) {
      SearchResults.Clear();
      return;
    }

    IsLoading = true;
    try {
      SearchResults.Clear();
      List<Song> results = await _apiService.SearchAsync(query);
      foreach (Song song in results)
        SearchResults.Add(song);
    } finally {
      IsLoading = false;
    }
  }

  public void PlaySong(Song song) {
    int index = SearchResults.IndexOf(song);
    _playerService.SetQueue(SearchResults.ToList(), index >= 0 ? index : 0);
    _ = _playerService.PlayAsync();
  }

  protected void OnPropertyChanged([CallerMemberName] string? propertyName = null) {
    PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
  }
}
