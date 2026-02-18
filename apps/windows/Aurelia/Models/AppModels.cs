namespace Aurelia.Models;

public record JellyfinCredentials(
    string ServerUrl,
    string? UserId = null,
    string? AccessToken = null,
    string? DeviceId = null
);

public record User(
    string Id,
    string Name,
    bool HasPassword = false
);

public record SessionInfo(
    User User,
    string ServerUrl,
    string AccessToken
);

public record HomeData(
    IReadOnlyList<Song>? MostPlayed = null,
    IReadOnlyList<Song>? RecentlyPlayed = null,
    IReadOnlyList<Album>? RecentlyAddedAlbums = null,
    IReadOnlyList<Album>? RandomAlbums = null,
    IReadOnlyList<Album>? FeaturedAlbums = null
);

public enum RepeatMode {
  Off,
  All,
  One
}
