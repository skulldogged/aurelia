using System.IO;
using System.Globalization;
using System.Threading;
using Aurelia.Models;
using Windows.System;
using TimeSpan = System.TimeSpan;

namespace Aurelia.Services;

public class PlayerService
{
    private readonly ApiService _apiService;
    private readonly string _settingsPath;
    private readonly List<Song> _queue = [];
    private readonly HashSet<string> _forceLossyItemIds = new(StringComparer.Ordinal);
    private readonly System.Timers.Timer _progressTimer;
    private readonly System.Timers.Timer _mediaControlsTimer;
    private DispatcherQueue? _dispatcherQueue;
    private int _currentIndex = -1;
    private int _playbackRequestId;
    private int _progressPollInFlight;
    private int _mediaControlsPollInFlight;
    private double _volume = 1.0;

    public event EventHandler<Song?>? CurrentSongChanged;
    public event EventHandler<PlaybackState>? PlaybackStateChanged;
    public event EventHandler<TimeSpan>? PositionChanged;
    public event EventHandler<List<Song>>? QueueChanged;
    public event EventHandler<double>? VolumeChanged;

    public Song? CurrentSong => _currentIndex >= 0 && _currentIndex < _queue.Count ? _queue[_currentIndex] : null;
    public IReadOnlyList<Song> Queue => _queue;
    public int CurrentIndex => _currentIndex;
    public PlaybackState State { get; private set; } = PlaybackState.Stopped;
    public TimeSpan Position { get; private set; }
    public TimeSpan Duration { get; private set; }

    public RepeatMode RepeatMode { get; set; } = RepeatMode.Off;
    public bool ShuffleEnabled { get; set; }

    public double Volume
    {
        get => _volume;
        set
        {
            var clamped = Math.Clamp(value, 0.0, 1.0);
            _volume = clamped;
            SaveVolume(clamped);
            VolumeChanged?.Invoke(this, clamped);
            _ = SetRustVolumeAsync(clamped);
        }
    }

    public PlayerService(ApiService apiService, string appDataDir)
    {
        _apiService = apiService;
        _settingsPath = Path.Combine(appDataDir, "player-settings.json");
        _dispatcherQueue = DispatcherQueue.GetForCurrentThread();

        _progressTimer = new System.Timers.Timer(100);
        _progressTimer.Elapsed += OnProgressTimerElapsed;
        _progressTimer.AutoReset = true;
        _mediaControlsTimer = new System.Timers.Timer(150);
        _mediaControlsTimer.Elapsed += OnMediaControlsTimerElapsed;
        _mediaControlsTimer.AutoReset = true;
        _mediaControlsTimer.Start();

        LoadVolume();
        _ = InitializeRustAudioAsync();
        Logger.Info("PlayerService initialized (Rust backend)");
    }

    private async Task InitializeRustAudioAsync()
    {
        try
        {
            await AureliaCore.AureliaCore.AudioInitPlayer();
            await SetRustVolumeAsync(_volume);
        }
        catch (Exception ex)
        {
            Logger.Error($"[PlayerService] Failed to initialize Rust audio backend: {ex.Message}");
        }
    }

    private void RunOnUi(Action action)
    {
        if (_dispatcherQueue == null)
        {
            _dispatcherQueue = DispatcherQueue.GetForCurrentThread();
        }

        if (_dispatcherQueue != null)
        {
            _dispatcherQueue.TryEnqueue(() => action());
        }
        else
        {
            action();
        }
    }

    private void OnProgressTimerElapsed(object? sender, System.Timers.ElapsedEventArgs e)
    {
        _ = PollPlaybackStateAsync();
    }

    private void OnMediaControlsTimerElapsed(object? sender, System.Timers.ElapsedEventArgs e)
    {
        _ = PollMediaControlsAsync();
    }

    private async Task PollPlaybackStateAsync()
    {
        if (State != PlaybackState.Playing)
        {
            return;
        }

        if (Interlocked.Exchange(ref _progressPollInFlight, 1) == 1)
        {
            return;
        }

        try
        {
            var positionSeconds = await AureliaCore.AureliaCore.AudioGetPositionSecs();
            Position = TimeSpan.FromSeconds(Math.Max(positionSeconds, 0));
            RunOnUi(() => PositionChanged?.Invoke(this, Position));

            var finished = await AureliaCore.AureliaCore.AudioIsFinishedPlayer();
            if (finished)
            {
                Logger.Info("[PlayerService] Rust backend reported track finished");
                RunOnUi(Next);
            }
        }
        catch (Exception ex)
        {
            Logger.Error($"[PlayerService] Progress poll error: {ex.Message}");
        }
        finally
        {
            Interlocked.Exchange(ref _progressPollInFlight, 0);
        }
    }

