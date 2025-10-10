# System and Environment Notes

## Operating System
- **Primary OS**: Windows
- **Shell**: PowerShell (pwsh.exe)
- Commands may differ from Unix systems

## Windows-Specific Setup
- Visual Studio C++ Build Tools required for Rust compilation
- WebView2 pre-installed on Windows 10/11 (1809+)
- Environment variables set with PowerShell syntax: `$env:VARIABLE = "value"`

## File Paths
- Use absolute paths with backslashes: `C:\Users\Mars\Projects\...`
- Git bash or WSL may be available but PowerShell is primary

## Development Tools
- All commands run in PowerShell terminal
- Bun works natively on Windows
- Rust installed via rustup works on Windows
- VS Code with Vue/TypeScript extensions recommended

## Common Windows Commands
- `ls` (PowerShell alias for directory listing)
- `cd` for changing directories
- `mkdir` for creating directories
- `git` commands work normally
- Environment variables: `$env:VAR_NAME`