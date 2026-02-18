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

  protected void OnPropertyChanged([CallerMemberName] string? propertyName = null) {
    PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
  }
}