    private async Task PollMediaControlsAsync()
    {
        if (Interlocked.Exchange(ref _mediaControlsPollInFlight, 1) == 1)
        {
            return;
        }

        try
        {
            for (var i = 0; i < 8; i++)
            {
                var encoded = AureliaCore.AureliaCore.MediaControlsPopEvent();
                if (string.IsNullOrWhiteSpace(encoded))
                {
                    break;
                }

                await HandleMediaControlEventAsync(encoded);
            }
        }
        catch (Exception ex)
        {
            Logger.Error($"[PlayerService] Media control poll error: {ex.Message}");
        }
        finally
        {
            Interlocked.Exchange(ref _mediaControlsPollInFlight, 0);
        }
    }

    private async Task HandleMediaControlEventAsync(string encoded)
    {
        try
        {
            if (string.Equals(encoded, "play", StringComparison.OrdinalIgnoreCase))
            {
                if (State == PlaybackState.Paused)
                {
                    Resume();
                }
                else if (State == PlaybackState.Stopped)
                {
                    await PlayAsync();
                }
                return;
            }

            if (string.Equals(encoded, "pause", StringComparison.OrdinalIgnoreCase))
            {
                Pause();
                return;
            }

            if (string.Equals(encoded, "toggle", StringComparison.OrdinalIgnoreCase))
            {
                TogglePlayPause();
                return;
            }

            if (string.Equals(encoded, "next", StringComparison.OrdinalIgnoreCase))
            {
                Next();
                return;
            }

            if (string.Equals(encoded, "previous", StringComparison.OrdinalIgnoreCase))
            {
                Previous();
                return;
            }

            if (string.Equals(encoded, "stop", StringComparison.OrdinalIgnoreCase))
            {
                Stop();
                return;
            }

            if (encoded.StartsWith("seek_delta:", StringComparison.OrdinalIgnoreCase) &&
                double.TryParse(
                    encoded.Substring("seek_delta:".Length),
                    NumberStyles.Float,
                    CultureInfo.InvariantCulture,
                    out var delta))
            {
                var target = Position + TimeSpan.FromSeconds(delta);
                if (target < TimeSpan.Zero)
                {
                    target = TimeSpan.Zero;
                }
                if (Duration > TimeSpan.Zero && target > Duration)
                {
                    target = Duration;
                }
                Seek(target);
                return;
            }

            if (encoded.StartsWith("set_position:", StringComparison.OrdinalIgnoreCase) &&
                double.TryParse(
                    encoded.Substring("set_position:".Length),
                    NumberStyles.Float,
                    CultureInfo.InvariantCulture,
                    out var absolute))
            {
                var target = TimeSpan.FromSeconds(Math.Max(absolute, 0));
                if (Duration > TimeSpan.Zero && target > Duration)
                {
                    target = Duration;
                }
                Seek(target);
                return;
            }
        }
        catch (Exception ex)
        {
            Logger.Error($"[PlayerService] Failed handling media event '{encoded}': {ex.Message}");
        }
    }

    public void SetQueue(List<Song> songs, int startIndex = 0)
    {
        _queue.Clear();
        _queue.AddRange(songs);
        _currentIndex = startIndex;
        QueueChanged?.Invoke(this, new List<Song>(_queue));

        if (_currentIndex >= 0 && _currentIndex < _queue.Count)
        {
            CurrentSongChanged?.Invoke(this, CurrentSong);
            _ = RefreshFavoriteStatusAsync();
        }
    }

    public async Task RefreshFavoriteStatusAsync()
    {
        if (CurrentSong == null) return;

        try
        {
            var favoriteIds = await _apiService.GetFavoriteIdsAsync();
            var isFavorite = favoriteIds.Contains(CurrentSong.id);

            if (CurrentSong.isFavorite != isFavorite)
            {
                UpdateCurrentSongFavorite(isFavorite);
            }
        }
        catch (Exception ex)
        {
            Logger.Error($"[PlayerService] Failed to refresh favorite status: {ex.Message}");
        }
    }

