import SwiftUI

struct MainTabView: View {
    @Binding var selectedTab: MainDestination

    var body: some View {
        TabView(selection: $selectedTab) {
            SwiftUI.Tab("Home", systemImage: MainDestination.home.systemImage, value: MainDestination.home) {
                HomeView()
            }

            SwiftUI.Tab("Songs", systemImage: MainDestination.songs.systemImage, value: MainDestination.songs) {
                LibraryView()
            }

            SwiftUI.Tab("Albums", systemImage: MainDestination.albums.systemImage, value: MainDestination.albums) {
                AlbumsView()
            }

            SwiftUI.Tab("Artists", systemImage: MainDestination.artists.systemImage, value: MainDestination.artists) {
                ArtistsView()
            }

            SwiftUI.Tab("Search", systemImage: MainDestination.search.systemImage, value: MainDestination.search) {
                SearchView()
            }

            SwiftUI.Tab("Settings", systemImage: MainDestination.settings.systemImage, value: MainDestination.settings) {
                SettingsView()
            }
        }
    }
}
