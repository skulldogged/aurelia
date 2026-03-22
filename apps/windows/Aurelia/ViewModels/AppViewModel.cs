using Aurelia.Models;
using Aurelia.Services;
using System.ComponentModel;
using System.Runtime.CompilerServices;

namespace Aurelia.ViewModels;

public class AppViewModel : INotifyPropertyChanged {
  private bool _isLoading;
  private bool _isLoggedIn;
  private bool _isSyncing;
  private User? _currentUser;
  private string? _serverUrl;

  public event PropertyChangedEventHandler? PropertyChanged;

  public bool IsLoading {
    get => _isLoading;
    set { _isLoading = value; OnPropertyChanged(); }
  }

  public bool IsLoggedIn {
    get => _isLoggedIn;
    set { _isLoggedIn = value; OnPropertyChanged(); }
  }

  public User? CurrentUser {
    get => _currentUser;
    set { _currentUser = value; OnPropertyChanged(); }
  }

  public bool IsSyncing {
    get => _isSyncing;
    set { _isSyncing = value; OnPropertyChanged(); }
  }

  public string? ServerUrl {
    get => _serverUrl;
    set { _serverUrl = value; OnPropertyChanged(); }
  }

  public SessionService SessionService { get; }
  public ApiService ApiService { get; }
  public PlayerService PlayerService { get; }

  public AppViewModel(SessionService sessionService, ApiService apiService, PlayerService playerService) {
    SessionService = sessionService;
    ApiService = apiService;
    PlayerService = playerService;

    SessionService.SessionChanged += OnSessionChanged;
    CheckExistingSession();
  }

  private void CheckExistingSession() {
    SessionInfo? session = SessionService.CurrentSession;
    if (session != null) {
      _isLoggedIn = true;
      _currentUser = session.User;
      _serverUrl = session.ServerUrl;
      OnPropertyChanged(nameof(IsLoggedIn));
      OnPropertyChanged(nameof(CurrentUser));
      OnPropertyChanged(nameof(ServerUrl));
    }
  }

  private void OnSessionChanged(object? sender, SessionInfo? session) {
    if (session != null) {
      IsLoggedIn = true;
      CurrentUser = session.User;
      ServerUrl = session.ServerUrl;
    } else {
      IsLoggedIn = false;
      CurrentUser = null;
      ServerUrl = null;
    }
  }

  public async Task LoginAsync(
      string serverUrl,
      string username,
      string password,
      AureliaCore.BackendProvider? provider = null
  ) {
    IsLoading = true;
    IsSyncing = true;
    try {
      string normalizedUrl = ApiService.NormalizeServerUrl(serverUrl);
      _ = await ApiService.LoginWithPasswordAsync(normalizedUrl, username, password, provider);
      await ApiService.SyncLibraryAsync();
    } finally {
      IsLoading = false;
      IsSyncing = false;
    }
  }

  public async Task LoginWithQuickConnectAsync(string serverUrl, string quickConnectCode) {
    IsLoading = true;
    IsSyncing = true;
    try {
      string normalizedUrl = ApiService.NormalizeServerUrl(serverUrl);
      _ = await ApiService.QuickConnectAsync(normalizedUrl, quickConnectCode);
      await ApiService.SyncLibraryAsync();
    } finally {
      IsLoading = false;
      IsSyncing = false;
    }
  }

  public async Task SyncLibraryAsync() {
    IsSyncing = true;
    try { await ApiService.SyncLibraryAsync(); } finally { IsSyncing = false; }
  }

  public async Task LogoutAsync() {
    await SessionService.ClearCredentialsAsync();
    PlayerService.Stop();
    PlayerService.ClearQueue();
  }

  protected void OnPropertyChanged([CallerMemberName] string? propertyName = null) {
    PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
  }
}
