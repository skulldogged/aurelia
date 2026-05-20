using Aurelia.Models;
using System.Text.Json;

namespace Aurelia.Services;

public class SessionService {
  private readonly string _appDataDir;
  private const string ProfilesSettingKey = "saved_profiles_json";
  private const string ActiveProfileSettingKey = "active_profile_id";

  public event EventHandler<SessionInfo?>? SessionChanged;

  private record StoredSessionProfile(
      string Id,
      string Provider,
      string ServerUrl,
      string Username,
      string Token,
      string UserId,
      string? DeviceId,
      DateTimeOffset UpdatedAt
  );

  public SessionService(string appDataDir) {
    _appDataDir = appDataDir;
    LoadCredentials();
  }

  public bool IsLoggedIn => CurrentSession != null && Credentials != null;
  public SessionInfo? CurrentSession { get; private set; }
  public JellyfinCredentials? Credentials { get; private set; }

  public Task SaveCredentialsAsync(JellyfinCredentials credentials) {
    string normalizedUsername = credentials.Username
            ?? (CurrentSession?.User.Name == "User" ? "" : CurrentSession?.User.Name)
            ?? "";
    Credentials = credentials with { Username = normalizedUsername };

    Credentials coreCredentials = new(
            Credentials.Provider,
            Credentials.ServerUrl,
            Credentials.Username ?? "",
            Credentials.AccessToken ?? "",
            Credentials.UserId ?? ""
        );

    AureliaCore.AureliaCore.SaveCredentials(_appDataDir, coreCredentials);

    if (!string.IsNullOrWhiteSpace(Credentials.ServerUrl)
        && !string.IsNullOrWhiteSpace(Credentials.AccessToken)
        && !string.IsNullOrWhiteSpace(Credentials.UserId)) {
      string profileId = UpsertProfile(Credentials);
      SetActiveProfileId(profileId);
    }

    return Task.CompletedTask;
  }

  public Task ClearCredentialsAsync() {
    Credentials = null;
    CurrentSession = null;
    AureliaCore.AureliaCore.ClearCredentials(_appDataDir);
    SetActiveProfileId(null);
    SessionChanged?.Invoke(this, null);
    return Task.CompletedTask;
  }

  public void SetSession(SessionInfo session) {
    CurrentSession = session;
    if (Credentials != null
        && !string.IsNullOrWhiteSpace(Credentials.ServerUrl)
        && !string.IsNullOrWhiteSpace(Credentials.AccessToken)
        && !string.IsNullOrWhiteSpace(Credentials.UserId)) {
      var updated = Credentials with { Username = session.User.Name };
      Credentials = updated;
      string profileId = UpsertProfile(updated);
      SetActiveProfileId(profileId);
    }
    SessionChanged?.Invoke(this, session);
  }

  private void LoadCredentials() {
    try {
      Credentials? creds = AureliaCore.AureliaCore.LoadCredentials(_appDataDir);
      if (creds != null) {
        Credentials = new JellyfinCredentials(
            creds.serverUrl,
            creds.userId,
            creds.token,
            null,
            creds.provider,
            creds.username
        );

        if (GetActiveProfileId() == null
            && !string.IsNullOrWhiteSpace(creds.serverUrl)
            && !string.IsNullOrWhiteSpace(creds.token)
            && !string.IsNullOrWhiteSpace(creds.userId)) {
          string profileId = UpsertProfile(Credentials);
          SetActiveProfileId(profileId);
        }

        if (!string.IsNullOrEmpty(creds.token) && !string.IsNullOrEmpty(creds.userId)) {
          string displayName = string.IsNullOrWhiteSpace(creds.username) ? creds.userId : creds.username;
          CurrentSession = new SessionInfo(
              new User(creds.userId, displayName, false),
              creds.serverUrl,
              creds.token
          );
        }
      }
    } catch {
      Credentials = null;
    }
  }

  public string? GetLyricsServerUrl() {
    try {
      var enabled = AureliaCore.AureliaCore.LoadSetting(_appDataDir, "lyrics_sidecar_enabled");
      if (!string.Equals(enabled, "true", StringComparison.OrdinalIgnoreCase)) {
        return null;
      }

      var v = AureliaCore.AureliaCore.LoadSetting(_appDataDir, "lyrics_server_url");
      return string.IsNullOrWhiteSpace(v) ? null : v;
    } catch { return null; }
  }

  public string? GetSavedLyricsServerUrl() {
    try {
      var v = AureliaCore.AureliaCore.LoadSetting(_appDataDir, "lyrics_server_url");
      return string.IsNullOrWhiteSpace(v) ? null : v;
    } catch { return null; }
  }

  public void SaveLyricsServerUrl(string? url) {
    try { AureliaCore.AureliaCore.SaveSetting(_appDataDir, "lyrics_server_url", url ?? ""); } catch { }
  }

  public bool GetLyricsSidecarEnabled() {
    try {
      var v = AureliaCore.AureliaCore.LoadSetting(_appDataDir, "lyrics_sidecar_enabled");
      return string.Equals(v, "true", StringComparison.OrdinalIgnoreCase);
    } catch { return false; }
  }

  public void SaveLyricsSidecarEnabled(bool enabled) {
    try { AureliaCore.AureliaCore.SaveSetting(_appDataDir, "lyrics_sidecar_enabled", enabled ? "true" : "false"); } catch { }
  }

