use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::error::{AppError, AppResult};
use crate::models::{Artist, Collaboration, Recording, Release};

use super::{AppState, Pagination};

pub async fn list_artists(
    State(state): State<AppState>,
    Query(p): Query<Pagination>,
) -> AppResult<Json<Vec<Artist>>> {
    let artists = state.repo.list_artists(p.limit_or(50), p.offset_or(0)).await?;
    Ok(Json(artists))
}

pub async fn get_artist(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<Artist>> {
    let artist = state
        .repo
        .get_artist(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("artist '{id}' not found")))?;
    Ok(Json(artist))
}

pub async fn get_artist_recordings(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<Recording>>> {
    Ok(Json(state.repo.get_artist_recordings(&id).await?))
}

pub async fn get_artist_releases(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<Release>>> {
    Ok(Json(state.repo.get_artist_releases(&id).await?))
}

pub async fn get_artist_collaborations(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<Collaboration>>> {
    Ok(Json(state.repo.get_artist_collaborations(&id).await?))
}
