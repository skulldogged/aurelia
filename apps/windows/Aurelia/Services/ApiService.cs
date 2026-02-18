using System.IO;
using System.Text.Json;
using System.Collections.Generic;
using Aurelia.Models;

namespace Aurelia.Services;

public class ApiService
{
    private readonly SessionService _sessionService;
    private readonly string _appDataDir;

    public ApiService(SessionService sessionService, string appDataDir)
    {
        _sessionService = sessionService;
        _appDataDir = appDataDir;
    }

    private string? GetServerUrl() => _sessionService.Credentials?.ServerUrl;
    private string? GetToken() => _sessionService.Credentials?.AccessToken;
    private string? GetUserId() => _sessionService.Credentials?.UserId;

    public string? GetPlaybackToken() => GetToken();

    public async Task<SessionInfo> LoginAsync(string serverUrl, string username, string password)
    {
        var normalizedUrl = NormalizeServerUrl(serverUrl);
        var deviceId = (await _sessionService.GetOrCreateDeviceIdAsync()).DeviceId 
            ?? Guid.NewGuid().ToString();

        try
        {
            var response = await AureliaCore.AureliaCore.Authenticate(
                normalizedUrl, 
                username, 
                password, 
                deviceId
            );

            var credentials = new JellyfinCredentials(
                normalizedUrl,
                response.userId,
                response.token,
                deviceId
            );
            await _sessionService.SaveCredentialsAsync(credentials);

            var session = new SessionInfo(
                new User(response.userId, username, false),
                normalizedUrl,
                response.token
            );
            _sessionService.SetSession(session);

            return session;
        }
        catch (Exception ex)
        {
            throw new Exception($"Login failed: {ex.Message}");
        }
    }

    public async Task<SessionInfo> LoginWithPasswordAsync(string serverUrl, string username, string password)
    {
        return await LoginAsync(serverUrl, username, password);
    }

    public Task<List<User>> GetUsersAsync(string serverUrl)
    {
        return Task.FromResult<List<User>>([]);
    }

    public Task<SessionInfo> QuickConnectAsync(string serverUrl, string quickConnectCode)
    {
        throw new NotImplementedException("QuickConnect not supported on Windows");
    }

    public Task<HomeData> GetHomeDataAsync()
    {
        var serverUrl = GetServerUrl();
        var token = GetToken();
        var userId = GetUserId();

        if (string.IsNullOrEmpty(serverUrl) || string.IsNullOrEmpty(token) || string.IsNullOrEmpty(userId))
        {
            return Task.FromResult(new HomeData());
        }

        try
        {
            var songs = LoadCachedSongsInternal();

            var homeData = AureliaCore.AureliaCore.DeriveMobileHomeData(
                songs,
                12,
                20,
                16,
                12
            );

            return Task.FromResult(new HomeData(
                MostPlayed: homeData.mostPlayed,
                RecentlyPlayed: homeData.recentlyPlayed,
                RecentlyAddedAlbums: homeData.recentlyAdded,
                RandomAlbums: homeData.randomAlbums,
                FeaturedAlbums: homeData.featuredAlbums
            ));
        }
        catch (Exception)
        {
            return Task.FromResult(new HomeData());
        }
    }

    public Task<List<AureliaCore.Song>> GetSongsAsync(string? artistId = null, string? albumId = null, int limit = 100, int startIndex = 0)
    {
        var songs = LoadCachedSongsInternal().ToList();

        if (!string.IsNullOrEmpty(albumId))
        {
            songs = songs.Where(s => s.albumId == albumId).ToList();
            // Sort by track number when viewing a specific album
            songs = songs
                .OrderBy(s => s.trackNumber ?? int.MaxValue)
                .ToList();
        }
        else if (!string.IsNullOrEmpty(artistId))
        {
            songs = songs.Where(s => s.artistIds?.Contains(artistId) == true).ToList();
            // Sort alphabetically by song name when viewing an artist
            songs = songs
                .OrderBy(s => s.name, StringComparer.OrdinalIgnoreCase)
                .ToList();
        }
        else
        {
            // Sort alphabetically by song name in the main library view
            songs = songs
                .OrderBy(s => s.name, StringComparer.OrdinalIgnoreCase)
                .ToList();
        }

        return Task.FromResult(songs
            .Skip(startIndex)
            .Take(limit)
            .ToList());
    }

