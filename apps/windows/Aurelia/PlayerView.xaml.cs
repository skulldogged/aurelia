using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media.Imaging;
using Aurelia.Models;
using Aurelia.Services;

namespace Aurelia;

public sealed partial class PlayerView : UserControl
{
    private PlayerService? _playerService;
    private bool _isDraggingSlider;
    private bool _isProgrammaticUpdate;

    private DispatcherTimer? _seekTimer;
    private DateTimeOffset _lastPositionTime;
    private double _lastPositionSeconds;

    public event EventHandler? DismissRequested;

    public PlayerView()
    {
        this.InitializeComponent();
        this.Loaded += PlayerView_Loaded;
        this.Unloaded += PlayerView_Unloaded;
    }

    private void PlayerView_Loaded(object sender, RoutedEventArgs e)
    {
        var app = App.Current;
        _playerService = app.PlayerService;

        _playerService.CurrentSongChanged += OnCurrentSongChanged;
        _playerService.PlaybackStateChanged += OnPlaybackStateChanged;
        _playerService.PositionChanged += OnPositionChanged;
        _playerService.VolumeChanged += OnVolumeChanged;

        VolumeSlider.Value = _playerService.Volume * 100;
        UpdateVolumeUI(_playerService.Volume);

        _seekTimer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(50) };
        _seekTimer.Tick += SeekTimer_Tick;

