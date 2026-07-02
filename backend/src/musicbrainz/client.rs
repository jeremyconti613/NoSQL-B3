use std::sync::Arc;
use std::time::Duration;

use reqwest::StatusCode;
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::error::{AppError, AppResult};

use super::models::{
    CoverArtResponse, MbArtist, MbArtistSearchResponse, MbRecording, MbRecordingSearchResponse,
    MbRelease,
};

/// MusicBrainz allows ~1 request/second per client. We enforce this
/// ourselves (rather than trusting callers) by serializing every request
/// through a shared "last request" timestamp guarded by a mutex, sleeping
/// as needed before firing the next one.
const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(1100);
const MAX_RETRIES: u32 = 3;

#[derive(Clone)]
pub struct MusicBrainzClient {
    http: reqwest::Client,
    base_url: String,
    last_request_at: Arc<Mutex<Option<Instant>>>,
}

impl MusicBrainzClient {
    pub fn new(base_url: impl Into<String>, user_agent: impl Into<String>) -> AppResult<Self> {
        let http = reqwest::Client::builder()
            .user_agent(user_agent.into())
            .timeout(Duration::from_secs(15))
            .build()?;

        Ok(Self {
            http,
            base_url: base_url.into(),
            last_request_at: Arc::new(Mutex::new(None)),
        })
    }

    /// Blocks until at least `MIN_REQUEST_INTERVAL` has elapsed since the
    /// previous MusicBrainz request, then reserves the current instant as
    /// the new "last request" time.
    async fn throttle(&self) {
        let mut last = self.last_request_at.lock().await;
        if let Some(prev) = *last {
            let elapsed = prev.elapsed();
            if elapsed < MIN_REQUEST_INTERVAL {
                tokio::time::sleep(MIN_REQUEST_INTERVAL - elapsed).await;
            }
        }
        *last = Some(Instant::now());
    }

    /// GETs a MusicBrainz endpoint (JSON), applying rate limiting and
    /// retrying transient failures (503 / network errors) with backoff.
    async fn get_json<T: serde::de::DeserializeOwned>(&self, path_and_query: &str) -> AppResult<T> {
        let url = format!("{}{}", self.base_url, path_and_query);

        let mut attempt = 0;
        loop {
            self.throttle().await;
            attempt += 1;

            let result = self.http.get(&url).send().await;

            match result {
                Ok(resp) if resp.status() == StatusCode::SERVICE_UNAVAILABLE && attempt < MAX_RETRIES => {
                    let backoff = Duration::from_millis(500 * 2u64.pow(attempt));
                    tracing::warn!(%url, attempt, "MusicBrainz 503, backing off");
                    tokio::time::sleep(backoff).await;
                    continue;
                }
                Ok(resp) if !resp.status().is_success() => {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    return Err(AppError::MusicBrainzApi(format!(
                        "MusicBrainz returned {status} for {url}: {body}"
                    )));
                }
                Ok(resp) => {
                    return resp.json::<T>().await.map_err(|e| {
                        AppError::MusicBrainzApi(format!("failed to parse response from {url}: {e}"))
                    });
                }
                Err(e) if attempt < MAX_RETRIES => {
                    tracing::warn!(%url, attempt, error = %e, "MusicBrainz request failed, retrying");
                    let backoff = Duration::from_millis(500 * 2u64.pow(attempt));
                    tokio::time::sleep(backoff).await;
                    continue;
                }
                Err(e) => return Err(AppError::MusicBrainz(e)),
            }
        }
    }

    /// `GET /artist?query=...` — free-text artist search.
    pub async fn search_artists(&self, query: &str, limit: u32) -> AppResult<MbArtistSearchResponse> {
        let q = urlencoding::encode(query);
        self.get_json(&format!("/artist?query={q}&limit={limit}&fmt=json"))
            .await
    }

    /// `GET /artist/{mbid}?inc=genres+artist-rels` — full artist detail.
    /// `area` is *not* requested via `inc`: MusicBrainz includes it by
    /// default on the single-artist lookup and rejects `area` as an `inc`
    /// value here (unlike the artist *search* endpoint).
    pub async fn get_artist(&self, mbid: &str) -> AppResult<MbArtist> {
        self.get_json(&format!("/artist/{mbid}?inc=genres+artist-rels&fmt=json"))
            .await
    }

    /// `GET /recording?artist={mbid}&inc=artist-credits` — a lightweight page
    /// of recordings this artist performed or is credited on. The MusicBrainz
    /// *browse* endpoint (querying recordings by `artist=`) does not support
    /// `inc=releases`/`release-groups`/`labels` at all (only direct
    /// single-recording lookups do) — use [`Self::get_recording`] to enrich
    /// each one with its releases.
    pub async fn get_artist_recordings(
        &self,
        mbid: &str,
        limit: u32,
    ) -> AppResult<MbRecordingSearchResponse> {
        self.get_json(&format!(
            "/recording?artist={mbid}&inc=artist-credits&limit={limit}&fmt=json"
        ))
        .await
    }

    /// `GET /recording/{mbid}?inc=artist-credits+releases` — full recording
    /// detail including the releases it appears on (with their own
    /// `release-group` for release type, but not label info — see
    /// [`Self::get_release`] for that).
    pub async fn get_recording(&self, mbid: &str) -> AppResult<MbRecording> {
        self.get_json(&format!("/recording/{mbid}?inc=artist-credits+releases&fmt=json"))
            .await
    }

    /// `GET /release/{mbid}?inc=labels+release-groups` — full release detail,
    /// including label info, used to enrich the (label-less) release objects
    /// nested under a recording lookup.
    pub async fn get_release(&self, mbid: &str) -> AppResult<MbRelease> {
        self.get_json(&format!("/release/{mbid}?inc=labels+release-groups&fmt=json"))
            .await
    }

    /// Cover Art Archive front cover URL for a release, if any (bonus feature).
    /// This hits a different host than MusicBrainz itself so it is not subject
    /// to the same rate limit, but we still throttle it defensively.
    pub async fn get_cover_art_url(&self, release_mbid: &str) -> Option<String> {
        self.throttle().await;
        let url = format!("https://coverartarchive.org/release/{release_mbid}");
        let resp = self.http.get(&url).send().await.ok()?;
        if !resp.status().is_success() {
            return None;
        }
        let parsed: CoverArtResponse = resp.json().await.ok()?;
        parsed
            .images
            .into_iter()
            .find(|img| img.front)
            .and_then(|img| img.image)
    }
}
