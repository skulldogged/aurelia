use super::*;

impl JellyfinClient {
    /// Fetch just the filesystem `Path` of an item from Jellyfin.
    ///
    /// Uses a lightweight `GET /Items/{id}?Fields=Path` request.
    pub async fn get_item_path(&self, item_id: &str) -> AppResult<Option<String>> {
        let url =
            utils::build_jellyfin_url(&self.server_url, &format!("/Items/{item_id}?Fields=Path"));

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let body: serde_json::Value = response.json().await?;
        Ok(body["Path"].as_str().map(ToString::to_string))
    }

    /// Get lyrics for a song
    pub async fn get_lyrics(&self, item_id: &str) -> AppResult<Option<JellyfinLyrics>> {
        let lyrics_url =
            utils::build_jellyfin_url(&self.server_url, &format!("/Audio/{item_id}/Lyrics"));

        tracing::info!("[Lyrics] GET {}", lyrics_url);

        let response = self
            .client
            .get(&lyrics_url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        let status = response.status();
        tracing::info!("[Lyrics] Response status: {}", status);

        if !status.is_success() {
            return Ok(None); // No lyrics available
        }

        // Read body as text first so we can log it on parse failure
        let body = response.text().await?;
        tracing::info!(
            "[Lyrics] Response body length: {} bytes, preview: {}",
            body.len(),
            &body[..body.len().min(300)]
        );

        match serde_json::from_str::<JellyfinLyrics>(&body) {
            Ok(lyrics) => Ok(Some(lyrics)),
            Err(e) => {
                tracing::error!("[Lyrics] Failed to parse Jellyfin lyrics JSON: {}", e);
                Ok(None)
            }
        }
    }

    /// Toggle favorite status for an item
    pub async fn toggle_favorite(
        &self,
        user_id: &str,
        item_id: &str,
        is_favorite: bool,
    ) -> AppResult<()> {
        let fav_url = utils::build_jellyfin_url(
            &self.server_url,
            &format!("/UserFavoriteItems/{item_id}?userId={user_id}"),
        );

        let response = if is_favorite {
            self.client
                .post(&fav_url)
                .header("Authorization", self.get_auth_header())
                .send()
                .await?
        } else {
            self.client
                .delete(&fav_url)
                .header("Authorization", self.get_auth_header())
                .send()
                .await?
        };

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to toggle favorite status: HTTP {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Get all favorite item IDs for the user
    pub async fn get_favorite_ids(&self, user_id: &str) -> AppResult<Vec<String>> {
        let mut all_ids = Vec::new();
        let mut start_index = 0;
        let page_size = 1000;

        loop {
            let query = format!(
                "/Items?userId={}&IsFavorite=true&StartIndex={}&Limit={}&Recursive=true",
                user_id, start_index, page_size
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
                    "Failed to get favorites: HTTP {}",
                    response.status()
                )));
            }

            let json: serde_json::Value = response.json().await?;
            let items = json["Items"].as_array().ok_or_else(|| {
                AppError::Network("Invalid favorites response".to_string())
            })?;

            if items.is_empty() {
                break;
            }

            for item in items {
                if let Some(id) = item["Id"].as_str() {
                    all_ids.push(id.to_string());
                }
            }

            let total = json["TotalRecordCount"].as_i64().unwrap_or(0) as usize;
            if start_index + items.len() >= total {
                break;
            }

            start_index += items.len();
        }

        Ok(all_ids)
    }

    /// Get audio stream URL for desktop (raw HTTP streaming with byte-range or startTimeTicks seeking).
    ///
    /// For seekable containers, returns a direct static stream.
    /// For non-seekable containers (ALAC, etc.), returns a transcoded AAC stream.
    /// The desktop player handles seeking via startTimeTicks on the raw stream.
    pub fn get_audio_stream_url(&self, item_id: &str, container: Option<&str>) -> String {
        let supports_seek = utils::supports_seeking(container);
        tracing::info!("[get_audio_stream_url] item_id: {}, container: {:?}, supports_seeking: {}", 
            item_id, container, supports_seek);
        
        let token = self.token.as_deref().unwrap_or("");
        if supports_seek {
            let url = format!(
                "{}?api_key={}&static=true",
                utils::build_jellyfin_url(&self.server_url, &format!("/Audio/{}/stream", item_id)),
                token
            );
            tracing::info!("[get_audio_stream_url] Using seekable URL: {}", &url[..url.len().min(100)]);
            url
        } else {
            let url = format!(
                "{}?api_key={}",
                utils::build_jellyfin_url(
                    &self.server_url,
                    &format!("/Audio/{}/stream.aac", item_id)
                ),
                token
            );
            tracing::info!("[get_audio_stream_url] Using transcoded URL: {}", &url[..url.len().min(100)]);
            url
        }
    }