        UpdateUI();
        if (_playerService.State == PlaybackState.Playing)
            _seekTimer.Start();
    }

    private void PlayerView_Unloaded(object sender, RoutedEventArgs e)
    {
        _seekTimer?.Stop();
        if (_playerService != null)
        {
            _playerService.CurrentSongChanged -= OnCurrentSongChanged;
            _playerService.PlaybackStateChanged -= OnPlaybackStateChanged;
            _playerService.PositionChanged -= OnPositionChanged;
            _playerService.VolumeChanged -= OnVolumeChanged;
        }
    }

    public void OnShown()
    {
        if (_playerService != null)
        {
            UpdateUI();
            if (_playerService.State == PlaybackState.Playing)
                _seekTimer?.Start();
        }
    }

    private void OnCurrentSongChanged(object? sender, Song? song)
    {
        UpdateUI();
    }

    private void OnPlaybackStateChanged(object? sender, PlaybackState state)
    {
        UpdatePlayPauseButton();
        if (state == PlaybackState.Playing)
            _seekTimer?.Start();
        else
            _seekTimer?.Stop();
    }

    private void OnPositionChanged(object? sender, TimeSpan position)
    {
        _lastPositionSeconds = position.TotalSeconds;
        _lastPositionTime = DateTimeOffset.UtcNow;
    }

    private void SeekTimer_Tick(object? sender, object e)
    {
        if (_isDraggingSlider) return;

        var elapsed = (DateTimeOffset.UtcNow - _lastPositionTime).TotalSeconds;
        var interpolated = _lastPositionSeconds + elapsed;

        if (ProgressSlider.Maximum > 0)
        {
            interpolated = Math.Min(interpolated, ProgressSlider.Maximum);
            _isProgrammaticUpdate = true;
            ProgressSlider.Value = interpolated;
            _isProgrammaticUpdate = false;
        }

        PositionText.Text = FormatTime(TimeSpan.FromSeconds(interpolated));
    }

    private void UpdateUI()
    {
        if (_playerService == null) return;

        var song = _playerService.CurrentSong;
        var duration = _playerService.Duration;
        var position = _playerService.Position;

        _lastPositionSeconds = position.TotalSeconds;
        _lastPositionTime = DateTimeOffset.UtcNow;

        if (song != null)
        {
            SongTitle.Text = song.name ?? "Unknown";
            SongArtist.Text = song.artists?.FirstOrDefault() ?? "Unknown Artist";
            SongAlbum.Text = song.album ?? "";
            DurationText.Text = FormatTime(duration);
            ProgressSlider.Maximum = Math.Max(1, duration.TotalSeconds);

            _isProgrammaticUpdate = true;
            ProgressSlider.Value = position.TotalSeconds;
            _isProgrammaticUpdate = false;

            PositionText.Text = FormatTime(position);

            if (!string.IsNullOrEmpty(song.albumArtUrl))
            {
                var sep = song.albumArtUrl.Contains('?') ? "&" : "?";
                AlbumArtBrush.ImageSource = new BitmapImage(new Uri($"{song.albumArtUrl}{sep}MaxWidth=400&Quality=80"));
            }
            else
            {
                AlbumArtBrush.ImageSource = null;
            }
        }
        else
        {
            SongTitle.Text = "Not Playing";
            SongArtist.Text = "";
            SongAlbum.Text = "";
            DurationText.Text = "0:00";
            ProgressSlider.Maximum = 100;

            _isProgrammaticUpdate = true;
            ProgressSlider.Value = 0;
            _isProgrammaticUpdate = false;

            PositionText.Text = "0:00";
            AlbumArtBrush.ImageSource = null;
        }

        UpdatePlayPauseButton();
    }

    private void UpdatePlayPauseButton()
    {
        if (_playerService == null) return;
        var isPlaying = _playerService.State == PlaybackState.Playing;
        PlayPauseButton.Content = new SymbolIcon(isPlaying ? Symbol.Pause : Symbol.Play);
    }

    private static string FormatTime(TimeSpan time)
    {
        return time.TotalHours >= 1 ? time.ToString(@"h\:mm\:ss") : time.ToString(@"m\:ss");
    }

    private void Dismiss_Click(object sender, RoutedEventArgs e)
    {
        DismissRequested?.Invoke(this, EventArgs.Empty);
    }

    private void PlayPause_Click(object sender, RoutedEventArgs e) => _playerService?.TogglePlayPause();
    private void Previous_Click(object sender, RoutedEventArgs e) => _playerService?.Previous();
    private void Next_Click(object sender, RoutedEventArgs e) => _playerService?.Next();
    private void Shuffle_Click(object sender, RoutedEventArgs e) => _playerService?.ToggleShuffle();
    private void Repeat_Click(object sender, RoutedEventArgs e) => _playerService?.CycleRepeatMode();

    private void ProgressSlider_PointerPressed(object sender, PointerRoutedEventArgs e)
    {
        _isDraggingSlider = true;
    }

    private void ProgressSlider_PointerCaptureLost(object sender, PointerRoutedEventArgs e)
    {
        if (_isDraggingSlider)
        {
            _isDraggingSlider = false;
            var newPosition = TimeSpan.FromSeconds(ProgressSlider.Value);
            _playerService?.Seek(newPosition);
            _lastPositionSeconds = ProgressSlider.Value;
            _lastPositionTime = DateTimeOffset.UtcNow;
        }
    }

    private void ProgressSlider_ValueChanged(object sender, RangeBaseValueChangedEventArgs e)
    {
        if (_isDraggingSlider)
        {
            PositionText.Text = FormatTime(TimeSpan.FromSeconds(ProgressSlider.Value));
        }
        else if (!_isProgrammaticUpdate && _playerService != null)
        {
            var newPosition = TimeSpan.FromSeconds(ProgressSlider.Value);
            _playerService.Seek(newPosition);
            _lastPositionSeconds = ProgressSlider.Value;
            _lastPositionTime = DateTimeOffset.UtcNow;
        }
    }

    private double _volumeBeforeMute = 1.0;

    private void OnVolumeChanged(object? sender, double volume)
    {
        VolumeSlider.Value = volume * 100;
        UpdateVolumeUI(volume);
    }

    private void VolumeSlider_ValueChanged(object sender, RangeBaseValueChangedEventArgs e)
    {
        if (_playerService == null) return;
        var volume = e.NewValue / 100.0;
        _playerService.Volume = volume;
        UpdateVolumeUI(volume);
    }

    private void Mute_Click(object sender, RoutedEventArgs e)
    {
        if (_playerService == null) return;
        if (_playerService.Volume > 0)
        {
            _volumeBeforeMute = _playerService.Volume;
            _playerService.Volume = 0;
        }
        else
        {
            _playerService.Volume = _volumeBeforeMute > 0 ? _volumeBeforeMute : 1.0;
        }
    }

    private void UpdateVolumeUI(double volume)
    {
        VolumeText.Text = $"{(int)(volume * 100)}%";
        VolumeIcon.Glyph = volume switch
        {
            0 => "\uE992",
            <= 0.33 => "\uE993",
            <= 0.66 => "\uE994",
            _ => "\uE995"
        };
    }
}
