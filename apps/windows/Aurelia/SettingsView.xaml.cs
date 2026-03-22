using Aurelia.Services;
using Aurelia.ViewModels;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;

namespace Aurelia;

public sealed partial class SettingsView : Page {
  private readonly SettingsViewModel _viewModel;
  private readonly SessionService _sessionService;
  private readonly string _appDataDir;
  private bool _suppressLyricsUrlChanged;
  private AureliaCore.BackendProvider? _detectedSwitchProvider;

  public SettingsView() {
    this.InitializeComponent();

    App app = App.Current;
    _viewModel = new SettingsViewModel(app!.AppViewModel, app.SessionService);
    _sessionService = app.SessionService;
    _appDataDir = System.IO.Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "AureliaWindows"
    );

    LoadSettings();
  }

  private void LoadSettings() {
    if (_viewModel.CurrentUser != null) {
      UsernameText.Text = _viewModel.CurrentUser.Name;
      SwitchUsernameBox.Text = _viewModel.CurrentUser.Name == "User" ? "" : _viewModel.CurrentUser.Name;
    }
    if (_viewModel.ServerUrl != null) {
      ServerUrlText.Text = _viewModel.ServerUrl;
      SwitchServerUrlBox.Text = _viewModel.ServerUrl;
    }
    ProviderText.Text = _sessionService.Credentials?.Provider.ToString() ?? "Unknown provider";

    _suppressLyricsUrlChanged = true;
    LyricsServerUrlBox.Text = _sessionService.GetLyricsServerUrl() ?? "";
    _suppressLyricsUrlChanged = false;

    RenderProfiles();
  }

  private async void Logout_Click(object sender, RoutedEventArgs e) {
    await _viewModel.LogoutAsync();
  }

  private async void Sync_Click(object sender, RoutedEventArgs e) {
    SyncProgress.Visibility = Visibility.Visible;
    SyncButton.IsEnabled = false;
    ClearCacheButton.IsEnabled = false;
    SyncStatus.Text = "Syncing library...";

    try {
      await _viewModel.SyncLibraryAsync();
      SyncStatus.Text = "Library synced successfully!";
    } catch (System.Exception ex) {
      SyncStatus.Text = $"Sync failed: {ex.Message}";
    } finally {
      SyncProgress.Visibility = Visibility.Collapsed;
      SyncButton.IsEnabled = true;
      ClearCacheButton.IsEnabled = true;
    }
  }

  private async void DetectSwitchProvider_Click(object sender, RoutedEventArgs e) {
    string serverUrl = SwitchServerUrlBox.Text.Trim();
    if (string.IsNullOrWhiteSpace(serverUrl)) {
      SyncStatus.Text = "Enter a server URL to detect provider";
      return;
    }

    DetectSwitchProviderButton.IsEnabled = false;
    try {
      _detectedSwitchProvider = await _viewModel.DetectProviderAsync(serverUrl);
      if (_detectedSwitchProvider == null) {
        DetectedSwitchProviderText.Text = "Could not detect provider";
      } else {
        DetectedSwitchProviderText.Text = $"Detected provider: {_detectedSwitchProvider}";
      }
      DetectedSwitchProviderText.Visibility = Visibility.Visible;
    } finally {
      DetectSwitchProviderButton.IsEnabled = true;
    }
  }

  private async void SwitchServer_Click(object sender, RoutedEventArgs e) {
    string serverUrl = SwitchServerUrlBox.Text.Trim();
    string username = SwitchUsernameBox.Text.Trim();
    string password = SwitchPasswordBox.Password;

    if (string.IsNullOrWhiteSpace(serverUrl) || string.IsNullOrWhiteSpace(username) || string.IsNullOrWhiteSpace(password)) {
      SyncStatus.Text = "Enter server URL, username, and password to switch";
      return;
    }

    SyncProgress.Visibility = Visibility.Visible;
    SwitchServerButton.IsEnabled = false;
    DetectSwitchProviderButton.IsEnabled = false;
    SyncStatus.Text = "Adding profile...";

    try {
      AureliaCore.BackendProvider? provider = GetSwitchProviderSelection();
      await _viewModel.AddProfileAsync(serverUrl, username, password, provider);
      SwitchPasswordBox.Password = "";
      LoadSettings();
      SyncStatus.Text = "Profile added";
    } catch (Exception ex) {
      SyncStatus.Text = $"Add profile failed: {ex.Message}";
    } finally {
      SyncProgress.Visibility = Visibility.Collapsed;
      SwitchServerButton.IsEnabled = true;
      DetectSwitchProviderButton.IsEnabled = true;
    }
  }

  private AureliaCore.BackendProvider? GetSwitchProviderSelection() {
    return SwitchProviderBox.SelectedIndex switch {
      1 => AureliaCore.BackendProvider.Jellyfin,
      2 => AureliaCore.BackendProvider.Navidrome,
      _ => _detectedSwitchProvider,
    };
  }

  private void RenderProfiles() {
    ProfilesListPanel.Children.Clear();

    IReadOnlyList<Aurelia.Models.SessionProfile> profiles = _viewModel.GetProfiles();
    string? activeProfileId = _viewModel.GetActiveProfileId();

    if (profiles.Count == 0) {
      ProfilesListPanel.Children.Add(new TextBlock {
        Text = "No saved profiles",
        Opacity = 0.7,
      });
      return;
    }

    foreach (Aurelia.Models.SessionProfile profile in profiles) {
      bool isActive = profile.Id == activeProfileId;

      var row = new Grid {
        Margin = new Thickness(0, 0, 0, 10),
      };
      row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
      row.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });

      var infoPanel = new StackPanel();
      infoPanel.Children.Add(new TextBlock {
        Text = isActive ? $"{profile.Label} (active)" : profile.Label,
        FontWeight = isActive ? Microsoft.UI.Text.FontWeights.SemiBold : Microsoft.UI.Text.FontWeights.Normal,
      });
      infoPanel.Children.Add(new TextBlock {
        Text = profile.ServerUrl,
        Opacity = 0.7,
        FontSize = 12,
      });
      row.Children.Add(infoPanel);

      var actions = new StackPanel {
        Orientation = Orientation.Horizontal,
        Spacing = 8,
      };
      Grid.SetColumn(actions, 1);

      var switchButton = new Button {
        Content = "Switch",
        IsEnabled = !isActive,
      };
      switchButton.Click += async (_, _) => {
        SyncProgress.Visibility = Visibility.Visible;
        SyncStatus.Text = "Switching profile...";
        try {
          bool switched = await _viewModel.SwitchProfileAsync(profile.Id);
          SyncStatus.Text = switched ? "Profile switched and synced" : "Failed to switch profile";
          LoadSettings();
        } catch (Exception ex) {
          SyncStatus.Text = $"Failed to switch profile: {ex.Message}";
        } finally {
          SyncProgress.Visibility = Visibility.Collapsed;
        }
      };

      var removeButton = new Button {
        Content = "Remove",
      };
      removeButton.Click += async (_, _) => {
        SyncProgress.Visibility = Visibility.Visible;
        SyncStatus.Text = "Removing profile...";
        try {
          bool removed = await _viewModel.RemoveProfileAsync(profile.Id);
          if (!removed) {
            SyncStatus.Text = "Failed to remove profile";
          } else if (_viewModel.GetProfiles().Count == 0) {
            SyncStatus.Text = "Profile removed. Logged out.";
          } else {
            SyncStatus.Text = "Profile removed";
          }
          LoadSettings();
        } catch (Exception ex) {
          SyncStatus.Text = $"Failed to remove profile: {ex.Message}";
        } finally {
          SyncProgress.Visibility = Visibility.Collapsed;
        }
      };

      actions.Children.Add(switchButton);
      actions.Children.Add(removeButton);
      row.Children.Add(actions);
      ProfilesListPanel.Children.Add(row);
    }
  }

  private void ClearCache_Click(object sender, RoutedEventArgs e) {
    try {
      // First clear the cache (songs/artists/albums)
      AureliaCore.AureliaCore.ClearCache(_appDataDir);

      // Then reset sync state separately to force a full sync (use snake_case for Rust)
      AureliaCore.AureliaCore.SetLibrarySyncState(_appDataDir, "{\"last_sync_time\":\"1970-01-01T00:00:00Z\",\"last_full_sync_time\":null,\"last_sync_version\":null,\"song_count\":0,\"artist_count\":0,\"album_count\":0,\"full_sync_in_progress\":false,\"full_sync_last_page_index\":0,\"full_sync_entity_type\":null}");

      System.IO.File.AppendAllText(_appDataDir + "\\app.log", $"{DateTime.Now:HH:mm:ss} [ClearCache] Cache cleared, sync state reset to epoch\n");
      SyncStatus.Text = "Cache cleared!";
    } catch (System.Exception ex) {
      SyncStatus.Text = $"Failed to clear cache: {ex.Message}";
    }
  }

  private void LyricsServerUrl_TextChanged(object sender, TextChangedEventArgs e) {
    if (_suppressLyricsUrlChanged) return;
    _sessionService.SaveLyricsServerUrl(LyricsServerUrlBox.Text.Trim());
  }
}
