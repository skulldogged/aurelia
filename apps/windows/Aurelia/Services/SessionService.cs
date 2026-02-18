using System.IO;
using System.Text.Json;
using AureliaCore;
using Aurelia.Models;

namespace Aurelia.Services;

public class SessionService
{
    private readonly string _appDataDir;
    private JellyfinCredentials? _credentials;
    private SessionInfo? _session;

    public event EventHandler<SessionInfo?>? SessionChanged;

    public SessionService(string appDataDir)
    {
        _appDataDir = appDataDir;
        LoadCredentials();
    }

    public bool IsLoggedIn => _session != null && _credentials != null;
    public SessionInfo? CurrentSession => _session;
    public JellyfinCredentials? Credentials => _credentials;

    public Task SaveCredentialsAsync(JellyfinCredentials credentials)
    {
        _credentials = credentials;

        var coreCredentials = new AureliaCore.Credentials(
            credentials.ServerUrl,
            "",
            credentials.AccessToken ?? "",
            credentials.UserId ?? ""
        );

        AureliaCore.AureliaCore.SaveCredentials(_appDataDir, coreCredentials);
        return Task.CompletedTask;
    }

    public Task ClearCredentialsAsync()
    {
        _credentials = null;
        _session = null;
        AureliaCore.AureliaCore.ClearCredentials(_appDataDir);
        SessionChanged?.Invoke(this, null);
        return Task.CompletedTask;
    }

    public void SetSession(SessionInfo session)
    {
        _session = session;
        SessionChanged?.Invoke(this, session);
    }

    private void LoadCredentials()
    {
        try
        {
            var creds = AureliaCore.AureliaCore.LoadCredentials(_appDataDir);
            if (creds != null)
            {
                _credentials = new JellyfinCredentials(
                    creds.serverUrl,
                    creds.userId,
                    creds.token,
                    null
                );
                
                if (!string.IsNullOrEmpty(creds.token) && !string.IsNullOrEmpty(creds.userId))
                {
                    _session = new SessionInfo(
                        new User(creds.userId, "User", false),
                        creds.serverUrl,
                        creds.token
                    );
                }
            }
        }
        catch
        {
            _credentials = null;
        }
    }

    public async Task<JellyfinCredentials> GetOrCreateDeviceIdAsync()
    {
        if (_credentials?.DeviceId != null)
        {
            return _credentials;
        }

        var deviceId = Guid.NewGuid().ToString();
        var updated = new JellyfinCredentials(
            _credentials?.ServerUrl ?? "",
            _credentials?.UserId,
            _credentials?.AccessToken,
            deviceId
        );
        await SaveCredentialsAsync(updated);
        return updated;
    }
}
