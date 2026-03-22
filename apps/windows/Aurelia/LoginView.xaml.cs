using Aurelia.ViewModels;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Aurelia;

public sealed partial class LoginView : Page {
  private readonly AppViewModel _appViewModel;
  private AureliaCore.BackendProvider? _detectedProvider;

  public LoginView() {
    this.InitializeComponent();

    App app = App.Current;
    _appViewModel = app!.AppViewModel;
  }

  private async void Login_Click(object sender, RoutedEventArgs e) {
    string serverUrl = ServerUrlBox.Text;
    string username = UsernameBox.Text;
    string password = PasswordBox.Password;

    if (string.IsNullOrWhiteSpace(serverUrl) || string.IsNullOrWhiteSpace(username) || string.IsNullOrWhiteSpace(password)) {
      ErrorText.Text = "Please fill in all fields";
      ErrorText.Visibility = Visibility.Visible;
      return;
    }

    ErrorText.Visibility = Visibility.Collapsed;
    DetectedProviderText.Visibility = Visibility.Collapsed;
    LoadingIndicator.Visibility = Visibility.Visible;
    LoginButton.IsEnabled = false;
    DetectProviderButton.IsEnabled = false;

    try {
      AureliaCore.BackendProvider? provider = GetSelectedProvider();
      await _appViewModel.LoginAsync(serverUrl, username, password, provider);

      if (!_appViewModel.IsLoggedIn) {
        ErrorText.Text = "Login failed. Please check your credentials.";
        ErrorText.Visibility = Visibility.Visible;
      }
    } catch (Exception ex) {
      ErrorText.Text = ex.Message;
      ErrorText.Visibility = Visibility.Visible;
    } finally {
      LoadingIndicator.Visibility = Visibility.Collapsed;
      LoginButton.IsEnabled = true;
      DetectProviderButton.IsEnabled = true;
    }
  }

  private async void DetectProvider_Click(object sender, RoutedEventArgs e) {
    string serverUrl = ServerUrlBox.Text;

    if (string.IsNullOrWhiteSpace(serverUrl)) {
      ErrorText.Text = "Enter a server URL first";
      ErrorText.Visibility = Visibility.Visible;
      return;
    }

    ErrorText.Visibility = Visibility.Collapsed;
    DetectProviderButton.IsEnabled = false;
    try {
      _detectedProvider = await _appViewModel.ApiService.DetectProviderAsync(serverUrl);
      if (_detectedProvider == null) {
        DetectedProviderText.Text = "Could not detect provider";
      } else {
        DetectedProviderText.Text = $"Detected provider: {_detectedProvider}";
      }
      DetectedProviderText.Visibility = Visibility.Visible;
    } catch (Exception ex) {
      ErrorText.Text = ex.Message;
      ErrorText.Visibility = Visibility.Visible;
    } finally {
      DetectProviderButton.IsEnabled = true;
    }
  }

  private AureliaCore.BackendProvider? GetSelectedProvider() {
    return ProviderBox.SelectedIndex switch {
      1 => AureliaCore.BackendProvider.Jellyfin,
      2 => AureliaCore.BackendProvider.Navidrome,
      _ => _detectedProvider,
    };
  }

}
