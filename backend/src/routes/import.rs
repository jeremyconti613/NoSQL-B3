use axum::{Json, extract::State};
use serde::Deserialize;

use crate::error::AppResult;
use crate::importer;
use crate::models::Artist;

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct ImportArtistRequest {
    pub mbid: String,
}

/// `POST /api/import/artists` — imports (or refreshes) one artist from
/// MusicBrainz into Neo4j, along with a page of recordings, releases,
/// labels, genres and detected collaborations. Idempotent: re-importing the
/// same MBID updates the existing node instead of creating a duplicate.
pub async fn import_artist(
    State(state): State<AppState>,
    Json(req): Json<ImportArtistRequest>,
) -> AppResult<Json<Artist>> {
    let artist = importer::import_artist(&state.repo, &state.mb, &req.mbid).await?;
    Ok(Json(artist))
}