    public Task<List<AureliaCore.Album>> GetAlbumsAsync(string? artistId = null, int limit = 100, int startIndex = 0)
    {
        var songs = LoadCachedSongsInternal();
        var serverUrl = GetServerUrl();
        var token = GetToken();
        
        var albums = songs
            .Where(s => !string.IsNullOrEmpty(s.albumId) && !string.IsNullOrEmpty(s.album))
            .GroupBy(s => s.albumId)
            .Select(g => g.First())
            .ToList();

        if (!string.IsNullOrEmpty(artistId))
        {
            albums = albums.Where(s => s.artistIds?.Contains(artistId) == true).ToList();
        }

        // Sort alphabetically by album name
        albums = albums.OrderBy(s => s.album, StringComparer.OrdinalIgnoreCase).ToList();

        return Task.FromResult(albums
            .Skip(startIndex)
            .Take(limit)
            .Select(s => new AureliaCore.Album(
                s.albumId,
                s.album ?? "Unknown Album",
                s.artists?.FirstOrDefault() ?? "Unknown Artist",
                s.artistIds?.FirstOrDefault(),
                !string.IsNullOrEmpty(s.albumId) && !string.IsNullOrEmpty(serverUrl) && !string.IsNullOrEmpty(token)
                    ? $"{serverUrl}/Items/{s.albumId}/Images/Primary?MaxWidth=300&Quality=90&api_key={token}"
                    : null,
                0,
                null,
                null,
                null,
                null,
                null
            ))
            .ToList());
    }

    public async Task<AureliaCore.Album?> GetAlbumAsync(string albumId)
    {
        var serverUrl = GetServerUrl();
        var token = GetToken();
        var userId = GetUserId();

        if (string.IsNullOrEmpty(serverUrl) || string.IsNullOrEmpty(token) || string.IsNullOrEmpty(userId))
        {
            return null;
        }

        try
        {
            return await AureliaCore.AureliaCore.FetchAlbum(serverUrl, token, userId, albumId, _appDataDir);
        }
        catch
        {
            return null;
        }
    }

    public Task<List<AureliaCore.Artist>> GetArtistsAsync(int limit = 100, int startIndex = 0)
    {
        var songs = LoadCachedSongsInternal();
        var serverUrl = GetServerUrl();
        var token = GetToken();
        
        var artists = songs
            .Where(s => s.artists != null && s.artists.Length > 0)
            .SelectMany(s => s.artists!.Select((artistName, index) => new { 
                Name = artistName, 
                Id = s.artistIds?.Length > index ? s.artistIds[index] : null 
            }))
            .Where(a => !string.IsNullOrEmpty(a.Name))
            .GroupBy(x => x.Name)
            .Select(g => g.First())
            .OrderBy(a => a.Name, StringComparer.OrdinalIgnoreCase)
            .Skip(startIndex)
            .Take(limit)
            .Select(a => new AureliaCore.Artist(
                a.Name ?? "Unknown Artist",
                a.Id ?? Guid.NewGuid().ToString(),
                null, // imageTags
                !string.IsNullOrEmpty(a.Id) && !string.IsNullOrEmpty(serverUrl) && !string.IsNullOrEmpty(token)
                    ? $"{serverUrl}/Items/{a.Id}/Images/Primary?MaxWidth=300&Quality=90&api_key={token}"
                    : null, // imageUrl
                null, // overview
                null, // providerIds
                null, // communityRating
                null, // songCount
                null, // dateModified
                null  // songs
            ))
            .ToList();

        return Task.FromResult(artists);
    }

    public async Task<AureliaCore.Artist?> GetArtistAsync(string artistId)
    {
        var serverUrl = GetServerUrl();
        var token = GetToken();
        var userId = GetUserId();

        if (string.IsNullOrEmpty(serverUrl) || string.IsNullOrEmpty(token) || string.IsNullOrEmpty(userId))
        {
            return null;
        }

        try
        {
            return await AureliaCore.AureliaCore.FetchArtist(serverUrl, token, userId, artistId, _appDataDir);
        }
        catch
        {
            return null;
        }
    }

