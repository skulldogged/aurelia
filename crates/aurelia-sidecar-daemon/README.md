# Aurelia Sidecar Lyrics Daemon

A lightweight HTTP server that serves word-synced lyrics from sidecar files for Jellyfin music libraries.

## Why?

Instead of running the full Aurelia web backend (which proxies all Jellyfin requests), users can run this minimal daemon alongside their Jellyfin server. Aurelia clients connect directly to:
- Jellyfin for music library, streaming, and authentication
- This daemon ONLY for lyrics

This makes deployment much simpler and lighter.

## Features

- **Multiple lyric formats**: TTML (Apple Music), LRC, Enhanced LRC, plain text
- **Word-level synchronization**: Full support for word-synced TTML files
- **Jellyfin integration**: Automatically resolves item IDs to file paths
- **In-memory caching**: Fast responses for frequently played songs
- **CORS enabled**: Works with web clients
- **Tiny footprint**: ~5MB binary vs ~50MB+ for full web backend

## Quick Start

### Using Environment Variables

```bash
export JELLYFIN_URL="http://localhost:8096"
export JELLYFIN_API_KEY="your-api-key"
export MUSIC_PATHS="/media/music,/mnt/music"
export PORT=8080

./aurelia-sidecar-daemon
```

### Using Config File

```bash
./aurelia-sidecar-daemon --config /etc/aurelia/daemon.toml
```

See `example-config.toml` for format.

### Using CLI Args

```bash
./aurelia-sidecar-daemon \
  --jellyfin-url http://localhost:8096 \
  --jellyfin-api-key YOUR_KEY \
  --music-paths /media/music \
  --port 8080
```

## API Endpoints

### GET /health
Health check and status.

```json
{
  "status": "ok",
  "version": "0.1.0",
  "jellyfin_connected": true,
  "cache_size": 42
}
```

### GET /lyrics/{item_id}
Get parsed lyrics for a Jellyfin item.

```bash
curl http://localhost:8080/lyrics/123e4567-e89b-12d3-a456-426614174000
```

Response:
```json
{
  "item_id": "123e4567-e89b-12d3-a456-426614174000",
  "found": true,
  "source": "Sidecar(\"/media/music/Artist/Album/song.ttml\")",
  "lyrics": {
    "plain": ["Line 1", "Line 2"],
    "synced": [
      {
        "time_ms": 10000,
        "end_time_ms": 15000,
        "line": "Line 1",
        "words": [
          {"time_ms": 10000, "end_time_ms": 12000, "word": "Line"},
          {"time_ms": 12000, "end_time_ms": 15000, "word": "1"}
        ]
      }
    ],
    "sections": null,
    "agents": null,
    "songwriters": null,
    "language": "en"
  }
}
```

### GET /lyrics/{item_id}/raw
Get raw lyrics file content (for debugging).

### GET /cache/clear
Clear the lyrics cache.

## Sidecar File Format

The daemon looks for lyrics files in the same directory as the audio file, with the same base name.

**Priority order:**
1. `{song}.ttml` - TTML format (Apple Music style, supports word sync)
2. `{song}.lrc` - Standard LRC format
3. `{song}.elrc` - Enhanced LRC (word-synced)
4. `{song}.txt` - Plain text

### TTML Example

```xml
<?xml version="1.0" encoding="UTF-8"?>
<tt xmlns="http://www.w3.org/ns/ttml" xml:lang="en">
  <body>
    <div>
      <p begin="10.000s" end="15.000s">
        <span begin="10.000s" end="12.000s">Hello</span>
        <span begin="12.000s" end="15.000s">world</span>
      </p>
    </div>
  </body>
</tt>
```

### LRC Example

```
[00:10.00]Line 1 lyrics here
[00:15.50]Line 2 lyrics here
[00:20.00]Line 3 lyrics here
```

## Client Integration

See `CLIENT_INTEGRATION.md` for examples of how to integrate this daemon into:
- Desktop clients (Electron/Rust)
- Web clients (TypeScript/Vue)
- Mobile clients (Kotlin/Swift)

## Docker

```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p aurelia-sidecar-daemon

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/aurelia-sidecar-daemon /usr/local/bin/
EXPOSE 8080
ENTRYPOINT ["aurelia-sidecar-daemon"]
```

## Building

```bash
# Build just the daemon
cargo build --release -p aurelia-sidecar-daemon

# Run with example config
cargo run -p aurelia-sidecar-daemon -- --config example-config.toml
```

## Architecture

```
┌─────────────────┐     ┌──────────────────┐     ┌──────────────┐
│  Aurelia Client │────▶│  Jellyfin Server │     │  Sidecar     │
│  (Any platform) │     │  (Music library) │     │  Files       │
└─────────────────┘     └──────────────────┘     └──────────────┘
         │                                              ▲
         │              ┌──────────────────┐            │
         └─────────────▶│  Lyrics Daemon   │────────────┘
                        │  (This service)  │
                        └──────────────────┘
                        - Reads sidecar files
                        - Queries Jellyfin for paths
                        - Caches results
                        - Serves via HTTP
```

## NixOS

A NixOS module is included in the flake. Add it to your configuration:

```nix
{
  inputs.aurelia.url = "github:aurelia-music/aurelia";

  outputs = { self, nixpkgs, aurelia, ... }: {
    nixosConfigurations.myserver = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        aurelia.nixosModules.aurelia-sidecar-daemon
        {
          services.aurelia-sidecar-daemon = {
            enable = true;
            settings = {
              jellyfin_url = "http://localhost:8096";
              music_paths = [ "/var/lib/jellyfin/media/music" ];
              bind = "127.0.0.1";
              port = 8080;
            };
            # Use environment file for API key
            environmentFile = "/var/secrets/aurelia-sidecar.env";
          };
        }
      ];
    };
  };
}
```

See `NIXOS.md` for complete documentation.

## License

Same as Aurelia (MPL-2.0)
