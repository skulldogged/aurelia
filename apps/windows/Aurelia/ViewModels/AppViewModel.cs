using System.ComponentModel;
using System.Runtime.CompilerServices;
using Aurelia.Models;
using Aurelia.Services;

namespace Aurelia.ViewModels;

public class AppViewModel : INotifyPropertyChanged
{
    private readonly SessionService _sessionService;
    private readonly ApiService _apiService;
    private readonly PlayerService _playerService;
    
    private bool _isLoading;
    private bool _isLoggedIn;
    private bool _isSyncing;
    private User? _currentUser;
    private string? _serverUrl;

    public event PropertyChangedEventHandler? PropertyChanged;

    public bool IsLoading
    {
        get => _isLoading;
        set { _isLoading = value; OnPropertyChanged(); }
    }

    public bool IsLoggedIn
    {
        get => _isLoggedIn;
        set { _isLoggedIn = value; OnPropertyChanged(); }
    }

    public User? CurrentUser
    {
        get => _currentUser;
        set { _currentUser = value; OnPropertyChanged(); }
    }

    public bool IsSyncing
    {
        get => _isSyncing;
        set { _isSyncing = value; OnPropertyChanged(); }
    }

    public string? ServerUrl
    {
        get => _serverUrl;
        set { _serverUrl = value; OnPropertyChanged(); }
    }

    public SessionService SessionService => _sessionService;
    public ApiService ApiService => _apiService;
    public PlayerService PlayerService => _playerService;

    public AppViewModel(SessionService sessionService, ApiService apiService, PlayerService playerService)
    {
        _sessionService = sessionService;
        _apiService = apiService;
        _playerService = playerService;

        _sessionService.SessionChanged += OnSessionChanged;
        CheckExistingSession();
    }

    private void CheckExistingSession()
    {
        var session = _sessionService.CurrentSession;
        if (session != null)
        {
            _isLoggedIn = true;
            _currentUser = session.User;
            _serverUrl = session.ServerUrl;
            OnPropertyChanged(nameof(IsLoggedIn));
            OnPropertyChanged(nameof(CurrentUser));
            OnPropertyChanged(nameof(ServerUrl));
        }
    }

    private void OnSessionChanged(object? sender, SessionInfo? session)
    {
        if (session != null)
        {
            IsLoggedIn = true;
            CurrentUser = session.User;
            ServerUrl = session.ServerUrl;
        }
        else
        {
            IsLoggedIn = false;
            CurrentUser = null;
            ServerUrl = null;
        }
    }

    public async Task LoginAsync(string serverUrl, string username, string password)
    {
        IsLoading = true;
        IsSyncing = true;
        try
        {
            var normalizedUrl = ApiService.NormalizeServerUrl(serverUrl);
            await _apiService.LoginWithPasswordAsync(normalizedUrl, username, password);
            await _apiService.SyncLibraryAsync();
        }
        finally
        {
            IsLoading = false;
            IsSyncing = false;
        }
    }

    public async Task LoginWithQuickConnectAsync(string serverUrl, string quickConnectCode)
    {
        IsLoading = true;
        IsSyncing = true;
        try
        {
            var normalizedUrl = ApiService.NormalizeServerUrl(serverUrl);
            await _apiService.QuickConnectAsync(normalizedUrl, quickConnectCode);
            await _apiService.SyncLibraryAsync();
        }
        finally
        {
            IsLoading = false;
            IsSyncing = false;
        }
    }

    public async Task SyncLibraryAsync()
    {
        IsSyncing = true;
        try { await _apiService.SyncLibraryAsync(); }
        finally { IsSyncing = false; }
    }

    public async Task LogoutAsync()
    {
        await _sessionService.ClearCredentialsAsync();
        _playerService.Stop();
        _playerService.ClearQueue();
    }

    protected void OnPropertyChanged([CallerMemberName] string? propertyName = null)
    {
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
    }
}
