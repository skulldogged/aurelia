# Aurelia Web

Self-hosted web version of Aurelia music player.

## Architecture

The web version uses:
- **Backend**: Rust Axum server (reuses `aurelia-core` directly)
- **Frontend**: Vue 3 app using Web Audio API (shares code with desktop)
- **Database**: Same redb database as desktop
- **Auth**: Session cookies
- **Audio**: Browser streams directly from Jellyfin URLs

## Project Structure

```
apps/web/
├── backend/          # Axum HTTP server + WebSocket
└── frontend/         # Vue 3 web app
```

## Getting Started

### Prerequisites

- Rust toolchain
- Bun

### Running the Backend

```bash
cd apps/web/backend
cargo run
```

The backend will start on `http://localhost:3000`.

### Running the Frontend

```bash
cd apps/web/frontend
# First time: install dependencies
bun install

# Run dev server
bun run dev
```

The frontend will start on `http://localhost:5173` and proxy API calls to the backend.

## API Endpoints

- `POST /api/auth/login` - Authenticate with Jellyfin
- `GET /api/auth/credentials` - Get saved credentials
- `GET /api/library` - Get library data
- `POST /api/library/sync` - Sync with Jellyfin
- `GET /api/playlists` - Get playlists
- `POST /api/audio/stream-url` - Get Jellyfin stream URL
- `WS /ws` - WebSocket for real-time updates

## Differences from Desktop

1. **Audio**: Uses Web Audio API instead of Rust audio player
2. **Images**: Browser loads directly from Jellyfin (no local caching)
3. **Window Controls**: None (browser handles this)
4. **Discord/Last.fm**: Handled by backend or direct browser calls
