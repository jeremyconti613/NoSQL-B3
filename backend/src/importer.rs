//! MusicBrainz -> Neo4j import pipeline, and the heuristics used to detect
//! collaborations/featurings between artists.
//!
//! Collaboration detection combines three of the signals called out in the
//! spec:
//!   1. Multiple artist-credits on the same recording (the strongest and
//!      most reliable signal — MusicBrainz has already resolved the credited
//!      artists to MBIDs for us).
//!   2. Textual markers (`feat.`, `ft.`, `featuring`, `avec`, ` x `, `&`) in
//!      the credit join-phrases or the recording title, used to double-check
//!      / log the classification (MusicBrainz's own credit split already
//!      captures this in practice, so this is mostly a defensive check for
//!      messy/legacy data).
//!   3. Explicit MusicBrainz artist-relations (`inc=artist-rels`) when the
//!      artist lookup returns any (e.g. "collaboration", "member of band").
//!
//! Release -> Area linking is intentionally skipped: MusicBrainz releases
//! only expose a raw ISO country code, not an Area MBID, and resolving one
//! would require an extra rate-limited API call per unique country. Given
//! the "limit MusicBrainz calls" data-quality requirement, we store the
//! country code directly on `Release.country` instead (see docs/data-model.md).

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::models::{Area, Artist, Label, Recording, Release};
use crate::musicbrainz::MusicBrainzClient;
use crate::musicbrainz::models::{MbArtist, MbLabelRef, MbRecording, MbRelease};
use crate::repo::Repo;

/// Everything fetched from MusicBrainz for one artist: raw enough to be
/// cached to disk (see `data/seed.json`) and replayed through
/// [`import_bundle`] without hitting the network again.
///
/// `recordings` holds fully-detailed recordings (each already enriched with
/// its releases via a per-recording lookup — see [`fetch_artist_bundle`]),
/// not the lightweight browse-list MusicBrainz returns by default.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistBundle {
    pub artist: MbArtist,
    pub recordings: Vec<MbRecording>,
}

/// How many recordings to pull per imported artist. MusicBrainz's own page
/// size cap is 100; we keep this modest to bound import time given the
/// ~1 req/s rate limit and that each recording needs its own follow-up
/// lookup (browsing recordings by artist doesn't expose their releases).
const RECORDINGS_PER_ARTIST: u32 = 20;

/// Release enrichment (label info via a `/release/{id}?inc=labels` lookup,
/// plus the bonus Cover Art Archive image) is capped per unique release to
/// keep overall import time bounded — MusicBrainz's own lookup is
/// rate-limited, so this is the dominant cost of an import beyond the
/// recording lookups themselves.
const MAX_RELEASE_ENRICHMENT_LOOKUPS: usize = 10;

const FEATURE_MARKERS: [&str; 6] = ["feat.", "featuring", "ft.", "avec", " x ", " & "];

fn contains_feature_marker(text: &str) -> bool {
    let lower = text.to_lowercase();
    FEATURE_MARKERS.iter().any(|m| lower.contains(m))
}

fn artist_from_mb(mb: &MbArtist) -> Artist {
    Artist {
        mbid: mb.id.clone(),
        name: mb.name.clone(),
        artist_type: mb.artist_type.clone(),
        country: mb.country.clone(),
        gender: mb.gender.clone(),
        begin_date: mb.life_span.as_ref().and_then(|l| l.begin.clone()),
        end_date: mb.life_span.as_ref().and_then(|l| l.end.clone()),
        disambiguation: mb.disambiguation.clone(),
    }
}

/// Builds a minimal artist record (mbid + name only) for artists we only
/// know about through someone else's credits/relations, not through a full
/// lookup. Re-importing this artist later will fill in the rest without
/// ever creating a duplicate node, since every write is `MERGE`d on `mbid`.
fn minimal_artist(mbid: &str, name: &str) -> Artist {
    Artist {
        mbid: mbid.to_string(),
        name: name.to_string(),
        artist_type: None,
        country: None,
        gender: None,
        begin_date: None,
        end_date: None,
        disambiguation: None,
    }
}

