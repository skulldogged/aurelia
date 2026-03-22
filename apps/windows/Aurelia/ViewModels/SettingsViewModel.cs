using Aurelia.Models;
using Aurelia.Services;
using System.ComponentModel;
using System.Runtime.CompilerServices;

namespace Aurelia.ViewModels;

public class SettingsViewModel : INotifyPropertyChanged {
  private readonly AppViewModel _appViewModel;
  private readonly SessionService _sessionService;

  private string? _serverUrl;
  private string? _username;
  private bool _isLoggingOut;

  public event PropertyChangedEventHandler? PropertyChanged;

  public string? ServerUrl {
    get => _serverUrl;
    set { _serverUrl = value; OnPropertyChanged(); }
  }

  public string? Username {
    get => _username;
    set { _username = value; OnPropertyChanged(); }
  }

  public bool IsLoggingOut {
    get => _isLoggingOut;
    set { _isLoggingOut = value; OnPropertyChanged(); }
  }

  public User? CurrentUser => _appViewModel.CurrentUser;

  public SettingsViewModel(AppViewModel appViewModel, SessionService sessionService) {
    _appViewModel = appViewModel;
    _sessionService = sessionService;

    LoadSettings();
  }

  private void LoadSettings() {
    SessionInfo? session = _sessionService.CurrentSession;
    if (session != null) {
      ServerUrl = session.ServerUrl;
      Username = session.User.Name;
    }
  }

  public async Task LogoutAsync() {
    IsLoggingOut = true;
    try {
      await _appViewModel.LogoutAsync();
    } finally {
      IsLoggingOut = false;
    }
  }

  public async Task SyncLibraryAsync() {
    await _appViewModel.SyncLibraryAsync();
  }

  public async Task<AureliaCore.BackendProvider?> DetectProviderAsync(string serverUrl) {
    return await _appViewModel.ApiService.DetectProviderAsync(serverUrl);
  }

  public async Task SwitchServerAsync(
      string serverUrl,
      string username,
      string password,
      AureliaCore.BackendProvider? provider = null
  ) {
    await _appViewModel.LoginAsync(serverUrl, username, password, provider);
  }

  public async Task AddProfileAsync(
      string serverUrl,
      string username,
      string password,
      AureliaCore.BackendProvider? provider = null
  ) {
    string? previousActiveProfileId = _sessionService.GetActiveProfileId();
    await _appViewModel.ApiService.LoginWithPasswordAsync(serverUrl, username, password, provider);

    if (!string.IsNullOrWhiteSpace(previousActiveProfileId)) {
      await _sessionService.SwitchProfileAsync(previousActiveProfileId);
      SessionInfo? session = _sessionService.CurrentSession;
      if (session != null) {
        ServerUrl = session.ServerUrl;
        Username = session.User.Name;
      }
    }
  }

  public IReadOnlyList<SessionProfile> GetProfiles() {
    return _sessionService.GetProfiles();
  }

  public string? GetActiveProfileId() {
    return _sessionService.GetActiveProfileId();
  }

  public async Task<bool> SwitchProfileAsync(string profileId) {
    bool switched = await _sessionService.SwitchProfileAsync(profileId);
    if (switched) {
      SessionInfo? session = _sessionService.CurrentSession;
      if (session != null) {
        ServerUrl = session.ServerUrl;
        Username = session.User.Name;
      }
      await _appViewModel.SyncLibraryAsync();
    }
    return switched;
  }

  public async Task<bool> RemoveProfileAsync(string profileId) {
    bool removed = await _sessionService.RemoveProfileAsync(profileId);
    if (removed) {
      SessionInfo? session = _sessionService.CurrentSession;
      if (session != null) {
        ServerUrl = session.ServerUrl;
        Username = session.User.Name;
      } else {
        ServerUrl = null;
        Username = null;
      }
    }
    return removed;
  }

  protected void OnPropertyChanged([CallerMemberName] string? propertyName = null) {
    PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
  }
}
