using Microsoft.UI.Xaml;

namespace Aurelia;

public sealed partial class MainWindow : Window {
  private readonly string _appDataDir;
  private bool _backdropInitialized;

  public MainWindow() {
    this.InitializeComponent();
    _appDataDir = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "Aurelia"
    );

    _ = Directory.CreateDirectory(_appDataDir);
    TryUseIntegratedTitleBar();
    Activated += MainWindow_Activated;
  }

  private void MainWindow_Activated(object sender, WindowActivatedEventArgs args) {
    if (_backdropInitialized) {
      return;
    }

    _backdropInitialized = true;
    TryEnableMicaBackdrop();
  }

  private void TryEnableMicaBackdrop() {
    try {
      SystemBackdrop = new Microsoft.UI.Xaml.Media.MicaBackdrop();
    } catch {
      // Keep default background if Mica fails on this machine.
    }
  }

  private void TryUseIntegratedTitleBar() {
    try {
      ExtendsContentIntoTitleBar = true;
      SetTitleBar(TitleBarHost);
    } catch {
      // Keep default system title bar on machines where custom title bars fail.
    }
  }
}
