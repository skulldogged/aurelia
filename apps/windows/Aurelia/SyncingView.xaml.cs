using Aurelia.ViewModels;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;
using System.ComponentModel;

namespace Aurelia;

public sealed partial class SyncingView : Page {
  private DispatcherTimer? _progressTimer;
  private double _displayedProgress;
  private double _targetProgress;
  private double _displayedCount;
  private double _targetCount;

  public SyncingView() {
    this.InitializeComponent();
  }

  private void SyncingView_Loaded(object sender, RoutedEventArgs e) {
    AppViewModel vm = App.Current.AppViewModel;
    if (!vm.IsSyncing) {
      _ = Frame.Navigate(typeof(MainView));
      return;
    }

    _progressTimer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(250) };
    _progressTimer.Tick += OnProgressTick;
    _progressTimer.Start();

    vm.PropertyChanged += OnViewModelPropertyChanged;
  }

  protected override void OnNavigatedFrom(NavigationEventArgs e) {
    base.OnNavigatedFrom(e);
    _progressTimer?.Stop();
    _progressTimer = null;
    App.Current.AppViewModel.PropertyChanged -= OnViewModelPropertyChanged;
  }

  private void OnProgressTick(object? sender, object e) {
    SyncProgress progress = App.Current.ApiService.GetSyncProgress();

    StatusText.Text = FormatStage(progress.stage);

    if (progress.total > 0) {
      _targetProgress = (double)progress.current / progress.total * 100;
      _displayedProgress += (_targetProgress - _displayedProgress) * 0.12;

      _targetCount = progress.current;
      _displayedCount += (_targetCount - _displayedCount) * 0.12;

      SyncProgressBar.IsIndeterminate = false;
      SyncProgressBar.Value = _displayedProgress;
      CountText.Text = $"{(int)_displayedCount:N0} / {progress.total:N0}";
    } else {
      SyncProgressBar.IsIndeterminate = true;
      CountText.Text = string.Empty;
    }
  }

  private static string FormatStage(string stage) {
    return stage switch {
      "songs" => "Syncing songs…",
      "albums" => "Syncing albums…",
      "artists" => "Syncing artists…",
      _ => "Syncing your library…",
    };
  }

  private void OnViewModelPropertyChanged(object? sender, PropertyChangedEventArgs e) {
    if (e.PropertyName == nameof(AppViewModel.IsSyncing) && !App.Current.AppViewModel.IsSyncing) {
      App.Current.AppViewModel.PropertyChanged -= OnViewModelPropertyChanged;
      _progressTimer?.Stop();
      _progressTimer = null;
      _ = DispatcherQueue.TryEnqueue(() => Frame.Navigate(typeof(MainView)));
    }
  }
}
