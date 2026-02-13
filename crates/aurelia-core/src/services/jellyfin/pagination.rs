use super::*;

impl JellyfinClient {
    // =========================================================================
    // Smart Sync API Methods
    // =========================================================================

    /// Fetch a single page of items from the Jellyfin Items endpoint.
    /// Returns raw JSON items, total count, and the server's Date header.
    pub async fn fetch_items_page(
        &self,
        base_query: &str,
        start_index: usize,
        limit: usize,
    ) -> AppResult<PaginatedResponse> {
        let query = format!(
            "{base_query}&StartIndex={start_index}&Limit={limit}&enableTotalRecordCount=true"
        );
        let url = utils::build_jellyfin_url(&self.server_url, &query);

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to fetch items page (startIndex={start_index}): HTTP {}",
                response.status()
            )));
        }

        // Capture the server's Date header for clock-skew-safe sync timestamps
        let server_date = response
            .headers()
            .get("date")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let response_json: serde_json::Value = response.json().await?;

        let total_record_count = response_json["TotalRecordCount"].as_u64().unwrap_or(0) as usize;

        let items = response_json["Items"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        Ok(PaginatedResponse {
            items,
            total_record_count,
            server_date,
        })
    }

    /// Fetch songs with pagination and optional incremental date filter.
    /// When `since_date` is Some, only items modified since that date are returned.
    /// Returns (songs, server_date_header).
    pub async fn get_songs_paginated(
        &self,
        user_id: &str,
        since_date: Option<&str>,
        page_size: usize,
    ) -> AppResult<(Vec<Song>, Option<String>)> {
        let mut base_query = format!(
            "/Items?userId={user_id}&IncludeItemTypes=Audio&Recursive=true&Fields=Genres,DateCreated,DateLastModified,MediaSources,ParentId,People,Tags,Path,RunTimeTicks,ImageTags,AlbumId,Artists,Album,ProductionYear,UserData,IndexNumber,PremiereDate,AlbumArtists,MediaStreams"
        );

        append_incremental_date_filter(&mut base_query, since_date);

        let mut all_songs = Vec::new();
        let mut start_index = 0;
        let mut server_date = None;

        loop {
            let page = self
                .fetch_items_page(&base_query, start_index, page_size)
                .await?;

            if server_date.is_none() {
                server_date = page.server_date;
            }

            if page.items.is_empty() {
                break;
            }

            let page_count = page.items.len();
            for item in &page.items {
                let song = self.parse_single_music_item(item)?;
                all_songs.push(song);
            }

            start_index += page_count;

            info!(
                "Fetched songs page: {}/{} (page_size={})",
                start_index, page.total_record_count, page_size
            );

            if start_index >= page.total_record_count {
                break;
            }
        }

        info!("Total songs fetched: {}", all_songs.len());
        Ok((all_songs, server_date))
    }

    /// Fetch albums with pagination and optional incremental date filter.
    /// Returns (albums, server_date_header).
    pub async fn get_albums_paginated(
        &self,
        user_id: &str,
        since_date: Option<&str>,
        page_size: usize,
    ) -> AppResult<(Vec<crate::models::Album>, Option<String>)> {
        let mut base_query = format!(
            "/Items?userId={user_id}&IncludeItemTypes=MusicAlbum&Recursive=true&Fields=ImageTags,Overview,ProductionYear,CommunityRating,Artists,ProviderIds,DateCreated,DateLastModified"
        );

        append_incremental_date_filter(&mut base_query, since_date);

        let mut all_albums = Vec::new();
        let mut start_index = 0;
        let mut server_date = None;

        loop {
            let page = self
                .fetch_items_page(&base_query, start_index, page_size)
                .await?;

            if server_date.is_none() {
                server_date = page.server_date;
            }

            if page.items.is_empty() {
                break;
            }

            let page_count = page.items.len();
            for item in &page.items {
                let album = self.parse_single_album(item);
                all_albums.push(album);
            }

            start_index += page_count;

            info!(
                "Fetched albums page: {}/{} (page_size={})",
                start_index, page.total_record_count, page_size
            );

            if start_index >= page.total_record_count {
                break;
            }
        }

        info!("Total albums fetched: {}", all_albums.len());
        Ok((all_albums, server_date))
    }

    /// Fetch artists with pagination and optional incremental date filter.
    /// Returns (artists, server_date_header).
    pub async fn get_artists_paginated(
        &self,
        user_id: &str,
        since_date: Option<&str>,
        page_size: usize,
    ) -> AppResult<(Vec<Artist>, Option<String>)> {
        let mut base_query = format!(
            "/Items?userId={user_id}&IncludeItemTypes=MusicArtist&Recursive=true&Fields=ImageTags,Overview,ProviderIds,CommunityRating,DateLastModified"
        );

        append_incremental_date_filter(&mut base_query, since_date);

        let mut all_artists = Vec::new();
        let mut start_index = 0;
        let mut server_date = None;

        loop {
            let page = self
                .fetch_items_page(&base_query, start_index, page_size)
                .await?;

            if server_date.is_none() {
                server_date = page.server_date;
            }

            if page.items.is_empty() {
                break;
            }

            let page_count = page.items.len();
            for item in &page.items {
                let artist = self.parse_single_artist(item)?;
                all_artists.push(artist);
            }

            start_index += page_count;

            info!(
                "Fetched artists page: {}/{} (page_size={})",
                start_index, page.total_record_count, page_size
            );

            if start_index >= page.total_record_count {
                break;
            }
        }

        info!("Total artists fetched: {}", all_artists.len());
        Ok((all_artists, server_date))
    }

    /// Fetch only IDs for a given item type (lightweight, for deletion detection).
    /// Uses large pages since each item is just an ID string (~36 bytes).
    pub async fn get_all_item_ids(&self, user_id: &str, item_type: &str) -> AppResult<Vec<String>> {
        // Request no fields - just IDs (Jellyfin always returns Id)
        let base_query =
            format!("/Items?userId={user_id}&IncludeItemTypes={item_type}&Recursive=true&Fields=");

        let mut all_ids = Vec::new();
        let mut start_index = 0;
        let page_size = 2000; // Large page since payloads are tiny

        loop {
            let page = self
                .fetch_items_page(&base_query, start_index, page_size)
                .await?;

            if page.items.is_empty() {
                break;
            }

            let page_count = page.items.len();
            for item in &page.items {
                if let Some(id) = item["Id"].as_str() {
                    all_ids.push(id.to_string());
                }
            }

            start_index += page_count;

            if start_index >= page.total_record_count {
                break;
            }
        }

        info!(
            "Fetched {} {} IDs for deletion check",
            all_ids.len(),
            item_type
        );
        Ok(all_ids)
    }

    /// Parse the HTTP Date header into an RFC3339 timestamp.
    /// Falls back to local UTC time if parsing fails.
    pub fn parse_server_date(date_header: Option<&str>) -> String {
        if let Some(header) = date_header {
            // HTTP Date format: "Wed, 09 Jun 2021 10:18:14 GMT"
            if let Ok(parsed) = chrono::DateTime::parse_from_rfc2822(header) {
                return parsed.to_rfc3339();
            }
        }
        // Fallback to local clock
        chrono::Utc::now().to_rfc3339()
    }
}