    public void AddToQueue(Song song)
    {
        _queue.Add(song);
        QueueChanged?.Invoke(this, new List<Song>(_queue));
    }

    public void ClearQueue()
    {
        Stop();
        _queue.Clear();
        _currentIndex = -1;
        QueueChanged?.Invoke(this, new List<Song>());
    }

    public async Task PlayAsync()
    {
        var requestId = Interlocked.Increment(ref _playbackRequestId);
        var song = CurrentSong;
        if (song == null)
        {
            Logger.Error("[PlayerService] PlayAsync called but CurrentSong is null");
            return;
        }

        var token = _apiService.GetPlaybackToken();
        if (string.IsNullOrWhiteSpace(token))
        {
            Logger.Error("[PlayerService] Missing playback token");
            return;
        }

        Logger.Info($"[PlayerService] PlayAsync starting for song: {song.name} (id: {song.id}, container: {song.container}, request={requestId})");

        try
        {
            var streamUrl = await _apiService.GetStreamUrlAsync(song.id, song.container);
            if (_forceLossyItemIds.Contains(song.id))
            {
                Logger.Info($"[PlayerService] Item {song.id} is marked as lossy-only fallback; skipping lossless attempts");
                streamUrl = await _apiService.GetFallbackStreamUrlAsync(song.id, song.container);
            }

            if (requestId != _playbackRequestId)
            {
                Logger.Info($"[PlayerService] Ignoring stale playback request {requestId}");
                return;
            }

            Logger.Info($"[PlayerService] Got stream URL: {(string.IsNullOrEmpty(streamUrl) ? "(empty)" : streamUrl.Substring(0, Math.Min(100, streamUrl.Length)) + "...")}");

            if (string.IsNullOrEmpty(streamUrl))
            {
                Logger.Error($"[PlayerService] GetStreamUrlAsync returned empty for song: {song.name} (id: {song.id})");
                return;
            }

            var started = await TryStartRustPlaybackAsync(streamUrl, token, song, requestId, "primary");
            var startedViaLossy = false;

            if (!started && !_forceLossyItemIds.Contains(song.id))
            {
                var losslessFallbacks = await _apiService.GetLosslessFallbackStreamUrlsAsync(song.id, song.container);
                if (requestId != _playbackRequestId)
                {
                    Logger.Info($"[PlayerService] Ignoring stale lossless fallback request {requestId}");
                    return;
                }

                var attemptedLosslessUrl = string.Empty;
                if (losslessFallbacks.Count > 0)
                {
                    var losslessUrl = losslessFallbacks[0];
                    if (!string.IsNullOrEmpty(losslessUrl) &&
                        !string.Equals(losslessUrl, streamUrl, StringComparison.OrdinalIgnoreCase))
                    {
                        Logger.Info("[PlayerService] Primary stream did not start; trying lossless fallback");
                        attemptedLosslessUrl = losslessUrl;
                        started = await TryStartRustPlaybackAsync(losslessUrl, token, song, requestId, "lossless-fallback");
                    }
                }

                if (!started && requestId == _playbackRequestId)
                {
                    var fallbackUrl = await _apiService.GetFallbackStreamUrlAsync(song.id, song.container);
                    if (requestId != _playbackRequestId)
                    {
                        Logger.Info($"[PlayerService] Ignoring stale lossy fallback request {requestId}");
                        return;
                    }

                    if (!string.IsNullOrEmpty(fallbackUrl) &&
                        !string.Equals(fallbackUrl, streamUrl, StringComparison.OrdinalIgnoreCase) &&
                        !string.Equals(fallbackUrl, attemptedLosslessUrl, StringComparison.OrdinalIgnoreCase))
                    {
                        Logger.Info("[PlayerService] Lossless fallback failed; trying lossy fallback stream URL");
                        started = await TryStartRustPlaybackAsync(fallbackUrl, token, song, requestId, "lossy-fallback");
                        startedViaLossy = started;
                    }
                }
            }

            if (!started)
            {
                Logger.Error($"[PlayerService] Playback failed to start for: {song.name} (id: {song.id})");
                SetState(PlaybackState.Stopped);
                return;
            }

            if (startedViaLossy)
            {
                _forceLossyItemIds.Add(song.id);
            }

            Position = TimeSpan.Zero;
            Duration = TimeSpan.FromSeconds(song.duration ?? 0);

            try
            {
                AureliaCore.AureliaCore.MediaControlsUpdateNowPlaying(
                    song.name ?? "Unknown",
                    song.artists?.FirstOrDefault(),
                    song.album,
                    song.duration,
                    song.albumArtUrl);
                AureliaCore.AureliaCore.MediaControlsSetPlaybackStatus(true, 0);
            }
            catch (Exception ex)
            {
                Logger.Error($"[PlayerService] Failed to update media controls metadata: {ex.Message}");
            }

            _progressTimer.Start();
            SetState(PlaybackState.Playing);

            Logger.Info("[PlayerService] Playback confirmed as started, reporting to server...");
            await _apiService.ReportPlaybackStartAsync(song.id);
            Logger.Info($"[PlayerService] Playback started for: {song.name}");
        }
        catch (Exception ex)
        {
            Logger.Error($"PlayAsync error: {ex}");
            SetState(PlaybackState.Stopped);
        }
    }

