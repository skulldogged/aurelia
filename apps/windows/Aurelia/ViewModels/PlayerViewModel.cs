using System.ComponentModel;
using System.Runtime.CompilerServices;
using System.Windows.Input;
using Aurelia.Models;
using Aurelia.Services;

namespace Aurelia.ViewModels;

public class PlayerViewModel : INotifyPropertyChanged
{
    private readonly PlayerService _playerService;
    private readonly ApiService _apiService;

    private Song? _currentSong;
    private PlaybackState _playbackState;
    private TimeSpan _position;
    private TimeSpan _duration;
    private RepeatMode _repeatMode;
    private bool _shuffleEnabled;
    private bool _isPlaying;

    public event PropertyChangedEventHandler? PropertyChanged;

    public Song? CurrentSong
    {
        get => _currentSong;
        set { _currentSong = value; OnPropertyChanged(); OnPropertyChanged(nameof(HasCurrentSong)); }
    }

    public bool HasCurrentSong => CurrentSong != null;

    public PlaybackState PlaybackState
    {
        get => _playbackState;
        set { _playbackState = value; OnPropertyChanged(); IsPlaying = value == PlaybackState.Playing; }
    }

    public TimeSpan Position
    {
        get => _position;
        set { _position = value; OnPropertyChanged(); OnPropertyChanged(nameof(PositionText)); }
    }

    public TimeSpan Duration
    {
        get => _duration;
        set { _duration = value; OnPropertyChanged(); OnPropertyChanged(nameof(DurationText)); }
    }

    public bool IsPlaying
    {
        get => _isPlaying;
        set { _isPlaying = value; OnPropertyChanged(); }
    }

    public RepeatMode RepeatMode
    {
        get => _repeatMode;
        set { _repeatMode = value; OnPropertyChanged(); OnPropertyChanged(nameof(RepeatModeText)); }
    }

    public bool ShuffleEnabled
    {
        get => _shuffleEnabled;
        set { _shuffleEnabled = value; OnPropertyChanged(); }
    }

    public string PositionText => FormatTime(Position);
    public string DurationText => FormatTime(Duration);
    public string RepeatModeText => RepeatMode switch
    {
        RepeatMode.Off => "Off",
        RepeatMode.All => "All",
        RepeatMode.One => "One",
        _ => "Off"
    };

    public ICommand PlayPauseCommand { get; }
    public ICommand NextCommand { get; }
    public ICommand PreviousCommand { get; }
    public ICommand ToggleShuffleCommand { get; }
    public ICommand CycleRepeatCommand { get; }

    public PlayerViewModel(PlayerService playerService, ApiService apiService)
    {
        _playerService = playerService;
        _apiService = apiService;

        PlayPauseCommand = new RelayCommand(_ => _playerService.TogglePlayPause());
        NextCommand = new RelayCommand(_ => _playerService.Next());
        PreviousCommand = new RelayCommand(_ => _playerService.Previous());
        ToggleShuffleCommand = new RelayCommand(_ => 
        {
            _playerService.ToggleShuffle();
            ShuffleEnabled = _playerService.ShuffleEnabled;
        });
        CycleRepeatCommand = new RelayCommand(_ =>
        {
            _playerService.CycleRepeatMode();
            RepeatMode = _playerService.RepeatMode;
        });

        _playerService.CurrentSongChanged += OnCurrentSongChanged;
        _playerService.PlaybackStateChanged += OnPlaybackStateChanged;
        _playerService.PositionChanged += OnPositionChanged;
    }

    private void OnCurrentSongChanged(object? sender, Song? song)
    {
        CurrentSong = song;
        if (song != null)
        {
            Duration = TimeSpan.FromSeconds(song.duration ?? 0);
        }
    }

    private void OnPlaybackStateChanged(object? sender, PlaybackState state)
    {
        PlaybackState = state;
    }

    private void OnPositionChanged(object? sender, TimeSpan position)
    {
        Position = position;
    }

    public void Seek(double percentage)
    {
        var newPosition = TimeSpan.FromTicks((long)(Duration.Ticks * percentage));
        _playerService.Seek(newPosition);
    }

    private static string FormatTime(TimeSpan time)
    {
        if (time.TotalHours >= 1)
        {
            return time.ToString(@"h\:mm\:ss");
        }
        return time.ToString(@"m\:ss");
    }

    protected void OnPropertyChanged([CallerMemberName] string? propertyName = null)
    {
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
    }
}

public class RelayCommand : ICommand
{
    private readonly Action<object?> _execute;
    private readonly Func<object?, bool>? _canExecute;
    private bool _canExecuteValue = true;

    public event EventHandler? CanExecuteChanged;

    public RelayCommand(Action<object?> execute, Func<object?, bool>? canExecute = null)
    {
        _execute = execute;
        _canExecute = canExecute;
    }

    public bool CanExecute(object? parameter)
    {
        if (_canExecute != null)
        {
            _canExecuteValue = _canExecute(parameter);
        }
        return _canExecuteValue;
    }

    public void Execute(object? parameter) => _execute(parameter);

    public void RaiseCanExecuteChanged()
    {
        CanExecuteChanged?.Invoke(this, EventArgs.Empty);
    }
}
