import SwiftUI

struct MainTabView: View {
    @Environment(AudioPlayerController.self) private var playerController
    @State private var selectedTab: Tab = .home
    @State private var showPlayer = false

    enum Tab: Hashable {
        case home, songs, albums, artists, search, settings
    }

    var body: some View {
        ZStack(alignment: .bottom) {
            TabView(selection: $selectedTab) {
                SwiftUI.Tab("Home", systemImage: "house.fill", value: Tab.home) {
                    HomeView()
                }

                SwiftUI.Tab("Songs", systemImage: "music.note", value: Tab.songs) {
                    LibraryView()
                }

                SwiftUI.Tab("Albums", systemImage: "square.stack.fill", value: Tab.albums) {
                    AlbumsView()
                }

                SwiftUI.Tab("Artists", systemImage: "music.mic", value: Tab.artists) {
                    ArtistsView()
                }

                SwiftUI.Tab("Search", systemImage: "magnifyingglass", value: Tab.search) {
                    SearchView()
                }

                SwiftUI.Tab("Settings", systemImage: "gearshape.fill", value: Tab.settings) {
                    SettingsView()
                }
            }

            // Mini Player overlay
            if playerController.snapshot.currentSongId != nil {
                MiniPlayerView(onTap: { showPlayer = true })
                    .padding(.bottom, 49) // Tab bar height
            }
        }
        .sheet(isPresented: $showPlayer) {
            PlayerView()
        }
    }
}
