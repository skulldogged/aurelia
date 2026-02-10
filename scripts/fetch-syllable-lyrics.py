#!/usr/bin/env python3
"""
Fetch word-synced (syllable) TTML lyrics from Apple Music for an entire album,
optionally with embedded translations.

Usage:
    uv run --with httpx python scripts/fetch-syllable-lyrics.py <apple-music-album-url> [--output-dir DIR]

Examples:
    # Basic: fetch lyrics only
    uv run --with httpx python scripts/fetch-syllable-lyrics.py \
        "https://music.apple.com/us/album/debi-tirar-mas-fotos/1787022393" \
        --output-dir "/mnt/music/Bad Bunny/DeBÍ TiRAR MáS FOToS"

    # With English translations embedded in the TTML
    uv run --with httpx python scripts/fetch-syllable-lyrics.py \
        "https://music.apple.com/us/album/debi-tirar-mas-fotos/1787022393" \
        --output-dir "/tmp/badbunny" --translate en

    # Single track
    uv run --with httpx python scripts/fetch-syllable-lyrics.py \
        "https://music.apple.com/us/album/debi-tirar-mas-fotos/1787022393?i=1787022572"

Requires a cookies.txt (Netscape format) with a valid Apple Music `media-user-token`.
"""

from __future__ import annotations

import argparse
import asyncio
import json
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

# ISO 639-1 language code to IETF script tag mapping for l[script] param.
# Apple uses these to specify the writing system for the translation.
LANG_SCRIPT_MAP = {
    "en": "en-Latn",
    "es": "es-Latn",
    "fr": "fr-Latn",
    "de": "de-Latn",
    "it": "it-Latn",
    "pt": "pt-Latn",
    "ja": "ja-Jpan",
    "ko": "ko-Kore",
    "zh": "zh-Hans",
    "zh-tw": "zh-Hant",
    "ar": "ar-Arab",
    "ru": "ru-Cyrl",
    "hi": "hi-Deva",
    "th": "th-Thai",
    "vi": "vi-Latn",
    "tr": "tr-Latn",
    "pl": "pl-Latn",
    "nl": "nl-Latn",
    "sv": "sv-Latn",
    "uk": "uk-Cyrl",
}

# Characters illegal in filenames on most OSes
ILLEGAL_CHARS_RE = re.compile(r'[\/:*?"<>|]')


def sanitize_filename(name: str) -> str:
    return ILLEGAL_CHARS_RE.sub("_", name)


def guess_script_tag(lang: str) -> str:
    """Guess the l[script] value for a given language code."""
    lang_lower = lang.lower().replace("_", "-")
    # Try exact match first (e.g. "zh-tw")
    if lang_lower in LANG_SCRIPT_MAP:
        return LANG_SCRIPT_MAP[lang_lower]
    # Try base language (e.g. "en" from "en-us")
    base = lang_lower.split("-")[0]
    if base in LANG_SCRIPT_MAP:
        return LANG_SCRIPT_MAP[base]
    # Default: assume Latin script
    return f"{base}-Latn"


# ---------------------------------------------------------------------------
# Apple Music API helpers
# ---------------------------------------------------------------------------

