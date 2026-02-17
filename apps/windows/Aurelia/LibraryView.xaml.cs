using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Aurelia.Models;
using Aurelia.ViewModels;

namespace Aurelia;

public sealed partial class LibraryView : Page
{
    private readonly LibraryViewModel _viewModel;

    public LibraryView()
    {
        this.InitializeComponent();

        var app = App.Current as App;
        _viewModel = new LibraryViewModel(app!.ApiService, app.PlayerService);

        this.Loaded += LibraryView_Loaded;
    }

    private async void LibraryView_Loaded(object sender, RoutedEventArgs e)
    {
        LoadingIndicator.Visibility = Visibility.Visible;

        await _viewModel.LoadAsync();

        SongsList.ItemsSource = _viewModel.Songs;
        AlbumsGrid.ItemsSource = _viewModel.Albums;
        ArtistsGrid.ItemsSource = _viewModel.Artists;
        PlaylistsGrid.ItemsSource = _viewModel.Playlists;

        LoadingIndicator.Visibility = Visibility.Collapsed;
    }

    private void Song_Click(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is Song song)
        {
            _viewModel.PlaySong(song);
        }
    }

    private void Album_Click(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is Album album)
        {
            _viewModel.PlayAlbum(album);
        }
    }

    private void Artist_Click(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is Artist artist)
        {
            _viewModel.PlayArtist(artist);
        }
    }

    private void Playlist_Click(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is Playlist playlist)
        {
            _viewModel.PlayPlaylist(playlist);
        }
    }
}
