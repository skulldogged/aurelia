#!/usr/bin/env python3
"""
Fetch word-synced (syllable) TTML lyrics from Apple Music for an entire album.

Usage:
    uv run --with httpx python scripts/fetch-syllable-lyrics.py <apple-music-album-url> [--output-dir DIR] [--cookies PATH]

Examples:
    # Save next to your music files
    uv run --with httpx python scripts/fetch-syllable-lyrics.py \
        "https://music.apple.com/us/album/debi-tirar-mas-fotos/1787022561" \
        --output-dir "/mnt/music/Bad Bunny/DeBÍ TiRAR MáS FOToS"

    # Save to current directory (default)
    uv run --with httpx python scripts/fetch-syllable-lyrics.py \
        "https://music.apple.com/us/album/debi-tirar-mas-fotos/1787022561"

Requires a cookies.txt (Netscape format) with a valid Apple Music `media-user-token`.
"""

from __future__ import annotations

import argparse
import asyncio
import re
import sys
from http.cookiejar import MozillaCookieJar
from pathlib import Path

import httpx

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

AMP_API_URL = "https://amp-api.music.apple.com"
APPLE_MUSIC_HOMEPAGE = "https://music.apple.com"
COOKIE_DOMAIN = ".music.apple.com"

ALBUM_URL_RE = re.compile(
    r"https://music\.apple\.com"
    r"/(?P<storefront>[a-z]{2})"
    r"/album"
    r"(?:/[^\s/]+)?"
    r"/(?P<id>[0-9]+)"
    r"(?:\?i=(?P<song_id>[0-9]+))?"
)

# Characters illegal in filenames on most OSes
ILLEGAL_CHARS_RE = re.compile(r'[\\/:*?"<>|]')


def sanitize_filename(name: str) -> str:
    return ILLEGAL_CHARS_RE.sub("_", name)


# ---------------------------------------------------------------------------
# Apple Music API helpers
# ---------------------------------------------------------------------------

async def get_developer_token(client: httpx.AsyncClient) -> str:
    """Scrape the developer JWT from the Apple Music web app."""
    resp = await client.get(APPLE_MUSIC_HOMEPAGE)
    resp.raise_for_status()

    # Find the index JS bundle
    m = re.search(r"/(assets/index-legacy[~-][^/\"]+\.js)", resp.text)
    if not m:
        raise RuntimeError("Could not find index.js URI in Apple Music homepage")

    js_resp = await client.get(f"{APPLE_MUSIC_HOMEPAGE}/{m.group(1)}")
    js_resp.raise_for_status()

    tok = re.search(r'(?=eyJh)(.*?)(?=")', js_resp.text)
    if not tok:
        raise RuntimeError("Could not extract developer token from index.js")
    return tok.group(1)


async def create_client(cookies_path: str) -> tuple[httpx.AsyncClient, str]:
    """Create an authenticated httpx client.  Returns (client, storefront)."""
    jar = MozillaCookieJar(cookies_path)
    jar.load(ignore_discard=True, ignore_expires=True)

    media_user_token = next(
        (c.value for c in jar if c.name == "media-user-token" and c.domain == COOKIE_DOMAIN),
        None,
    )
    if not media_user_token:
        raise ValueError(
            "media-user-token not found in cookies. "
            "Export cookies from the Apple Music website while logged in."
        )

    client = httpx.AsyncClient(
        headers={
            "accept": "*/*",
            "origin": APPLE_MUSIC_HOMEPAGE,
            "referer": APPLE_MUSIC_HOMEPAGE,
            "user-agent": (
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                "AppleWebKit/537.36 (KHTML, like Gecko) "
                "Chrome/137.0.0.0 Safari/537.36"
            ),
        },
        params={"l": "en-US"},
        follow_redirects=True,
        timeout=60.0,
    )

    token = await get_developer_token(client)
    client.headers["authorization"] = f"Bearer {token}"
    client.cookies.set("media-user-token", media_user_token)

    # Resolve the user's storefront
    acct = await client.get(f"{AMP_API_URL}/v1/me/account", params={"meta": "subscription"})
    acct.raise_for_status()
    acct_data = acct.json()
    storefront = acct_data["meta"]["subscription"]["storefront"]

    active = acct_data["meta"]["subscription"].get("active", False)
    if not active:
        print("WARNING: Apple Music subscription does not appear active", file=sys.stderr)

    return client, storefront


