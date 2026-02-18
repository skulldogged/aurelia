using Aurelia.ViewModels;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Aurelia;

public sealed partial class SearchView : Page {
  private readonly SearchViewModel _viewModel;

  public SearchView() {
    this.InitializeComponent();

    App app = App.Current;
    _viewModel = new SearchViewModel(app!.ApiService, app.PlayerService);
  }

  private async void SearchBox_QuerySubmitted(AutoSuggestBox sender, AutoSuggestBoxQuerySubmittedEventArgs args) {
    string query = args.QueryText;
    if (string.IsNullOrWhiteSpace(query)) return;

    LoadingIndicator.Visibility = Visibility.Visible;
    SearchResultsList.Visibility = Visibility.Collapsed;
    EmptyState.Visibility = Visibility.Collapsed;

    await _viewModel.SearchAsync(query);

    SearchResultsList.ItemsSource = _viewModel.SearchResults;

    if (_viewModel.SearchResults.Count > 0) {
      SearchResultsList.Visibility = Visibility.Visible;
    } else {
      EmptyState.Text = "No results found";
      EmptyState.Visibility = Visibility.Visible;
    }

    LoadingIndicator.Visibility = Visibility.Collapsed;
  }

  private void SearchResult_Click(object sender, ItemClickEventArgs e) {
    if (e.ClickedItem is Song song) {
      _viewModel.PlaySong(song);
    }
  }
}