    public async Task<List<AureliaCore.Playlist>> GetPlaylistsAsync()
    {
        var serverUrl = GetServerUrl();
        var token = GetToken();
        var userId = GetUserId();

        if (string.IsNullOrEmpty(serverUrl) || string.IsNullOrEmpty(token) || string.IsNullOrEmpty(userId))
        {
            return [];
        }

        try
        {
            return (await AureliaCore.AureliaCore.GetPlaylists(serverUrl, token, userId)).ToList();
        }
        catch
        {
            return [];
        }
    }

    public async Task<AureliaCore.Playlist?> GetPlaylistAsync(string playlistId)
    {
        var serverUrl = GetServerUrl();
        var token = GetToken();

        if (string.IsNullOrEmpty(serverUrl) || string.IsNullOrEmpty(token))
        {
            return null;
        }

        try
        {
            var songs = await AureliaCore.AureliaCore.GetPlaylistItems(serverUrl, token, playlistId);
            return new AureliaCore.Playlist(
                "Playlist",           // name
                "",                    // serverId  
                playlistId,           // id
                null,                 // canDelete
                null,                 // sortName
                false,                // isFolder
                "Playlist",           // itemType
                null,                 // userData
                null,                 // runTimeTicks
                songs?.Length,        // childCount
                null,                 // imageTags
                null,                 // backdropImageTags
                null,                 // imageBlurHashes
                "",                   // locationType
                null,                 // mediaType
                null,                 // dateCreated
                null,                 // dateLastSaved
                null,                 // isFavorite
                null,                 // description
                songs                 // songs
            );
        }
        catch
        {
            return null;
        }
    }

    public Task<List<AureliaCore.Song>> SearchAsync(string query)
    {
        var songs = LoadCachedSongsInternal();

        if (string.IsNullOrWhiteSpace(query))
        {
            return Task.FromResult<List<AureliaCore.Song>>([]);
        }

        var lowerQuery = query.ToLowerInvariant();
        return Task.FromResult(songs
            .Where(s =>
                (s.name?.ToLowerInvariant().Contains(lowerQuery) == true) ||
                (s.album?.ToLowerInvariant().Contains(lowerQuery) == true) ||
                (s.artists?.Any(a => a.ToLowerInvariant().Contains(lowerQuery)) == true))
            .ToList());
    }

    public Task ReportPlaybackStartAsync(string itemId) => Task.CompletedTask;

    public Task ReportPlaybackProgressAsync(string itemId, long positionTicks) => Task.CompletedTask;

    public Task ReportPlaybackStoppedAsync(string itemId, long positionTicks) => Task.CompletedTask;

    public async Task<bool> ToggleFavoriteAsync(string itemId, bool isFavorite)
    {
        var serverUrl = GetServerUrl();
        var token = GetToken();
        var userId = GetUserId();
        
        Logger.Info($"[ToggleFavoriteAsync] serverUrl={serverUrl}, token={token?.Substring(0, Math.Min(10, token?.Length ?? 0))}..., userId={userId}");

        if (string.IsNullOrEmpty(serverUrl) || string.IsNullOrEmpty(token) || string.IsNullOrEmpty(userId))
        {
            Logger.Error("[ToggleFavoriteAsync] Missing credentials");
            return isFavorite;
        }

        try
        {
            var result = await AureliaCore.AureliaCore.ToggleFavorite(serverUrl, token, userId, itemId, isFavorite);
            Logger.Info($"[ToggleFavoriteAsync] Result: {result}");
            return result;
        }
        catch (Exception ex)
        {
            Logger.Error($"[ToggleFavoriteAsync] Error: {ex.Message}");
            return isFavorite;
        }
    }

    public async Task<List<string>> GetFavoriteIdsAsync()
    {
        var serverUrl = GetServerUrl();
        var token = GetToken();
        var userId = GetUserId();

        if (string.IsNullOrEmpty(serverUrl) || string.IsNullOrEmpty(token) || string.IsNullOrEmpty(userId))
        {
            return [];
        }

        try
        {
            var result = await AureliaCore.AureliaCore.GetFavoriteIds(serverUrl, token, userId);
            return result?.ToList() ?? [];
        }
        catch (Exception ex)
        {
            Logger.Error($"[GetFavoriteIdsAsync] Error: {ex.Message}");
            return [];
        }
    }

