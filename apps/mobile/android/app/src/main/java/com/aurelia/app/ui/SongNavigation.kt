package com.aurelia.app.ui

import uniffi.aurelia_core.Song

internal fun Song.safeAlbumId(): String? = albumId?.takeIf { it.isNotBlank() }

internal fun Song.safePrimaryArtistId(): String? = artistIds?.firstOrNull()?.takeIf { it.isNotBlank() }