fn recording_from_mb(mb: &MbRecording, popularity: f64) -> Recording {
    Recording {
        mbid: mb.id.clone(),
        title: mb.title.clone(),
        length: mb.length,
        first_release_date: mb.first_release_date.clone(),
        popularity: Some(popularity),
        source: Some("musicbrainz".to_string()),
    }
}

fn release_from_mb(mb: &MbRelease, cover_art_url: Option<String>) -> Release {
    Release {
        mbid: mb.id.clone(),
        title: mb.title.clone(),
        date: mb.date.clone(),
        country: mb.country.clone(),
        status: mb.status.clone(),
        release_type: mb.release_group.as_ref().and_then(|g| g.primary_type.clone()),
        cover_art_url,
    }
}

fn label_from_mb(mb: &MbLabelRef) -> Label {
    let country = mb.area.as_ref().and_then(|a| {
        a.iso_3166_1_codes
            .as_ref()
            .and_then(|codes| codes.first().cloned())
            .or_else(|| a.name.clone())
    });
    Label {
        mbid: mb.id.clone(),
        name: mb.name.clone(),
        country,
    }
}

/// Popularity is an internal heuristic (the spec asks for "un score de
/// popularité interne si calculé", not a MusicBrainz field): the number of
/// releases a recording appears on plus the number of credited artists,
/// as a rough proxy for how widely-circulated/collaborative a track is.
fn compute_popularity(mb: &MbRecording) -> f64 {
    let release_count = mb.releases.as_ref().map(|r| r.len()).unwrap_or(0) as f64;
    let credit_count = mb.artist_credit.as_ref().map(|c| c.len()).unwrap_or(1) as f64;
    release_count + credit_count
}

/// Fetches everything needed to import one artist from MusicBrainz, without
/// writing anything to Neo4j. Split out from [`import_bundle`] so the seed
/// binary can cache the result to `data/seed.json` and replay it offline.
///
/// MusicBrainz's "browse recordings by artist" endpoint only returns a
/// lightweight summary per recording (no releases), so each one is enriched
/// with a follow-up direct lookup that does include releases.
pub async fn fetch_artist_bundle(mb: &MusicBrainzClient, mbid: &str) -> AppResult<ArtistBundle> {
    let artist = mb.get_artist(mbid).await?;
    let summary = mb.get_artist_recordings(mbid, RECORDINGS_PER_ARTIST).await?;

    let mut recordings = Vec::with_capacity(summary.recordings.len());
    for r in &summary.recordings {
        match mb.get_recording(&r.id).await {
            Ok(detail) => recordings.push(detail),
            Err(e) => {
                tracing::warn!(recording = %r.id, error = %e, "failed to fetch recording detail, skipping");
            }
        }
    }

    Ok(ArtistBundle { artist, recordings })
}

/// Imports (or refreshes) one artist by MusicBrainz ID: fetches live, then
/// writes to Neo4j. Safe to call repeatedly — every write is keyed by MBID
/// via `MERGE`. For offline/cached imports, see [`import_bundle`].
pub async fn import_artist(repo: &Repo, mb: &MusicBrainzClient, mbid: &str) -> AppResult<Artist> {
    let bundle = fetch_artist_bundle(mb, mbid).await?;
    import_bundle(repo, mb, &bundle).await
}