async def get_developer_token(client: httpx.AsyncClient) -> str:
    """Scrape the developer JWT from the Apple Music web app."""
    resp = await client.get(APPLE_MUSIC_HOMEPAGE)
    resp.raise_for_status()

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
    """Fetch all tracks on an album."""
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
    translate_lang: str | None = None,
    dump_json: Path | None = None,
) -> dict | None:
    """
    Fetch word-synced TTML for a single song.

    If translate_lang is set (e.g. "en"), fetches the localized TTML which
    contains both original lyrics and translations embedded in
    <iTunesMetadata><translations>.

    Returns dict with 'ttml', 'has_translations', 'translation_lang' keys,
    or None if no lyrics available.
    """
    # Use the direct /syllable-lyrics sub-resource endpoint.
    # With extend=ttmlLocalizations and l[lyrics]/l[script], the API returns
    # a TTML with translations embedded in <translations><translation>.
    url = f"{AMP_API_URL}/v1/catalog/{storefront}/songs/{song_id}/syllable-lyrics"

    params: dict[str, str] = {}
    if translate_lang:
        lang_lower = translate_lang.lower().replace("_", "-")
        # l[lyrics] wants a locale like "en-us", l[script] wants "en-Latn"
        locale = lang_lower if "-" in lang_lower else f"{lang_lower}-{lang_lower}"
        params["l[lyrics]"] = locale
        params["l[script]"] = guess_script_tag(translate_lang)
        params["extend"] = "ttmlLocalizations"

    resp = await client.get(url, params=params)
    if resp.status_code == 404:
        return None
    resp.raise_for_status()

    data = resp.json()

    if dump_json:
        dump_path = dump_json / f"{song_id}.json"
        dump_path.write_text(json.dumps(data, indent=2, ensure_ascii=False), encoding="utf-8")

    if "data" not in data or not data["data"]:
        return None

    attrs = data["data"][0].get("attributes", {})

    # When requesting translations, the TTML is in 'ttmlLocalizations'.
    # Without translations, it's in 'ttml'.
    ttml = attrs.get("ttmlLocalizations") or attrs.get("ttml")
    if not ttml:
        return None

    # Check if translations are actually present in the TTML
    has_translations = bool(re.search(
        r'<translations[^/]*>(.+?)</translations>', ttml, re.DOTALL
    ))

    # Extract translation language from the TTML if present
    trans_lang = None
    if has_translations:
        m = re.search(r'<translation[^>]*xml:lang="([^"]+)"', ttml)
        if m:
            trans_lang = m.group(1)

    return {
        "ttml": ttml,
        "has_translations": has_translations,
        "translation_lang": trans_lang,
    }


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
    parser.add_argument(
        "--translate", "-t",
        metavar="LANG",
        help=(
            "Fetch lyrics with translations embedded in the TTML. "
            "LANG is a language code like 'en', 'ja', 'ko', 'fr', etc."
        ),
    )
    parser.add_argument(
        "--dump-json",
        action="store_true",
        help="Save raw API JSON responses alongside .ttml files.",
    )
    args = parser.parse_args()

    # Parse URL
    m = ALBUM_URL_RE.match(args.url)
    if not m:
        print(f"ERROR: Not a valid Apple Music album URL: {args.url}", file=sys.stderr)
        sys.exit(1)

    album_id = m.group("id")
    single_song_id = m.group("song_id")

    # Authenticate
    print("Authenticating with Apple Music...")
    client, storefront = await create_client(args.cookies)
    print(f"Storefront: {storefront}")

    if args.translate:
        print(f"Translation: {args.translate}")

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

    dump_dir = output_dir / "_json" if args.dump_json else None
    if dump_dir:
        dump_dir.mkdir(parents=True, exist_ok=True)

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

        result = await fetch_syllable_lyrics(
            client, storefront, song_id,
            translate_lang=args.translate,
            dump_json=dump_dir,
        )

        if not result:
            print("     No syllable lyrics available")
            skipped += 1
            continue

        ttml = result["ttml"]
        is_word = 'timing="Word"' in ttml
        sync_type = "word-synced" if is_word else "line-synced"

        if result["has_translations"]:
            trans_info = f" + {result['translation_lang']} translation"
        else:
            trans_info = ""
            if args.translate:
                trans_info = " (no translation available)"

        print(f"     {sync_type} ({len(ttml)} chars){trans_info}")

        # Build filename and save
        safe_title = sanitize_filename(name)
        safe_artist = sanitize_filename(artist)
        filename = args.filename_template.format(
            track=track_num,
            title=safe_title,
            artist=safe_artist,
            disc=disc_num,
        )

        save_path = output_dir / filename
        save_path.write_text(ttml, encoding="utf-8")
        print(f"     Saved: {save_path}")
        succeeded += 1

    print(f"\nDone! {succeeded} saved, {skipped} skipped (no lyrics), {failed} failed")
    await client.aclose()


if __name__ == "__main__":
    asyncio.run(main())
