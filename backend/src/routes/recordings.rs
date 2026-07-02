use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::error::{AppError, AppResult};
use crate::models::{Artist, Recording, Release};

use super::{AppState, Pagination};

pub async fn list_recordings(
    State(state): State<AppState>,
    Query(p): Query<Pagination>,
) -> AppResult<Json<Vec<Recording>>> {
    Ok(Json(state.repo.list_recordings(p.limit_or(50), p.offset_or(0)).await?))
}

pub async fn get_recording(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<Recording>> {
    let recording = state
        .repo
        .get_recording(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("recording '{id}' not found")))?;
    Ok(Json(recording))
}

pub async fn get_recording_artists(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<Artist>>> {
    Ok(Json(state.repo.get_recording_artists(&id).await?))
}

pub async fn get_recording_releases(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<Release>>> {
    Ok(Json(state.repo.get_recording_releases(&id).await?))
}