  public async Task<JellyfinCredentials> GetOrCreateDeviceIdAsync() {
    if (Credentials?.DeviceId != null) {
      return Credentials;
    }

    var deviceId = Guid.NewGuid().ToString();
    JellyfinCredentials updated = new(
            Credentials?.ServerUrl ?? "",
            Credentials?.UserId,
            Credentials?.AccessToken,
            deviceId,
            Credentials?.Provider ?? AureliaCore.BackendProvider.Jellyfin,
            Credentials?.Username
        );
    await SaveCredentialsAsync(updated);
    return updated;
  }

  public IReadOnlyList<SessionProfile> GetProfiles() {
    return LoadStoredProfiles()
        .OrderByDescending(profile => profile.UpdatedAt)
        .Select(profile => {
          AureliaCore.BackendProvider provider = ParseProvider(profile.Provider);
          string username = string.IsNullOrWhiteSpace(profile.Username) ? profile.UserId : profile.Username;
          string label = BuildProfileLabel(profile.ServerUrl, username, provider);
          return new SessionProfile(
            profile.Id,
            label,
            provider,
            profile.ServerUrl,
            profile.UserId,
            username
          );
        })
        .ToList();
  }

  public string? GetActiveProfileId() {
    try {
      string? value = AureliaCore.AureliaCore.LoadSetting(_appDataDir, ActiveProfileSettingKey);
      return string.IsNullOrWhiteSpace(value) ? null : value;
    } catch {
      return null;
    }
  }

  public async Task<bool> SwitchProfileAsync(string profileId) {
    StoredSessionProfile? profile = LoadStoredProfiles().FirstOrDefault(item => item.Id == profileId);
    if (profile == null) {
      return false;
    }

    var credentials = new JellyfinCredentials(
      profile.ServerUrl,
      profile.UserId,
      profile.Token,
      profile.DeviceId,
      ParseProvider(profile.Provider),
      profile.Username
    );
    await SaveCredentialsAsync(credentials);

    string displayName = string.IsNullOrWhiteSpace(profile.Username) ? profile.UserId : profile.Username;
    SetSession(new SessionInfo(
      new User(profile.UserId, displayName, false),
      profile.ServerUrl,
      profile.Token
    ));

    return true;
  }

  public async Task<bool> RemoveProfileAsync(string profileId) {
    List<StoredSessionProfile> profiles = LoadStoredProfiles();
    bool removed = profiles.RemoveAll(profile => profile.Id == profileId) > 0;
    if (!removed) {
      return false;
    }

    SaveStoredProfiles(profiles);

    if (GetActiveProfileId() == profileId) {
      StoredSessionProfile? replacement = profiles
          .OrderByDescending(profile => profile.UpdatedAt)
          .FirstOrDefault();

      if (replacement == null) {
        await ClearCredentialsAsync();
      } else {
        await SwitchProfileAsync(replacement.Id);
      }
    }

    return true;
  }

  private string UpsertProfile(JellyfinCredentials credentials) {
    string id = BuildProfileId(credentials);
    List<StoredSessionProfile> profiles = LoadStoredProfiles();
    profiles.RemoveAll(profile => profile.Id == id);
    profiles.Add(new StoredSessionProfile(
      id,
      credentials.Provider.ToString().ToLowerInvariant(),
      credentials.ServerUrl,
      credentials.Username ?? "",
      credentials.AccessToken ?? "",
      credentials.UserId ?? "",
      credentials.DeviceId,
      DateTimeOffset.UtcNow
    ));
    SaveStoredProfiles(profiles);
    return id;
  }

  private static string BuildProfileId(JellyfinCredentials credentials) {
    string provider = credentials.Provider.ToString().ToLowerInvariant();
    string username = (credentials.Username ?? "").Trim().ToLowerInvariant();
    string serverUrl = credentials.ServerUrl.Trim().ToLowerInvariant();
    return $"{provider}|{username}|{serverUrl}";
  }

  private static string BuildProfileLabel(
      string serverUrl,
      string username,
      AureliaCore.BackendProvider provider
  ) {
    string host = serverUrl;
    if (Uri.TryCreate(serverUrl, UriKind.Absolute, out Uri? uri) && !string.IsNullOrWhiteSpace(uri.Host)) {
      host = uri.Host;
    }
    return $"{username} @ {host}";
  }

  private static AureliaCore.BackendProvider ParseProvider(string value) {
    return AureliaCore.BackendProvider.Jellyfin;
  }

  private List<StoredSessionProfile> LoadStoredProfiles() {
    try {
      string? raw = AureliaCore.AureliaCore.LoadSetting(_appDataDir, ProfilesSettingKey);
      if (string.IsNullOrWhiteSpace(raw)) {
        return [];
      }

      List<StoredSessionProfile>? parsed = JsonSerializer.Deserialize<List<StoredSessionProfile>>(raw);
      return parsed ?? [];
    } catch {
      return [];
    }
  }

  private void SaveStoredProfiles(List<StoredSessionProfile> profiles) {
    try {
      string raw = JsonSerializer.Serialize(profiles);
      AureliaCore.AureliaCore.SaveSetting(_appDataDir, ProfilesSettingKey, raw);
    } catch {
      // best effort only
    }
  }

  private void SetActiveProfileId(string? profileId) {
    try {
      AureliaCore.AureliaCore.SaveSetting(_appDataDir, ActiveProfileSettingKey, profileId ?? "");
    } catch {
      // best effort only
    }
  }
}
