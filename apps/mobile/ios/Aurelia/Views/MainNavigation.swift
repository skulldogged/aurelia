import SwiftUI

enum MainDestination: String, CaseIterable, Identifiable, Hashable {
    case home
    case songs
    case albums
    case artists
    case search
    case settings

    var id: String {
        rawValue
    }

    var title: String {
        switch self {
        case .home: "Home"
        case .songs: "Songs"
        case .albums: "Albums"
        case .artists: "Artists"
        case .search: "Search"
        case .settings: "Settings"
        }
    }

    var systemImage: String {
        switch self {
        case .home: "house.fill"
        case .songs: "music.note"
        case .albums: "square.stack.fill"
        case .artists: "music.mic"
        case .search: "magnifyingglass"
        case .settings: "gearshape.fill"
        }
    }

    @ViewBuilder
    func destinationView() -> some View {
        switch self {
        case .home:
            HomeView()
        case .songs:
            LibraryView()
        case .albums:
            AlbumsView()
        case .artists:
            ArtistsView()
        case .search:
            SearchView()
        case .settings:
            SettingsView()
        }
    }
}
