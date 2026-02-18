using Aurelia.ViewModels;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Aurelia;

public sealed partial class LoginView : Page {
  private readonly AppViewModel _appViewModel;

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
    LoadingIndicator.Visibility = Visibility.Visible;
    LoginButton.IsEnabled = false;

    try {
      await _appViewModel.LoginAsync(serverUrl, username, password);

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
    }
  }

}