    public Task<string?> GetImageUrlAsync(string itemId, ImageType type = ImageType.Primary, int maxWidth = 600)
    {
        var serverUrl = GetServerUrl();
        var token = GetToken();
        if (string.IsNullOrEmpty(serverUrl) || string.IsNullOrEmpty(token))
        {
            return Task.FromResult<string?>(null);
        }

        var imageType = type switch
        {
            ImageType.Primary => "Primary",
            ImageType.Backdrop => "Backdrop",
            ImageType.Thumb => "Thumb",
            ImageType.Logo => "Logo",
            _ => "Primary"
        };

        return Task.FromResult<string?>($"{serverUrl}/Items/{itemId}/Images/{imageType}?MaxWidth={maxWidth}&Quality=90&api_key={token}");
    }

    public async Task SyncLibraryAsync()
    {
        var serverUrl = GetServerUrl();
        var token = GetToken();
        var userId = GetUserId();

        // Log sync state before sync
        try {
            var stateBefore = AureliaCore.AureliaCore.GetSyncState(_appDataDir);
            Logger.Info($"[SyncLibraryAsync] State BEFORE: lastSyncTime={stateBefore.lastSyncTime}, fullSyncInProgress={stateBefore.fullSyncInProgress}");
        } catch (Exception ex) {
            Logger.Info($"[SyncLibraryAsync] Error getting state before: {ex.Message}");
        }

        Logger.Info($"[SyncLibraryAsync] serverUrl={serverUrl}, token={token?.Substring(0, Math.Min(10, token?.Length ?? 0))}..., userId={userId}");

        if (string.IsNullOrEmpty(serverUrl) || string.IsNullOrEmpty(token) || string.IsNullOrEmpty(userId))
        {
            Logger.Info("[SyncLibraryAsync] Missing credentials, skipping sync");
            return;
        }

        try
        {
            Logger.Info($"[SyncLibraryAsync] Starting sync to {_appDataDir}");
            var report = await AureliaCore.AureliaCore.SyncLibrarySmart(serverUrl, token, userId, _appDataDir);
            Logger.Info($"[SyncLibraryAsync] Sync complete: {report}");
            
            // Log sync state after sync
            try {
                var stateAfter = AureliaCore.AureliaCore.GetSyncState(_appDataDir);
                Logger.Info($"[SyncLibraryAsync] State AFTER: lastSyncTime={stateAfter.lastSyncTime}, fullSyncInProgress={stateAfter.fullSyncInProgress}");
            } catch (Exception ex) {
                Logger.Info($"[SyncLibraryAsync] Error getting state after: {ex.Message}");
            }
            
            // Reload songs after sync
            var songs = AureliaCore.AureliaCore.LoadCachedSongs(_appDataDir) ?? [];
            Logger.Info($"[SyncLibraryAsync] Reloaded {songs.Length} songs from cache after sync");
        }
        catch (Exception ex)
        {
            Logger.Info($"[SyncLibraryAsync] Sync failed: {ex.Message}");
        }
    }

    public Task<string> GetStreamUrlAsync(string itemId, string? container = null)
    {
        var serverUrl = GetServerUrl();
        var token = GetToken();

        if (string.IsNullOrEmpty(serverUrl) || string.IsNullOrEmpty(token))
        {
            Logger.Info($"[GetStreamUrlAsync] ERROR: serverUrl or token is empty. serverUrl: '{(string.IsNullOrEmpty(serverUrl) ? "(empty)" : serverUrl)}', token: '{(string.IsNullOrEmpty(token) ? "(empty)" : "[REDACTED]")}'");
            return Task.FromResult(string.Empty);
        }

        try
        {
            Logger.Info($"[GetStreamUrlAsync] Building stream URL for itemId: {itemId}, container: {(container ?? "(null)")}");
            var result = AureliaCore.AureliaCore.BuildStreamUrl(serverUrl, token, itemId, container);
            Logger.Info($"[GetStreamUrlAsync] Result: {(string.IsNullOrEmpty(result) ? "(empty)" : result.Substring(0, Math.Min(100, result.Length)) + "...")}");
            return Task.FromResult(result);
        }
        catch (Exception ex)
        {
            Logger.Info($"[GetStreamUrlAsync] EXCEPTION: {ex}");
            return Task.FromResult(string.Empty);
        }
    }

