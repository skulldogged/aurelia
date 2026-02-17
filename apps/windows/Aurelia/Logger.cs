namespace Aurelia;

public static class Logger
{
    private static readonly string LogPath;
    private static bool _consoleAttached;
    private static bool _consoleInitialized;
    private const uint AttachParentProcess = 0xFFFFFFFF;

    [System.Runtime.InteropServices.DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool AttachConsole(uint dwProcessId);

    [System.Runtime.InteropServices.DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool AllocConsole();

    static Logger()
    {
        var logDir = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "AureliaWindows"
        );
        Directory.CreateDirectory(logDir);
        LogPath = Path.Combine(logDir, "debug.log");
    }

    public static void InitializeConsoleLogging()
    {
        if (_consoleInitialized)
        {
            return;
        }

        _consoleInitialized = true;
        try
        {
            var noConsole = string.Equals(Environment.GetEnvironmentVariable("AURELIA_NO_CONSOLE_LOG"), "1", StringComparison.Ordinal);
            if (noConsole)
            {
                return;
            }

            _consoleAttached = AttachConsole(AttachParentProcess);

            if (!_consoleAttached)
            {
                _consoleAttached = AllocConsole();
            }

            if (_consoleAttached)
            {
                Console.SetOut(new StreamWriter(Console.OpenStandardOutput()) { AutoFlush = true });
                Console.SetError(new StreamWriter(Console.OpenStandardError()) { AutoFlush = true });
            }
        }
        catch
        {
            _consoleAttached = false;
        }
    }

    public static void Log(string message)
    {
        try
        {
            var line = $"{DateTime.Now:HH:mm:ss.fff} {message}";
            File.AppendAllText(LogPath, $"{line}\n");
            System.Diagnostics.Debug.WriteLine(line);
            if (_consoleAttached)
            {
                Console.WriteLine(line);
            }
        }
        catch { }
    }

    public static void Debug(string message) => Log($"[DEBUG] {message}");
    public static void Info(string message) => Log($"[INFO] {message}");
    public static void Error(string message) => Log($"[ERROR] {message}");
    public static void Error(Exception ex) => Log($"[ERROR] {ex}");
}
