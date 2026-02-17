using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Aurelia.ViewModels;
using AureliaCore;

namespace Aurelia;

public sealed partial class SettingsView : Page
{
    private readonly SettingsViewModel _viewModel;
    private readonly string _appDataDir;

    public SettingsView()
    {
        this.InitializeComponent();

        var app = App.Current as App;
        _viewModel = new SettingsViewModel(app!.AppViewModel, app.SessionService);
        _appDataDir = System.IO.Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "AureliaWindows"
        );

        LoadSettings();
    }

    private void LoadSettings()
    {
        if (_viewModel.CurrentUser != null)
        {
            UsernameText.Text = _viewModel.CurrentUser.Name;
        }
        if (_viewModel.ServerUrl != null)
        {
            ServerUrlText.Text = _viewModel.ServerUrl;
        }
    }

    private async void Logout_Click(object sender, RoutedEventArgs e)
    {
        await _viewModel.LogoutAsync();
    }

    private async void Sync_Click(object sender, RoutedEventArgs e)
    {
        SyncProgress.Visibility = Visibility.Visible;
        SyncButton.IsEnabled = false;
        ClearCacheButton.IsEnabled = false;
        SyncStatus.Text = "Syncing library...";

        try
        {
            await _viewModel.SyncLibraryAsync();
            SyncStatus.Text = "Library synced successfully!";
        }
        catch (System.Exception ex)
        {
            SyncStatus.Text = $"Sync failed: {ex.Message}";
        }
        finally
        {
            SyncProgress.Visibility = Visibility.Collapsed;
            SyncButton.IsEnabled = true;
            ClearCacheButton.IsEnabled = true;
        }
    }

    private async void ClearCache_Click(object sender, RoutedEventArgs e)
    {
        try
        {
            // First clear the cache (songs/artists/albums)
            AureliaCore.AureliaCore.ClearCache(_appDataDir);
            
            // Then reset sync state separately to force a full sync (use snake_case for Rust)
            AureliaCore.AureliaCore.SetLibrarySyncState(_appDataDir, "{\"last_sync_time\":\"1970-01-01T00:00:00Z\",\"last_full_sync_time\":null,\"last_sync_version\":null,\"song_count\":0,\"artist_count\":0,\"album_count\":0,\"full_sync_in_progress\":false,\"full_sync_last_page_index\":0,\"full_sync_entity_type\":null}");
            
            System.IO.File.AppendAllText(_appDataDir + "\\app.log", $"{DateTime.Now:HH:mm:ss} [ClearCache] Cache cleared, sync state reset to epoch\n");
            SyncStatus.Text = "Cache cleared!";
        }
        catch (System.Exception ex)
        {
            SyncStatus.Text = $"Failed to clear cache: {ex.Message}";
        }
    }
}
