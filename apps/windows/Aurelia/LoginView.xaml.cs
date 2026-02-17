using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Aurelia.ViewModels;

namespace Aurelia;

public sealed partial class LoginView : Page
{
    private readonly AppViewModel _appViewModel;

    public LoginView()
    {
        this.InitializeComponent();

        var app = App.Current as App;
        _appViewModel = app!.AppViewModel;

        _appViewModel.PropertyChanged += AppViewModel_PropertyChanged;
    }

    private void AppViewModel_PropertyChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(AppViewModel.IsLoggedIn) && _appViewModel.IsLoggedIn)
        {
            NavigateToMain();
        }
    }

    private async void Login_Click(object sender, RoutedEventArgs e)
    {
        var serverUrl = ServerUrlBox.Text;
        var username = UsernameBox.Text;
        var password = PasswordBox.Password;

        if (string.IsNullOrWhiteSpace(serverUrl) || string.IsNullOrWhiteSpace(username) || string.IsNullOrWhiteSpace(password))
        {
            ErrorText.Text = "Please fill in all fields";
            ErrorText.Visibility = Visibility.Visible;
            return;
        }

        ErrorText.Visibility = Visibility.Collapsed;
        LoadingIndicator.Visibility = Visibility.Visible;
        LoginButton.IsEnabled = false;

        try
        {
            await _appViewModel.LoginAsync(serverUrl, username, password);
            
            if (!_appViewModel.IsLoggedIn)
            {
                ErrorText.Text = "Login failed. Please check your credentials.";
                ErrorText.Visibility = Visibility.Visible;
            }
        }
        catch (Exception ex)
        {
            ErrorText.Text = ex.Message;
            ErrorText.Visibility = Visibility.Visible;
        }
        finally
        {
            LoadingIndicator.Visibility = Visibility.Collapsed;
            LoginButton.IsEnabled = true;
        }
    }

    private void NavigateToMain()
    {
        if (Frame != null)
        {
            Frame.Navigate(typeof(MainView));
        }
    }
}
