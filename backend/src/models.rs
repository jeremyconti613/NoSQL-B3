//! Domain models mirroring the Neo4j node shapes described in the project spec.
//! These are the types returned by our own `/api/*` endpoints (as opposed to
//! `musicbrainz::models`, which mirrors MusicBrainz's own JSON shapes).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    pub mbid: String,
    pub name: String,
    #[serde(rename = "type")]
    pub artist_type: Option<String>,
    pub country: Option<String>,
    pub gender: Option<String>,
    #[serde(rename = "beginDate")]
    pub begin_date: Option<String>,
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
    pub disambiguation: Option<String>,
}

/// Result of a live search against MusicBrainz's own catalog (as opposed to
/// artists already imported into our Neo4j graph) — the fields required by
/// the "Recherche d'artistes" spec section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistSearchResult {
    pub mbid: String,
    pub name: String,
    pub country: Option<String>,
    #[serde(rename = "type")]
    pub artist_type: Option<String>,
    #[serde(rename = "beginDate")]
    pub begin_date: Option<String>,
    pub score: Option<i64>,
    pub disambiguation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    pub mbid: String,
    pub title: String,
    pub length: Option<i64>,
    #[serde(rename = "firstReleaseDate")]
    pub first_release_date: Option<String>,
    /// Internal popularity heuristic (see `importer::compute_popularity`).
    pub popularity: Option<f64>,
    /// Where this record was sourced from, e.g. "musicbrainz".
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub mbid: String,
    pub title: String,
    pub date: Option<String>,
    pub country: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "releaseType")]
    pub release_type: Option<String>,
    #[serde(rename = "coverArtUrl")]
    pub cover_art_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub mbid: String,
    pub name: String,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genre {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Area {
    pub mbid: String,
    pub name: String,
    #[serde(rename = "type")]
    pub area_type: Option<String>,
}

/// A collaboration edge between two artists, aggregated by shared recordings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collaboration {
    pub artist: Artist,
    pub weight: i64,
    pub shared_recordings: Vec<String>,
}

/// Node/link shape consumed directly by `react-force-graph` on the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLink {
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub link_type: String,
    pub weight: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub links: Vec<GraphLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewStats {
    pub artist_count: i64,
    pub recording_count: i64,
    pub release_count: i64,
    pub label_count: i64,
    pub genre_count: i64,
    pub area_count: i64,
    pub collaboration_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistStat {
    pub artist: Artist,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationStat {
    pub artist_a: Artist,
    pub artist_b: Artist,
    pub weight: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenreStat {
    pub genre: String,
    pub count: i64,
}
