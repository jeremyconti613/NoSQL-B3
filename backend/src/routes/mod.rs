pub mod artists;
pub mod graph;
pub mod import;
pub mod recordings;
pub mod releases;
pub mod search;
pub mod stats;

use axum::{
    Router,
    routing::{get, post},
};
use serde::Deserialize;

use crate::musicbrainz::MusicBrainzClient;
use crate::repo::Repo;

#[derive(Clone)]
pub struct AppState {
    pub repo: Repo,
    pub mb: MusicBrainzClient,
}

/// Shared `?limit=&offset=` query params for list endpoints.
#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Pagination {
    pub fn limit_or(&self, default: i64) -> i64 {
        self.limit.unwrap_or(default).clamp(1, 500)
    }

    pub fn offset_or(&self, default: i64) -> i64 {
        self.offset.unwrap_or(default).max(0)
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/artists", get(artists::list_artists))
        .route("/api/artists/{id}", get(artists::get_artist))
        .route("/api/artists/{id}/recordings", get(artists::get_artist_recordings))
        .route("/api/artists/{id}/releases", get(artists::get_artist_releases))
        .route(
            "/api/artists/{id}/collaborations",
            get(artists::get_artist_collaborations),
        )
        .route("/api/search/artists", get(search::search_artists))
        .route("/api/import/artists", post(import::import_artist))
        .route("/api/recordings", get(recordings::list_recordings))
        .route("/api/recordings/{id}", get(recordings::get_recording))
        .route("/api/recordings/{id}/artists", get(recordings::get_recording_artists))
        .route("/api/recordings/{id}/releases", get(recordings::get_recording_releases))
        .route("/api/releases", get(releases::list_releases))
        .route("/api/releases/{id}", get(releases::get_release))
        .route("/api/releases/{id}/recordings", get(releases::get_release_recordings))
        .route("/api/releases/{id}/artists", get(releases::get_release_artists))
        .route("/api/graph", get(graph::graph_full))
        .route("/api/graph/artists/{id}", get(graph::graph_for_artist))
        .route("/api/graph/collaborations", get(graph::graph_collaborations))
        .route("/api/stats/overview", get(stats::overview))
        .route("/api/stats/top-collaborations", get(stats::top_collaborations))
        .route("/api/stats/top-artists", get(stats::top_artists))
        .route("/api/stats/top-genres", get(stats::top_genres))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}