/// Writes an already-fetched [`ArtistBundle`] into Neo4j: the artist itself,
/// its genres/area, its recordings, their releases/labels, and all detected
/// collaborations. Safe to call repeatedly — every write is keyed by MBID
/// via `MERGE`. `mb` is only used for the bonus cover-art lookup, which is
/// itself best-effort and never fails the import if unreachable.
pub async fn import_bundle(repo: &Repo, mb: &MusicBrainzClient, bundle: &ArtistBundle) -> AppResult<Artist> {
    let mb_artist = &bundle.artist;
    let artist = artist_from_mb(mb_artist);
    repo.upsert_artist(&artist).await?;

    if let Some(genres) = &mb_artist.genres {
        for genre in genres {
            repo.upsert_genre(&genre.name).await?;
            repo.link_associated_with_genre(&artist.mbid, &genre.name).await?;
        }
    }

    if let Some(area) = &mb_artist.area
        && let Some(area_id) = &area.id {
            let area_domain = Area {
                mbid: area_id.clone(),
                name: area.name.clone().unwrap_or_default(),
                area_type: area.area_type.clone(),
            };
            repo.upsert_area(&area_domain).await?;
            repo.link_from_area(&artist.mbid, area_id).await?;
        }

    // Signal 3: explicit MusicBrainz artist-relations.
    if let Some(relations) = &mb_artist.relations {
        for relation in relations {
            let is_collaboration_like = relation.relation_type.to_lowercase().contains("collab")
                || relation.relation_type.to_lowercase().contains("member of band");
            if !is_collaboration_like {
                continue;
            }
            if let Some(target) = &relation.artist
                && let (Some(target_id), Some(target_name)) = (&target.id, &target.name) {
                    repo.upsert_artist(&minimal_artist(target_id, target_name)).await?;
                    // No specific shared recording for a relation-derived edge;
                    // use the relation type itself as the "shared recording"
                    // marker so weight increments stay meaningful and idempotent.
                    repo.link_collaborated(&artist.mbid, target_id, &format!("rel:{}", relation.relation_type))
                        .await?;
                }
        }
    }

    let mut enrichment_lookups = 0usize;
    let mut seen_releases: HashSet<String> = HashSet::new();

    for mb_recording in &bundle.recordings {
        let popularity = compute_popularity(mb_recording);
        let recording = recording_from_mb(mb_recording, popularity);
        repo.upsert_recording(&recording).await?;

        let credits = mb_recording.artist_credit.clone().unwrap_or_default();

        // Signal 2 (defensive/logging only): flag recordings whose title or
        // join-phrases carry a feature marker, in case MusicBrainz's own
        // credit split ever under-represents a collaboration.
        let has_marker = contains_feature_marker(&mb_recording.title)
            || credits.iter().any(|c| c.joinphrase.as_deref().is_some_and(contains_feature_marker));
        if has_marker {
            tracing::debug!(recording = %recording.title, "feature marker detected in title/credits");
        }

        // Signal 1 & 4: every credited artist is upserted; the first credit
        // is treated as the primary performer, any additional credits as
        // featured artists. Every distinct pair gets a COLLABORATED_WITH edge.
        let mut credited_ids = Vec::new();
        for (i, credit) in credits.iter().enumerate() {
            repo.upsert_artist(&minimal_artist(&credit.artist.id, &credit.artist.name)).await?;
            if i == 0 {
                repo.link_performed(&credit.artist.id, &recording.mbid).await?;
            } else {
                repo.link_featured_on(&credit.artist.id, &recording.mbid).await?;
            }
            credited_ids.push(credit.artist.id.clone());
        }
        // If MusicBrainz returned no credits at all, fall back to crediting
        // the artist we're importing directly.
        if credited_ids.is_empty() {
            repo.link_performed(&artist.mbid, &recording.mbid).await?;
            credited_ids.push(artist.mbid.clone());
        }

        for i in 0..credited_ids.len() {
            for j in (i + 1)..credited_ids.len() {
                repo.link_collaborated(&credited_ids[i], &credited_ids[j], &recording.mbid)
                    .await?;
            }
        }

        // Releases this recording appears on. Each unique release is
        // (bounded) enriched with a `/release/{id}?inc=labels` lookup for
        // its label-info and a Cover Art Archive lookup for its cover image
        // — neither is available from the recording-level lookup above.
        for mb_release in mb_recording.releases.clone().unwrap_or_default() {
            let is_new = seen_releases.insert(mb_release.id.clone());
            let mut label_info = mb_release.label_info.clone().unwrap_or_default();
            let mut cover_art_url = None;

            if is_new && enrichment_lookups < MAX_RELEASE_ENRICHMENT_LOOKUPS {
                enrichment_lookups += 1;
                if let Ok(detail) = mb.get_release(&mb_release.id).await {
                    label_info = detail.label_info.unwrap_or_default();
                }
                cover_art_url = mb.get_cover_art_url(&mb_release.id).await;
            }

            let release = release_from_mb(&mb_release, cover_art_url);
            repo.upsert_release(&release).await?;
            repo.link_appears_on(&recording.mbid, &release.mbid).await?;

            for info in label_info {
                if let Some(mb_label) = info.label {
                    let label = label_from_mb(&mb_label);
                    repo.upsert_label(&label).await?;
                    repo.link_released_by(&release.mbid, &label.mbid).await?;
                }
            }
        }
    }

    Ok(artist)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::musicbrainz::models::{MbArea, MbArtistCredit, MbArtistRef, MbLabelRef};

    fn credit(id: &str, name: &str) -> MbArtistCredit {
        MbArtistCredit {
            name: name.to_string(),
            joinphrase: None,
            artist: MbArtistRef { id: id.to_string(), name: name.to_string() },
        }
    }

    #[test]
    fn detects_feature_markers_case_insensitively() {
        assert!(contains_feature_marker("Get Lucky (feat. Pharrell Williams)"));
        assert!(contains_feature_marker("Some Track FEATURING Someone"));
        assert!(contains_feature_marker("Track ft. Someone"));
        assert!(contains_feature_marker("Track avec Someone"));
        assert!(contains_feature_marker("Artist A x Artist B"));
        assert!(contains_feature_marker("Artist A & Artist B"));
        assert!(!contains_feature_marker("Around the World"));
    }

    #[test]
    fn popularity_combines_releases_and_credits() {
        let mb = MbRecording {
            id: "r1".into(),
            title: "Test".into(),
            length: Some(200_000),
            first_release_date: None,
            artist_credit: Some(vec![credit("a1", "Artist One"), credit("a2", "Artist Two")]),
            releases: Some(vec![
                MbRelease {
                    id: "rel1".into(),
                    title: "Album".into(),
                    date: None,
                    country: None,
                    status: None,
                    release_group: None,
                    label_info: None,
                },
                MbRelease {
                    id: "rel2".into(),
                    title: "Compilation".into(),
                    date: None,
                    country: None,
                    status: None,
                    release_group: None,
                    label_info: None,
                },
            ]),
        };
        // 2 releases + 2 credited artists.
        assert_eq!(compute_popularity(&mb), 4.0);
    }

    #[test]
    fn popularity_defaults_to_single_credit_when_missing() {
        let mb = MbRecording {
            id: "r1".into(),
            title: "Test".into(),
            length: None,
            first_release_date: None,
            artist_credit: None,
            releases: None,
        };
        // 0 releases + 1 (fallback: assume a single, unresolved performer).
        assert_eq!(compute_popularity(&mb), 1.0);
    }

    #[test]
    fn label_country_prefers_iso_code_over_area_name() {
        let mb = MbLabelRef {
            id: "l1".into(),
            name: "Columbia".into(),
            area: Some(MbArea {
                id: Some("a1".into()),
                name: Some("United States".into()),
                area_type: Some("Country".into()),
                iso_3166_1_codes: Some(vec!["US".into()]),
            }),
        };
        let label = label_from_mb(&mb);
        assert_eq!(label.country.as_deref(), Some("US"));
    }

    #[test]
    fn label_country_falls_back_to_area_name_without_iso_code() {
        let mb = MbLabelRef {
            id: "l1".into(),
            name: "Some Label".into(),
            area: Some(MbArea {
                id: Some("a1".into()),
                name: Some("Worldwide".into()),
                area_type: None,
                iso_3166_1_codes: None,
            }),
        };
        let label = label_from_mb(&mb);
        assert_eq!(label.country.as_deref(), Some("Worldwide"));
    }

    #[test]
    fn label_country_is_none_without_area() {
        let mb = MbLabelRef { id: "l1".into(), name: "Some Label".into(), area: None };
        assert_eq!(label_from_mb(&mb).country, None);
    }
}
