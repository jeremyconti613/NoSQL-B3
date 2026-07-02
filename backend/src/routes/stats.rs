use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::error::AppResult;
use crate::models::{ArtistStat, CollaborationStat, GenreStat, OverviewStats};

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct TopQuery {
    limit: Option<i64>,
}

/// `GET /api/stats/overview` — global node/relationship counts.
pub async fn overview(State(state): State<AppState>) -> AppResult<Json<OverviewStats>> {
    Ok(Json(state.repo.stats_overview().await?))
}

/// `GET /api/stats/top-collaborations` — artist pairs with the most shared
/// recordings.
pub async fn top_collaborations(
    State(state): State<AppState>,
    Query(q): Query<TopQuery>,
) -> AppResult<Json<Vec<CollaborationStat>>> {
    let limit = q.limit.unwrap_or(10).clamp(1, 100);
    Ok(Json(state.repo.stats_top_collaborations(limit).await?))
}

/// `GET /api/stats/top-artists` — the most-connected artists (by number of
/// distinct collaborators).
pub async fn top_artists(
    State(state): State<AppState>,
    Query(q): Query<TopQuery>,
) -> AppResult<Json<Vec<ArtistStat>>> {
    let limit = q.limit.unwrap_or(10).clamp(1, 100);
    Ok(Json(state.repo.stats_top_artists(limit).await?))
}

/// `GET /api/stats/top-genres` — most common genres across imported artists.
pub async fn top_genres(
    State(state): State<AppState>,
    Query(q): Query<TopQuery>,
) -> AppResult<Json<Vec<GenreStat>>> {
    let limit = q.limit.unwrap_or(10).clamp(1, 100);
    Ok(Json(state.repo.stats_top_genres(limit).await?))
}