    private async Task<bool> TryStartRustPlaybackAsync(string streamUrl, string token, Song song, int requestId, string pathLabel)
    {
        try
        {
            await AureliaCore.AureliaCore.AudioInitPlayer();
            Logger.Info($"[PlayerService] [{pathLabel}] Calling Rust audio play...");
            await AureliaCore.AureliaCore.AudioPlayUrl(streamUrl, token, null);

            for (var i = 0; i < 40; i++)
            {
                if (requestId != _playbackRequestId)
                {
                    return false;
                }

                var isPlaying = await AureliaCore.AureliaCore.AudioIsPlayingPlayer();
                if (isPlaying)
                {
                    Logger.Info($"[PlayerService] [{pathLabel}] Playback engine confirmed Playing");
                    return true;
                }

                await Task.Delay(50);
            }

            Logger.Error($"[PlayerService] [{pathLabel}] Playback start timed out for song id: {song.id}");
            return false;
        }
        catch (Exception ex)
        {
            Logger.Error($"[PlayerService] [{pathLabel}] Playback start error: {ex.Message}");
            return false;
        }
    }

    public void Pause()
    {
        _ = PauseInternalAsync();
    }

    private async Task PauseInternalAsync()
    {
        try
        {
            await AureliaCore.AureliaCore.AudioPausePlayer();
        }
        catch (Exception ex)
        {
            Logger.Error($"[PlayerService] Pause failed: {ex.Message}");
        }

        _progressTimer.Stop();
        try
        {
            AureliaCore.AureliaCore.MediaControlsSetPlaybackStatus(false, Position.TotalSeconds);
        }
        catch (Exception ex)
        {
            Logger.Error($"[PlayerService] Failed to set media controls paused state: {ex.Message}");
        }
        SetState(PlaybackState.Paused);
    }

    public void Resume()
    {
        _ = ResumeInternalAsync();
    }

    private async Task ResumeInternalAsync()
    {
        try
        {
            await AureliaCore.AureliaCore.AudioResumePlayer();
            _progressTimer.Start();
            try
            {
                AureliaCore.AureliaCore.MediaControlsSetPlaybackStatus(true, Position.TotalSeconds);
            }
            catch (Exception ex)
            {
                Logger.Error($"[PlayerService] Failed to set media controls playing state: {ex.Message}");
            }
            SetState(PlaybackState.Playing);
        }
        catch (Exception ex)
        {
            Logger.Error($"[PlayerService] Resume failed: {ex.Message}");
            SetState(PlaybackState.Stopped);
        }
    }

    public void Stop()
    {
        _ = StopInternalAsync();
    }

    private async Task StopInternalAsync()
    {
        try
        {
            await AureliaCore.AureliaCore.AudioStopPlayer();
        }
        catch (Exception ex)
        {
            Logger.Error($"[PlayerService] Stop failed: {ex.Message}");
        }

        _progressTimer.Stop();
        Position = TimeSpan.Zero;
        try
        {
            AureliaCore.AureliaCore.MediaControlsClearNowPlaying();
        }
        catch (Exception ex)
        {
            Logger.Error($"[PlayerService] Failed to clear media controls: {ex.Message}");
        }
        SetState(PlaybackState.Stopped);
    }