    public Task<string> GetFallbackStreamUrlAsync(string itemId, string? container = null)
    {
        var serverUrl = GetServerUrl();
        var token = GetToken();

        if (string.IsNullOrEmpty(serverUrl) || string.IsNullOrEmpty(token))
        {
            Logger.Info("[GetFallbackStreamUrlAsync] Missing server URL or token");
            return Task.FromResult(string.Empty);
        }

        try
        {
            var normalizedServer = serverUrl.TrimEnd('/');
            var escapedId = Uri.EscapeDataString(itemId);
            var escapedToken = Uri.EscapeDataString(token);
            var result = $"{normalizedServer}/Audio/{escapedId}/stream.aac?api_key={escapedToken}";
            Logger.Info($"[GetFallbackStreamUrlAsync] Building lossy fallback stream URL for itemId: {itemId}, container: {(container ?? "(null)")}");
            Logger.Info($"[GetFallbackStreamUrlAsync] Result: {(string.IsNullOrEmpty(result) ? "(empty)" : result.Substring(0, Math.Min(100, result.Length)) + "...")}");
            return Task.FromResult(result);
        }
        catch (Exception ex)
        {
            Logger.Info($"[GetFallbackStreamUrlAsync] EXCEPTION: {ex}");
            return Task.FromResult(string.Empty);
        }
    }

    public Task<List<string>> GetLosslessFallbackStreamUrlsAsync(string itemId, string? container = null)
    {
        var serverUrl = GetServerUrl();
        var token = GetToken();
        var urls = new List<string>();

        if (string.IsNullOrEmpty(serverUrl) || string.IsNullOrEmpty(token))
        {
            Logger.Info("[GetLosslessFallbackStreamUrlsAsync] Missing server URL or token");
            return Task.FromResult(urls);
        }

        var normalizedServer = serverUrl.TrimEnd('/');
        var escapedId = Uri.EscapeDataString(itemId);
        var escapedToken = Uri.EscapeDataString(token);
        var baseUrl = $"{normalizedServer}/Audio/{escapedId}/universal?api_key={escapedToken}";

        // Lossless FLAC transcode path. This usually strips container quirks/art streams.
        urls.Add(
            $"{baseUrl}" +
            "&container=flac" +
            "&transcodingContainer=flac" +
            "&transcodingProtocol=http" +
            "&audioCodec=flac" +
            "&enableDirectPlay=false" +
            "&enableDirectStream=false" +
            "&maxStreamingBitrate=999999999"
        );

        // Uncompressed PCM/WAV fallback (still lossless, highest compatibility).
        urls.Add(
            $"{baseUrl}" +
            "&container=wav" +
            "&transcodingContainer=wav" +
            "&transcodingProtocol=http" +
            "&audioCodec=pcm_s16le" +
            "&enableDirectPlay=false" +
            "&enableDirectStream=false" +
            "&maxStreamingBitrate=999999999"
        );

        Logger.Info($"[GetLosslessFallbackStreamUrlsAsync] Built {urls.Count} lossless fallback URL(s) for itemId: {itemId}, container: {(container ?? "(null)")}");
        return Task.FromResult(urls);
    }

    public AureliaCore.SyncProgress GetSyncProgress()
    {
        return AureliaCore.AureliaCore.GetSyncProgress();
    }

    public bool HasCachedLibrary()
    {
        var songs = LoadCachedSongsInternal();
        return songs.Length > 0;
    }

    private AureliaCore.Song[] LoadCachedSongsInternal()
    {
        try
        {
            var songs = AureliaCore.AureliaCore.LoadCachedSongs(_appDataDir) ?? [];
            Logger.Info($"[LoadCachedSongsInternal] Loaded {songs.Length} songs from {_appDataDir}");
            return songs;
        }
        catch (Exception ex)
        {
            Logger.Info($"[LoadCachedSongsInternal] Error: {ex.Message}");
            return [];
        }
    }

    public static string NormalizeServerUrl(string url)
    {
        url = url.Trim();
        if (!url.StartsWith("http://", StringComparison.OrdinalIgnoreCase) &&
            !url.StartsWith("https://", StringComparison.OrdinalIgnoreCase))
        {
            url = "https://" + url;
        }
        return url.TrimEnd('/');
    }
}

public enum ImageType
{
    Primary,
    Backdrop,
    Thumb,
    Logo
}
