use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;

use crate::error::AppResult;
use crate::models::GraphData;

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct GraphQuery {
    limit: Option<i64>,
}

/// `GET /api/graph` — a bounded snapshot of the whole graph (artists,
/// recordings, releases and their relations), shaped for `react-force-graph`.
pub async fn graph_full(State(state): State<AppState>, Query(q): Query<GraphQuery>) -> AppResult<Json<GraphData>> {
    let limit = q.limit.unwrap_or(60).clamp(1, 500);
    Ok(Json(state.repo.graph_full(limit).await?))
}

/// `GET /api/graph/artists/:id` — neighborhood graph centered on one artist.
pub async fn graph_for_artist(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<GraphData>> {
    Ok(Json(state.repo.graph_for_artist(&id).await?))
}

/// `GET /api/graph/collaborations` — the collaboration network only.
pub async fn graph_collaborations(
    State(state): State<AppState>,
    Query(q): Query<GraphQuery>,
) -> AppResult<Json<GraphData>> {
    let limit = q.limit.unwrap_or(200).clamp(1, 2000);
    Ok(Json(state.repo.graph_collaborations(limit).await?))
}
