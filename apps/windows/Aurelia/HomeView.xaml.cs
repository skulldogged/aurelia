using Aurelia.ViewModels;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media.Imaging;

namespace Aurelia;

public sealed partial class HomeView : Page {
  private readonly HomeViewModel _viewModel;
  private int _featuredIndex;

  public HomeView() {
    this.InitializeComponent();

    App app = App.Current;
    _viewModel = new HomeViewModel(app.ApiService, app.PlayerService);

    this.Loaded += HomeView_Loaded;
  }

  private async void HomeView_Loaded(object sender, RoutedEventArgs e) {
    GreetingText.Text = _viewModel.Greeting;
    LoadingIndicator.IsActive = true;

    await _viewModel.LoadAsync();

    // Featured album hero
    _featuredIndex = 0;
    UpdateFeaturedAlbum();

    // Bind carousels
    MostPlayedList.ItemsSource = _viewModel.MostPlayed;
    RecentlyPlayedList.ItemsSource = _viewModel.RecentlyPlayed;
    RecentlyAddedList.ItemsSource = _viewModel.RecentlyAddedAlbums;
    RandomAlbumsList.ItemsSource = _viewModel.RandomAlbums;

    // Show/hide sections based on data
    FeaturedSection.Visibility = _viewModel.FeaturedAlbums.Count > 0
        ? Visibility.Visible : Visibility.Collapsed;
    MostPlayedSection.Visibility = _viewModel.MostPlayed.Count > 0
        ? Visibility.Visible : Visibility.Collapsed;
    RecentlyPlayedSection.Visibility = _viewModel.RecentlyPlayed.Count > 0
        ? Visibility.Visible : Visibility.Collapsed;
    RecentlyAddedSection.Visibility = _viewModel.RecentlyAddedAlbums.Count > 0
        ? Visibility.Visible : Visibility.Collapsed;
    RandomAlbumsSection.Visibility = _viewModel.RandomAlbums.Count > 0
        ? Visibility.Visible : Visibility.Collapsed;

    bool hasAnyData = _viewModel.FeaturedAlbums.Count > 0
        || _viewModel.MostPlayed.Count > 0
        || _viewModel.RecentlyPlayed.Count > 0
        || _viewModel.RecentlyAddedAlbums.Count > 0
        || _viewModel.RandomAlbums.Count > 0;

    EmptyState.Visibility = hasAnyData ? Visibility.Collapsed : Visibility.Visible;

    LoadingIndicator.IsActive = false;
  }

  private void UpdateFeaturedAlbum() {
    if (_viewModel.FeaturedAlbums.Count == 0) return;

    Album album = _viewModel.FeaturedAlbums[_featuredIndex];
    FeaturedAlbumName.Text = album.name;
    FeaturedArtistName.Text = album.artist;

    if (!string.IsNullOrEmpty(album.albumArtUrl)) {
      string sep = album.albumArtUrl.Contains('?') ? "&" : "?";
      string artUrl = $"{album.albumArtUrl}{sep}MaxWidth=240&Quality=80";
      string backdropUrl = $"{album.albumArtUrl}{sep}MaxWidth=600&Quality=60";
      FeaturedArt.ImageSource = new BitmapImage(new System.Uri(artUrl));
      FeaturedBackdrop.ImageSource = new BitmapImage(new System.Uri(backdropUrl));
    } else {
      FeaturedArt.ImageSource = null;
      FeaturedBackdrop.ImageSource = null;
    }
  }

  private void FeaturedPlay_Click(object sender, RoutedEventArgs e) {
    if (_viewModel.FeaturedAlbums.Count > 0) {
      _viewModel.PlayAlbum(_viewModel.FeaturedAlbums[_featuredIndex]);
    }
  }

  private void FeaturedNext_Click(object sender, RoutedEventArgs e) {
    if (_viewModel.FeaturedAlbums.Count == 0) return;
    _featuredIndex = (_featuredIndex + 1) % _viewModel.FeaturedAlbums.Count;
    UpdateFeaturedAlbum();
  }

  private void Album_Click(object sender, RoutedEventArgs e) {
    if (sender is Button button && button.Tag is AureliaCore.Album album) {
      _viewModel.PlayAlbum(album);
    }
  }

  private void MostPlayedSong_Click(object sender, RoutedEventArgs e) {
    if (sender is Button button && button.Tag is AureliaCore.Song song) {
      _viewModel.PlaySong(song, _viewModel.MostPlayed);
    }
  }

  private void RecentSong_Click(object sender, RoutedEventArgs e) {
    if (sender is Button button && button.Tag is AureliaCore.Song song) {
      _viewModel.PlaySong(song, _viewModel.RecentlyPlayed);
    }
  }
}
