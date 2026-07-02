use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::models::ArtistSearchResult;

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    /// Accepts both `?q=` and `?query=` for convenience.
    q: Option<String>,
    query: Option<String>,
    limit: Option<u32>,
}

/// `GET /api/search/artists?q=...` — live search against the MusicBrainz
/// catalog (not the local Neo4j graph). Used by the frontend's Search page
/// to find artists to import.
pub async fn search_artists(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> AppResult<Json<Vec<ArtistSearchResult>>> {
    let term = params
        .q
        .or(params.query)
        .ok_or_else(|| AppError::BadRequest("missing query parameter 'q'".to_string()))?;
    if term.trim().is_empty() {
        return Err(AppError::BadRequest("query parameter 'q' must not be empty".to_string()));
    }

    let limit = params.limit.unwrap_or(15).clamp(1, 50);
    let response = state.mb.search_artists(&term, limit).await?;

    let results = response
        .artists
        .into_iter()
        .map(|a| ArtistSearchResult {
            mbid: a.id,
            name: a.name,
            country: a.country,
            artist_type: a.artist_type,
            begin_date: a.life_span.and_then(|l| l.begin),
            score: a.score,
            disambiguation: a.disambiguation,
        })
        .collect();

    Ok(Json(results))
}
