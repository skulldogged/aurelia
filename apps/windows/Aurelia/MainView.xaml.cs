using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media.Imaging;
using Aurelia.Models;
using Aurelia.Services;

namespace Aurelia;

public sealed partial class MainView : Page
{
    private readonly PlayerService _playerService;
    private readonly ApiService _apiService;

    public MainView()
    {
        this.InitializeComponent();

        var app = App.Current;
        _playerService = app.PlayerService;
        _apiService = app.ApiService;

        _playerService.CurrentSongChanged += OnCurrentSongChanged;
        _playerService.PlaybackStateChanged += OnPlaybackStateChanged;

        MiniPlayer.Translation = new System.Numerics.Vector3(0, 0, 16);
        UpdateShuffleIcon();
        UpdateRepeatIcon();

        NavView.SelectedItem = NavView.MenuItems[0];
        ContentFrame.Navigate(typeof(HomeView));
    }

    private void NavView_SelectionChanged(NavigationView sender, object args)
    {
        if (sender.SelectedItem is NavigationViewItem item && item.Tag is string tag)
        {
            Type? pageType = tag switch
            {
                "Home" => typeof(HomeView),
                "Library" => typeof(LibraryView),
                "Search" => typeof(SearchView),
                "Settings" => typeof(SettingsView),
                _ => null
            };

            if (pageType != null)
            {
                ContentFrame.Navigate(pageType);
            }
        }
    }

    private void OnCurrentSongChanged(object? sender, Song? song)
    {
        if (song != null)
        {
            MiniPlayer.Visibility = Visibility.Visible;
            MiniPlayerTitle.Text = song.name ?? "Unknown";
            MiniPlayerArtist.Text = song.artists?.FirstOrDefault() ?? "Unknown Artist";

            if (!string.IsNullOrEmpty(song.albumArtUrl))
            {
                var sep = song.albumArtUrl.Contains('?') ? "&" : "?";
                MiniPlayerArtBrush.ImageSource = new BitmapImage(new Uri($"{song.albumArtUrl}{sep}MaxWidth=96&Quality=80"));
            }
            else
            {
                MiniPlayerArtBrush.ImageSource = null;
            }

            UpdatePlayPauseIcon();
            UpdateFavoriteIcon();
        }
        else
        {
            MiniPlayer.Visibility = Visibility.Collapsed;
        }
    }

    private void OnPlaybackStateChanged(object? sender, PlaybackState state)
    {
        UpdatePlayPauseIcon();
    }

    private void UpdatePlayPauseIcon()
    {
        var isPlaying = _playerService.State == PlaybackState.Playing;
        MiniPlayPauseIcon.Glyph = isPlaying ? "\uE769" : "\uE768"; // Pause : Play
    }

    private Microsoft.UI.Xaml.Media.Brush GetBrush(string key) =>
        (Microsoft.UI.Xaml.Media.Brush)Application.Current.Resources[key];

    private void UpdateShuffleIcon()
    {
        MiniShuffleIcon.Foreground = _playerService.ShuffleEnabled
            ? GetBrush("AccentTextFillColorPrimaryBrush")
            : GetBrush("TextFillColorSecondaryBrush");
    }

    private void UpdateRepeatIcon()
    {
        MiniRepeatIcon.Glyph = _playerService.RepeatMode == RepeatMode.One ? "\uE8ED" : "\uE8EE";
        MiniRepeatIcon.Foreground = _playerService.RepeatMode != RepeatMode.Off
            ? GetBrush("AccentTextFillColorPrimaryBrush")
            : GetBrush("TextFillColorSecondaryBrush");
    }

    // Center tapped -> open full player
    private void MiniPlayerCenter_Click(object sender, RoutedEventArgs e)
    {
        PlayerOverlay.Visibility = Visibility.Visible;
        MiniPlayer.Visibility = Visibility.Collapsed;
        PlayerControl.OnShown();
    }

    private void PlayerControl_DismissRequested(object? sender, EventArgs e)
    {
        PlayerOverlay.Visibility = Visibility.Collapsed;
        if (_playerService.CurrentSong != null)
        {
            MiniPlayer.Visibility = Visibility.Visible;
        }
    }

    // Left controls
    private void MiniPlayerShuffle_Click(object sender, RoutedEventArgs e)
    {
        _playerService.ToggleShuffle();
        UpdateShuffleIcon();
    }

    private void MiniPlayerPrevious_Click(object sender, RoutedEventArgs e)
    {
        _playerService.Previous();
    }

    private void MiniPlayerPlayPause_Click(object sender, RoutedEventArgs e)
    {
        _playerService.TogglePlayPause();
    }

    private void MiniPlayerNext_Click(object sender, RoutedEventArgs e)
    {
        _playerService.Next();
    }

    private void MiniPlayerRepeat_Click(object sender, RoutedEventArgs e)
    {
        _playerService.CycleRepeatMode();
        UpdateRepeatIcon();
    }

    private void UpdateFavoriteIcon()
    {
        var song = _playerService.CurrentSong;
        var isFavorite = song?.isFavorite ?? false;
        Logger.Info($"[UpdateFavoriteIcon] Song: {song?.name}, isFavorite: {song?.isFavorite}, resolved: {isFavorite}");
        MiniFavoriteIcon.Glyph = isFavorite ? "\uEB52" : "\uEB51"; // FilledHeart : EmptyHeart
        MiniFavoriteIcon.Foreground = isFavorite
            ? GetBrush("AccentTextFillColorPrimaryBrush")
            : GetBrush("TextFillColorSecondaryBrush");
    }

    // Right actions
    private async void MiniPlayerFavorite_Click(object sender, RoutedEventArgs e)
    {
        var song = _playerService.CurrentSong;
        if (song == null)
        {
            Logger.Info("[MiniPlayerFavorite_Click] No current song");
            return;
        }

        var currentFavorite = song.isFavorite ?? false;
        var targetFavorite = !currentFavorite; // What we WANT it to be
        Logger.Info($"[MiniPlayerFavorite_Click] Toggling favorite for {song.id}, current: {currentFavorite}, target: {targetFavorite}");

        try
        {
            var newFavorite = await _apiService.ToggleFavoriteAsync(song.id, targetFavorite);
            Logger.Info($"[MiniPlayerFavorite_Click] Result: {newFavorite}");

            // Update the song in the queue so subsequent toggles work correctly
            _playerService.UpdateCurrentSongFavorite(newFavorite);
        }
        catch (Exception ex)
        {
            Logger.Error($"[MiniPlayerFavorite_Click] Error: {ex.Message}");
        }
    }
}