async def get_album_tracks(
    client: httpx.AsyncClient,
    storefront: str,
    album_id: str,
) -> list[dict]:
    """Fetch all tracks on an album with syllable-lyrics included."""
    resp = await client.get(
        f"{AMP_API_URL}/v1/catalog/{storefront}/albums/{album_id}",
        params={
            "include": "tracks",
            "extend": "extendedAssetUrls",
        },
    )
    resp.raise_for_status()
    data = resp.json()

    if "data" not in data or len(data["data"]) == 0:
        raise RuntimeError(f"Album {album_id} not found")

    album = data["data"][0]
    album_name = album["attributes"]["name"]
    album_artist = album["attributes"]["artistName"]
    print(f"Album: {album_name} by {album_artist}")

    tracks_rel = album.get("relationships", {}).get("tracks", {})
    tracks = tracks_rel.get("data", [])

    # Follow pagination if needed
    next_url = tracks_rel.get("next")
    while next_url:
        resp = await client.get(AMP_API_URL + next_url)
        resp.raise_for_status()
        page = resp.json()
        tracks.extend(page.get("data", []))
        next_url = page.get("next")

    print(f"Found {len(tracks)} tracks")
    return tracks


async def fetch_syllable_lyrics(
    client: httpx.AsyncClient,
    storefront: str,
    song_id: str,
) -> str | None:
    """Fetch word-synced TTML for a single song.  Returns TTML string or None."""
    resp = await client.get(
        f"{AMP_API_URL}/v1/catalog/{storefront}/songs/{song_id}",
        params={
            "include": "syllable-lyrics",
        },
    )
    if resp.status_code == 404:
        return None
    resp.raise_for_status()

    data = resp.json()
    song = data["data"][0]
    syl = song.get("relationships", {}).get("syllable-lyrics", {})

    if (
        syl
        and "data" in syl
        and len(syl["data"]) > 0
        and "attributes" in syl["data"][0]
        and "ttml" in syl["data"][0]["attributes"]
    ):
        return syl["data"][0]["attributes"]["ttml"]

    return None


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

async def main():
    parser = argparse.ArgumentParser(
        description="Fetch word-synced (syllable) TTML lyrics from Apple Music for an album.",
    )
    parser.add_argument(
        "url",
        help="Apple Music album URL (e.g. https://music.apple.com/us/album/name/1234567890)",
    )
    parser.add_argument(
        "--output-dir", "-o",
        help="Directory to save .ttml files (default: current directory)",
        default=".",
    )
    parser.add_argument(
        "--cookies", "-c",
        help="Path to cookies.txt (default: ~/gamdl/cookies.txt)",
        default=str(Path.home() / "gamdl" / "cookies.txt"),
    )
    parser.add_argument(
        "--filename-template",
        help=(
            'Filename template. Available: {track}, {title}, {artist}, {disc}. '
            'Default: "{track:02d}. {title}.ttml"'
        ),
        default="{track:02d}. {title}.ttml",
    )
    args = parser.parse_args()

    # Parse URL
    m = ALBUM_URL_RE.match(args.url)
    if not m:
        print(f"ERROR: Not a valid Apple Music album URL: {args.url}", file=sys.stderr)
        sys.exit(1)

    album_id = m.group("id")
    single_song_id = m.group("song_id")  # ?i=... for a specific track

    # Authenticate
    print("Authenticating with Apple Music...")
    client, storefront = await create_client(args.cookies)
    print(f"Storefront: {storefront}")

    # Fetch album tracks
    tracks = await get_album_tracks(client, storefront, album_id)

    # If URL had ?i=songId, filter to just that track
    if single_song_id:
        tracks = [t for t in tracks if t["id"] == single_song_id]
        if not tracks:
            print(f"ERROR: Song {single_song_id} not found in album", file=sys.stderr)
            sys.exit(1)

    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Fetch lyrics for each track
    succeeded = 0
    failed = 0
    skipped = 0

    for track in tracks:
        attrs = track["attributes"]
        name = attrs["name"]
        artist = attrs["artistName"]
        track_num = attrs.get("trackNumber", 0)
        disc_num = attrs.get("discNumber", 1)
        song_id = track["id"]

        print(f"\n[{track_num:2d}] {name} — {artist}")
        print(f"     Song ID: {song_id}", end="")

        ttml = await fetch_syllable_lyrics(client, storefront, song_id)

        if ttml is None:
            print(" — no syllable lyrics available")
            skipped += 1
            continue

        is_word = 'timing="Word"' in ttml
        print(f" — {'word-synced' if is_word else 'line-synced only'} ({len(ttml)} chars)")

        if not is_word:
            print("     WARNING: Only line-synced lyrics available for this track")

        # Build filename
        safe_title = sanitize_filename(name)
        safe_artist = sanitize_filename(artist)
        filename = args.filename_template.format(
            track=track_num,
            title=safe_title,
            artist=safe_artist,
            disc=disc_num,
        )

        out_path = output_dir / filename
        out_path.write_text(ttml, encoding="utf-8")
        print(f"     Saved: {out_path}")
        succeeded += 1

    print(f"\nDone! {succeeded} saved, {skipped} skipped (no lyrics), {failed} failed")
    await client.aclose()


if __name__ == "__main__":
    asyncio.run(main())
