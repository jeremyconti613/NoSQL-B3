//! Serde mirrors of the subset of MusicBrainz JSON (`fmt=json`) responses we
//! consume. Every field beyond `id`/`name` is optional: MusicBrainz data is
//! famously incomplete (missing countries, dates, genres, ...), and the
//! project's data-quality requirement is to degrade gracefully rather than
//! fail the whole import when a field is absent.

use serde::{Deserialize, Serialize};

fn de_flexible_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Flexible {
        Int(i64),
        Str(String),
        Null,
    }
    match Option::<Flexible>::deserialize(deserializer)? {
        Some(Flexible::Int(i)) => Ok(Some(i)),
        Some(Flexible::Str(s)) => Ok(s.parse().ok()),
        Some(Flexible::Null) | None => Ok(None),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbLifeSpan {
    pub begin: Option<String>,
    pub end: Option<String>,
    #[serde(default)]
    pub ended: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbArea {
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub area_type: Option<String>,
    #[serde(rename = "iso-3166-1-codes", default)]
    pub iso_3166_1_codes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbGenre {
    pub name: String,
    #[serde(default)]
    pub count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbArtistRef {
    pub id: String,
    pub name: String,
}

/// One entry in `artist-credit`: the credited artist plus the literal
/// join-phrase MusicBrainz uses to render the full credit string (e.g.
/// `" feat. "`, `" & "`, `" x "`) — a strong signal for collaboration/feature
/// detection alongside textual markers in the title itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbArtistCredit {
    pub name: String,
    #[serde(default)]
    pub joinphrase: Option<String>,
    pub artist: MbArtistRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbRelationTarget {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

/// A `relations` entry, present when the artist lookup includes `artist-rels`
/// (e.g. explicit "collaborator" / "member of band" relationships).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbRelation {
    #[serde(rename = "type")]
    pub relation_type: String,
    #[serde(rename = "target-type", default)]
    pub target_type: Option<String>,
    #[serde(default)]
    pub artist: Option<MbRelationTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbArtist {
    pub id: String,
    pub name: String,
    #[serde(rename = "type", default)]
    pub artist_type: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub gender: Option<String>,
    #[serde(default)]
    pub disambiguation: Option<String>,
    #[serde(rename = "life-span", default)]
    pub life_span: Option<MbLifeSpan>,
    #[serde(default)]
    pub area: Option<MbArea>,
    #[serde(default)]
    pub genres: Option<Vec<MbGenre>>,
    #[serde(default)]
    pub relations: Option<Vec<MbRelation>>,
    #[serde(default, deserialize_with = "de_flexible_i64")]
    pub score: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbArtistSearchResponse {
    #[serde(default)]
    pub count: i64,
    #[serde(default)]
    pub artists: Vec<MbArtist>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbReleaseGroup {
    #[serde(rename = "primary-type", default)]
    pub primary_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbLabelRef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub area: Option<MbArea>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbLabelInfo {
    #[serde(default)]
    pub label: Option<MbLabelRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbRelease {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(rename = "release-group", default)]
    pub release_group: Option<MbReleaseGroup>,
    #[serde(rename = "label-info", default)]
    pub label_info: Option<Vec<MbLabelInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbRecording {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub length: Option<i64>,
    #[serde(rename = "first-release-date", default)]
    pub first_release_date: Option<String>,
    #[serde(rename = "artist-credit", default)]
    pub artist_credit: Option<Vec<MbArtistCredit>>,
    #[serde(default)]
    pub releases: Option<Vec<MbRelease>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbRecordingSearchResponse {
    #[serde(default)]
    pub count: i64,
    #[serde(default)]
    pub recordings: Vec<MbRecording>,
}

/// Cover Art Archive response for `GET /release/{mbid}` (bonus cover images).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverArtResponse {
    #[serde(default)]
    pub images: Vec<CoverArtImage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverArtImage {
    #[serde(default)]
    pub front: bool,
    #[serde(default)]
    pub image: Option<String>,
}
