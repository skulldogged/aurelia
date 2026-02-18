using Aurelia.Services;
using Aurelia.ViewModels;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using WinRT.Interop;

namespace Aurelia;

public partial class App : Application {
  private Window? _window;
  private string _appDataDir = null!;

  public static new App Current => (App)Application.Current;

  public SessionService SessionService { get; private set; } = null!;
  public ApiService ApiService { get; private set; } = null!;
  public PlayerService PlayerService { get; private set; } = null!;
  public AppViewModel AppViewModel { get; private set; } = null!;

  public App() {
    this.InitializeComponent();
    UnhandledException += OnUnhandledException;
  }

  protected override void OnLaunched(Microsoft.UI.Xaml.LaunchActivatedEventArgs args) {
    _appDataDir = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "AureliaWindows"
    );
    _ = Directory.CreateDirectory(_appDataDir);
    Logger.InitializeConsoleLogging();
    Environment.SetEnvironmentVariable("AURELIA_RUST_LOG_FILE", Path.Combine(_appDataDir, "rust.log"));
    if (string.IsNullOrWhiteSpace(Environment.GetEnvironmentVariable("RUST_LOG"))) {
      Environment.SetEnvironmentVariable("RUST_LOG", "info");
    }
    Logger.Info("[App] Startup initialized");

    InitializeServices();

    _window = new MainWindow();
    _window.Activate();
    TryInitializeMediaControls();

    Frame frame = new();
    _window.Content = frame;

    if (AppViewModel.IsLoggedIn) {
      if (!ApiService.HasCachedLibrary()) {
        _ = frame.Navigate(typeof(SyncingView));
        _ = AppViewModel.SyncLibraryAsync();
      } else {
        _ = frame.Navigate(typeof(MainView));
        _ = AppViewModel.SyncLibraryAsync();
      }
    } else {
      _ = frame.Navigate(typeof(LoginView));
    }

    AppViewModel.PropertyChanged += (s, e) => {
      if (e.PropertyName == nameof(AppViewModel.IsLoggedIn)) {
        _ = AppViewModel.IsLoggedIn
          ? frame.Navigate(AppViewModel.IsSyncing ? typeof(SyncingView) : typeof(MainView))
          : frame.Navigate(typeof(LoginView));
      }
    };
  }

  private void InitializeServices() {
    SessionService = new SessionService(_appDataDir);
    ApiService = new ApiService(SessionService, _appDataDir);
    PlayerService = new PlayerService(ApiService, _appDataDir);
    AppViewModel = new AppViewModel(SessionService, ApiService, PlayerService);
  }

  private void TryInitializeMediaControls() {
    try {
      if (_window == null) {
        return;
      }

      nint hwnd = WindowNative.GetWindowHandle(_window);
      ulong? hwndValue = hwnd == IntPtr.Zero ? null : (ulong?)hwnd.ToInt64();
      AureliaCore.AureliaCore.MediaControlsInit(hwndValue);
      Logger.Info("[App] Media controls initialized");
    } catch (Exception ex) {
      Logger.Error($"[App] Failed to initialize media controls: {ex.Message}");
    }
  }

  private static void OnUnhandledException(object sender, Microsoft.UI.Xaml.UnhandledExceptionEventArgs e) {
    try {
      string dir = Path.Combine(
          Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
          "AureliaWindows"
      );
      _ = Directory.CreateDirectory(dir);
      string path = Path.Combine(dir, "windows-crash.log");
      File.AppendAllText(path, $"{DateTimeOffset.Now:u} {e.Exception}\n");
    } catch {
      // Avoid throwing during crash handling.
    }
  }
}
