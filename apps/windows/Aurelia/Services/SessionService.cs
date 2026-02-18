using Aurelia.Models;

namespace Aurelia.Services;

public class SessionService {
  private readonly string _appDataDir;

  public event EventHandler<SessionInfo?>? SessionChanged;

  public SessionService(string appDataDir) {
    _appDataDir = appDataDir;
    LoadCredentials();
  }

  public bool IsLoggedIn => CurrentSession != null && Credentials != null;
  public SessionInfo? CurrentSession { get; private set; }
  public JellyfinCredentials? Credentials { get; private set; }

  public Task SaveCredentialsAsync(JellyfinCredentials credentials) {
    Credentials = credentials;

    Credentials coreCredentials = new(
            credentials.ServerUrl,
            "",
            credentials.AccessToken ?? "",
            credentials.UserId ?? ""
        );

    AureliaCore.AureliaCore.SaveCredentials(_appDataDir, coreCredentials);
    return Task.CompletedTask;
  }

  public Task ClearCredentialsAsync() {
    Credentials = null;
    CurrentSession = null;
    AureliaCore.AureliaCore.ClearCredentials(_appDataDir);
    SessionChanged?.Invoke(this, null);
    return Task.CompletedTask;
  }

  public void SetSession(SessionInfo session) {
    CurrentSession = session;
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
            null
        );

        if (!string.IsNullOrEmpty(creds.token) && !string.IsNullOrEmpty(creds.userId)) {
          CurrentSession = new SessionInfo(
              new User(creds.userId, "User", false),
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
      var v = AureliaCore.AureliaCore.LoadSetting(_appDataDir, "lyrics_server_url");
      return string.IsNullOrWhiteSpace(v) ? null : v;
    } catch { return null; }
  }

  public void SaveLyricsServerUrl(string? url) {
    try { AureliaCore.AureliaCore.SaveSetting(_appDataDir, "lyrics_server_url", url ?? ""); } catch { }
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
            deviceId
        );
    await SaveCredentialsAsync(updated);
    return updated;
  }
}
