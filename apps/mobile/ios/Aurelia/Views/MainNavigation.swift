import SwiftUI

enum MainDestination: String, CaseIterable, Identifiable {
    case home
    case songs
    case albums
    case artists
    case search
    case settings

    var id: String { rawValue }

    var title: String {
        switch self {
        case .home: return "Home"
        case .songs: return "Songs"
        case .albums: return "Albums"
        case .artists: return "Artists"
        case .search: return "Search"
        case .settings: return "Settings"
        }
    }

    var systemImage: String {
        switch self {
        case .home: return "house.fill"
        case .songs: return "music.note"
        case .albums: return "square.stack.fill"
        case .artists: return "music.mic"
        case .search: return "magnifyingglass"
        case .settings: return "gearshape.fill"
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