    /// Get audio stream URL for mobile (ExoPlayer/Media3).
    ///
    /// For seekable containers, returns a direct static stream.
    /// For non-seekable containers (ALAC, etc.), uses the `/universal` endpoint which
    /// transcodes to AAC. Uses `transcodingProtocol=http` so ExoPlayer receives a
    /// progressive stream it can parse directly (not HLS which requires a special MediaSource).
    pub fn get_mobile_audio_stream_url(&self, item_id: &str, container: Option<&str>) -> String {
        let token = self.token.as_deref().unwrap_or("");
        if utils::supports_seeking(container) {
            format!(
                "{}?api_key={}&static=true",
                utils::build_jellyfin_url(&self.server_url, &format!("/Audio/{}/stream", item_id)),
                token
            )
        } else {
            format!(
                "{}?api_key={}\
                 &container=mp3,aac,m4a|aac,flac,ogg\
                 &transcodingContainer=aac\
                 &transcodingProtocol=http\
                 &audioCodec=aac\
                 &maxStreamingBitrate=999999999",
                utils::build_jellyfin_url(
                    &self.server_url,
                    &format!("/Audio/{}/universal", item_id)
                ),
                token
            )
        }
    }

    /// Register client capabilities with the Jellyfin server
    pub async fn register_capabilities(&self, capabilities: &ClientCapabilities) -> AppResult<()> {
        let capabilities_url =
            utils::build_jellyfin_url(&self.server_url, "/Sessions/Capabilities/Full");

        let request_body = serde_json::json!({
            "capabilities": capabilities
        });

        let response = self
            .client
            .post(&capabilities_url)
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "No response body".to_string());
            error!(
                "Failed to register capabilities: HTTP {} - Response body: {}",
                status, body
            );
            return Err(AppError::Network(format!(
                "Failed to register capabilities: HTTP {} - {}",
                status, body
            )));
        }

        debug!("Successfully registered client capabilities with Jellyfin server");
        Ok(())
    }

    /// Report playback start to the Jellyfin server
    pub async fn report_playback_start(
        &self,
        item_id: &str,
        position_ticks: Option<i64>,
    ) -> AppResult<()> {
        let playing_url = utils::build_jellyfin_url(&self.server_url, "/Sessions/Playing");

        let mut request_body = serde_json::json!({
            "ItemId": item_id,
            "CanSeek": true,
            "IsPaused": false,
            "IsMuted": false
        });

        if let Some(position) = position_ticks {
            request_body["PositionTicks"] = serde_json::json!(position);
        }

        let response = self
            .client
            .post(&playing_url)
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "No response body".to_string());
            return Err(AppError::Network(format!(
                "Failed to report playback start: HTTP {status} - {body}"
            )));
        }

        debug!("Successfully reported playback start for item: {}", item_id);
        Ok(())
    }

    /// Report playback progress to the Jellyfin server
    pub async fn report_playback_progress(
        &self,
        item_id: &str,
        position_ticks: Option<i64>,
        event_name: Option<&str>,
        is_paused: Option<bool>,
    ) -> AppResult<()> {
        let progress_url =
            utils::build_jellyfin_url(&self.server_url, "/Sessions/Playing/Progress");

        let mut request_body = serde_json::json!({
            "ItemId": item_id,
            "IsMuted": false
        });

        if let Some(position) = position_ticks {
            request_body["PositionTicks"] = serde_json::json!(position);
        }

        if let Some(event) = event_name {
            request_body["EventName"] = serde_json::json!(event);
        }

        if let Some(paused) = is_paused {
            request_body["IsPaused"] = serde_json::json!(paused);
        } else {
            request_body["IsPaused"] = serde_json::json!(false);
        }

        let response = self
            .client
            .post(&progress_url)
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to report playback progress: HTTP {}",
                response.status()
            )));
        }

        debug!(
            "Successfully reported playback progress for item: {}",
            item_id
        );
        Ok(())
    }

    /// Report playback stop to the Jellyfin server
    pub async fn report_playback_stop(
        &self,
        item_id: &str,
        position_ticks: Option<i64>,
    ) -> AppResult<()> {
        let stopped_url = utils::build_jellyfin_url(&self.server_url, "/Sessions/Playing/Stopped");

        let mut request_body = serde_json::json!({
            "ItemId": item_id,
            "IsPaused": false,
            "IsMuted": false
        });

        if let Some(position) = position_ticks {
            request_body["PositionTicks"] = serde_json::json!(position);
        }

        let response = self
            .client
            .post(&stopped_url)
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to report playback stop: HTTP {}",
                response.status()
            )));
        }

        debug!("Successfully reported playback stop for item: {}", item_id);
        Ok(())
    }

    /// Mark item as played (update play count and last played date)
    pub async fn mark_item_played(&self, user_id: &str, item_id: &str) -> AppResult<()> {
        let played_url = utils::build_jellyfin_url(
            &self.server_url,
            &format!("/UserPlayedItems/{item_id}?userId={user_id}"),
        );

        let response = self
            .client
            .post(&played_url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to mark item as played: HTTP {}",
                response.status()
            )));
        }

        debug!(
            "Successfully marked item {} as played for user {}",
            item_id, user_id
        );
        Ok(())
    }
}
