use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::error::{AppError, AppResult};
use crate::models::{Artist, Recording, Release};

use super::{AppState, Pagination};

pub async fn list_releases(
    State(state): State<AppState>,
    Query(p): Query<Pagination>,
) -> AppResult<Json<Vec<Release>>> {
    Ok(Json(state.repo.list_releases(p.limit_or(50), p.offset_or(0)).await?))
}

pub async fn get_release(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<Release>> {
    let release = state
        .repo
        .get_release(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("release '{id}' not found")))?;
    Ok(Json(release))
}

pub async fn get_release_recordings(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<Recording>>> {
    Ok(Json(state.repo.get_release_recordings(&id).await?))
}

pub async fn get_release_artists(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<Artist>>> {
    Ok(Json(state.repo.get_release_artists(&id).await?))
}