    public void Next()
    {
        if (_queue.Count == 0) return;

        if (ShuffleEnabled)
        {
            var random = new Random();
            _currentIndex = random.Next(_queue.Count);
        }
        else if (_currentIndex < _queue.Count - 1)
        {
            _currentIndex++;
        }
        else if (RepeatMode == RepeatMode.All)
        {
            _currentIndex = 0;
        }
        else
        {
            Stop();
            return;
        }

        CurrentSongChanged?.Invoke(this, CurrentSong);
        _ = PlayAsync();
    }

    public void Previous()
    {
        if (_queue.Count == 0) return;

        if (Position.TotalSeconds > 3)
        {
            Seek(TimeSpan.Zero);
            return;
        }

        if (_currentIndex > 0)
        {
            _currentIndex--;
        }
        else if (RepeatMode == RepeatMode.All)
        {
            _currentIndex = _queue.Count - 1;
        }

        CurrentSongChanged?.Invoke(this, CurrentSong);
        _ = PlayAsync();
    }

    public void Seek(TimeSpan position)
    {
        _ = SeekInternalAsync(position);
    }

    private async Task SeekInternalAsync(TimeSpan position)
    {
        try
        {
            await AureliaCore.AureliaCore.AudioSeekPlayer(position.TotalSeconds);
            Position = position;
            RunOnUi(() => PositionChanged?.Invoke(this, position));
        }
        catch (Exception ex)
        {
            Logger.Error($"[PlayerService] Seek failed: {ex.Message}");
        }
    }

    public void PlaySongAt(int index)
    {
        if (index >= 0 && index < _queue.Count)
        {
            _currentIndex = index;
            CurrentSongChanged?.Invoke(this, CurrentSong);
            _ = PlayAsync();
        }
    }

    public void TogglePlayPause()
    {
        if (State == PlaybackState.Playing)
        {
            Pause();
        }
        else if (State == PlaybackState.Paused)
        {
            Resume();
        }
        else
        {
            _ = PlayAsync();
        }
    }

    public void CycleRepeatMode()
    {
        RepeatMode = RepeatMode switch
        {
            RepeatMode.Off => RepeatMode.All,
            RepeatMode.All => RepeatMode.One,
            RepeatMode.One => RepeatMode.Off,
            _ => RepeatMode.Off,
        };
    }

    public void ToggleShuffle()
    {
        ShuffleEnabled = !ShuffleEnabled;
    }

    public void UpdateCurrentSongFavorite(bool isFavorite)
    {
        if (_currentIndex >= 0 && _currentIndex < _queue.Count)
        {
            var currentSong = _queue[_currentIndex];
            var updatedSong = currentSong with { isFavorite = isFavorite };
            _queue[_currentIndex] = updatedSong;
            CurrentSongChanged?.Invoke(this, updatedSong);
        }
    }

    private void SetState(PlaybackState newState)
    {
        State = newState;
        RunOnUi(() => PlaybackStateChanged?.Invoke(this, State));
    }

    private async Task SetRustVolumeAsync(double volume)
    {
        try
        {
            await AureliaCore.AureliaCore.AudioSetVolumePlayer(volume);
        }
        catch (Exception ex)
        {
            Logger.Error($"[PlayerService] Failed to set Rust volume: {ex.Message}");
        }
    }

    private void LoadVolume()
    {
        try
        {
            if (File.Exists(_settingsPath))
            {
                var json = File.ReadAllText(_settingsPath);
                var settings = System.Text.Json.JsonSerializer.Deserialize<Dictionary<string, double>>(json);
                if (settings != null && settings.TryGetValue("volume", out var vol))
                {
                    _volume = Math.Clamp(vol, 0.0, 1.0);
                }
            }
        }
        catch
        {
            _volume = 1.0;
        }
    }

    private void SaveVolume(double volume)
    {
        try
        {
            var json = System.Text.Json.JsonSerializer.Serialize(new Dictionary<string, double> { ["volume"] = volume });
            File.WriteAllText(_settingsPath, json);
        }
        catch
        {
            // ignore volume persistence failures
        }
    }
}

public enum PlaybackState
{
    Stopped,
    Playing,
    Paused
}
