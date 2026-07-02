//! Cypher access layer: every read/write against Neo4j goes through this
//! module. Writes use `MERGE ... SET n += $props` (rather than a full
//! `ON CREATE/ON MATCH SET`) so that re-importing an artist with partially
//! missing MusicBrainz data never clobbers previously known good fields —
//! only keys we actually have a value for get sent in `$props`.

use std::collections::{HashMap, HashSet};

use neo4rs::{BoltType, Graph, Row, query};

use crate::error::{AppError, AppResult};
use crate::models::{
    Area, Artist, ArtistStat, Collaboration, CollaborationStat, GenreStat, GraphData, GraphLink,
    GraphNode, Label, OverviewStats, Recording, Release,
};

#[derive(Clone)]
pub struct Repo {
    graph: Graph,
}

// ---------------------------------------------------------------------------
// Row -> domain struct helpers
// ---------------------------------------------------------------------------

/// Reads an optional column: absent column, null value, or a type mismatch
/// all collapse to `None` rather than failing the whole row — MusicBrainz
/// data is frequently incomplete and we must degrade gracefully.
fn opt<T: serde::de::DeserializeOwned>(row: &Row, key: &str) -> Option<T> {
    row.get::<Option<T>>(key).ok().flatten()
}

/// Reads a required column, turning a missing/invalid value into a clear
/// internal error instead of a panic.
fn req<T: serde::de::DeserializeOwned>(row: &Row, key: &str) -> AppResult<T> {
    row.get::<T>(key)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("missing/invalid column '{key}': {e}")))
}

fn artist_from_row(row: &Row) -> AppResult<Artist> {
    Ok(Artist {
        mbid: req(row, "mbid")?,
        name: req(row, "name")?,
        artist_type: opt(row, "type"),
        country: opt(row, "country"),
        gender: opt(row, "gender"),
        begin_date: opt(row, "beginDate"),
        end_date: opt(row, "endDate"),
        disambiguation: opt(row, "disambiguation"),
    })
}

fn artist_from_row_prefixed(row: &Row, prefix: &str) -> AppResult<Artist> {
    Ok(Artist {
        mbid: req(row, &format!("{prefix}Mbid"))?,
        name: req(row, &format!("{prefix}Name"))?,
        artist_type: opt(row, &format!("{prefix}Type")),
        country: opt(row, &format!("{prefix}Country")),
        gender: opt(row, &format!("{prefix}Gender")),
        begin_date: opt(row, &format!("{prefix}BeginDate")),
        end_date: opt(row, &format!("{prefix}EndDate")),
        disambiguation: opt(row, &format!("{prefix}Disambiguation")),
    })
}

fn recording_from_row(row: &Row) -> AppResult<Recording> {
    Ok(Recording {
        mbid: req(row, "mbid")?,
        title: req(row, "title")?,
        length: opt(row, "length"),
        first_release_date: opt(row, "firstReleaseDate"),
        popularity: opt(row, "popularity"),
        source: opt(row, "source"),
    })
}

fn release_from_row(row: &Row) -> AppResult<Release> {
    Ok(Release {
        mbid: req(row, "mbid")?,
        title: req(row, "title")?,
        date: opt(row, "date"),
        country: opt(row, "country"),
        status: opt(row, "status"),
        release_type: opt(row, "releaseType"),
        cover_art_url: opt(row, "coverArtUrl"),
    })
}

fn artist_node(a: &Artist) -> GraphNode {
    GraphNode {
        id: a.mbid.clone(),
        label: a.name.clone(),
        node_type: "Artist".to_string(),
    }
}

fn recording_node(r: &Recording) -> GraphNode {
    GraphNode {
        id: r.mbid.clone(),
        label: r.title.clone(),
        node_type: "Recording".to_string(),
    }
}

fn release_node(r: &Release) -> GraphNode {
    GraphNode {
        id: r.mbid.clone(),
        label: r.title.clone(),
        node_type: "Release".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Props builders (only `Some` fields are sent, see module doc)
// ---------------------------------------------------------------------------

fn artist_props(a: &Artist) -> HashMap<String, BoltType> {
    let mut m = HashMap::new();
    m.insert("name".into(), a.name.clone().into());
    if let Some(v) = &a.artist_type {
        m.insert("type".into(), v.clone().into());
    }
    if let Some(v) = &a.country {
        m.insert("country".into(), v.clone().into());
    }
    if let Some(v) = &a.gender {
        m.insert("gender".into(), v.clone().into());
    }
    if let Some(v) = &a.begin_date {
        m.insert("beginDate".into(), v.clone().into());
    }
    if let Some(v) = &a.end_date {
        m.insert("endDate".into(), v.clone().into());
    }
    if let Some(v) = &a.disambiguation {
        m.insert("disambiguation".into(), v.clone().into());
    }
    m
}

fn recording_props(r: &Recording) -> HashMap<String, BoltType> {
    let mut m = HashMap::new();
    m.insert("title".into(), r.title.clone().into());
    if let Some(v) = r.length {
        m.insert("length".into(), v.into());
    }
    if let Some(v) = &r.first_release_date {
        m.insert("firstReleaseDate".into(), v.clone().into());
    }
    if let Some(v) = r.popularity {
        m.insert("popularity".into(), v.into());
    }
    if let Some(v) = &r.source {
        m.insert("source".into(), v.clone().into());
    }
    m
}

fn release_props(r: &Release) -> HashMap<String, BoltType> {
    let mut m = HashMap::new();
    m.insert("title".into(), r.title.clone().into());
    if let Some(v) = &r.date {
        m.insert("date".into(), v.clone().into());
    }
    if let Some(v) = &r.country {
        m.insert("country".into(), v.clone().into());
    }
    if let Some(v) = &r.status {
        m.insert("status".into(), v.clone().into());
    }
    if let Some(v) = &r.release_type {
        m.insert("releaseType".into(), v.clone().into());
    }
    if let Some(v) = &r.cover_art_url {
        m.insert("coverArtUrl".into(), v.clone().into());
    }
    m
}

const ARTIST_FIELDS: &str = "a.mbid AS mbid, a.name AS name, a.type AS type, a.country AS country, \
     a.gender AS gender, a.beginDate AS beginDate, a.endDate AS endDate, a.disambiguation AS disambiguation";

const RECORDING_FIELDS: &str = "r.mbid AS mbid, r.title AS title, r.length AS length, \
     r.firstReleaseDate AS firstReleaseDate, r.popularity AS popularity, r.source AS source";

const RELEASE_FIELDS: &str = "rel.mbid AS mbid, rel.title AS title, rel.date AS date, rel.country AS country, \
     rel.status AS status, rel.releaseType AS releaseType, rel.coverArtUrl AS coverArtUrl";

impl Repo {
    pub fn new(graph: Graph) -> Self {
        Self { graph }
    }

    // -- Writes (used by the importer) --------------------------------------

    pub async fn upsert_artist(&self, a: &Artist) -> AppResult<()> {
        let q = query("MERGE (a:Artist {mbid: $mbid}) SET a += $props")
            .param("mbid", a.mbid.clone())
            .param("props", artist_props(a));
        self.graph.run(q).await?;
        Ok(())
    }

    pub async fn upsert_recording(&self, r: &Recording) -> AppResult<()> {
        let q = query("MERGE (r:Recording {mbid: $mbid}) SET r += $props")
            .param("mbid", r.mbid.clone())
            .param("props", recording_props(r));
        self.graph.run(q).await?;
        Ok(())
    }

    pub async fn upsert_release(&self, r: &Release) -> AppResult<()> {
        let q = query("MERGE (r:Release {mbid: $mbid}) SET r += $props")
            .param("mbid", r.mbid.clone())
            .param("props", release_props(r));
        self.graph.run(q).await?;
        Ok(())
    }

    pub async fn upsert_label(&self, l: &Label) -> AppResult<()> {
        let mut props = HashMap::new();
        props.insert("name".to_string(), BoltType::from(l.name.clone()));
        if let Some(c) = &l.country {
            props.insert("country".to_string(), BoltType::from(c.clone()));
        }
        let q = query("MERGE (l:Label {mbid: $mbid}) SET l += $props")
            .param("mbid", l.mbid.clone())
            .param("props", props);
        self.graph.run(q).await?;
        Ok(())
    }

    pub async fn upsert_genre(&self, name: &str) -> AppResult<()> {
        let q = query("MERGE (:Genre {name: $name})").param("name", name);
        self.graph.run(q).await?;
        Ok(())
    }

    pub async fn upsert_area(&self, a: &Area) -> AppResult<()> {
        let mut props = HashMap::new();
        props.insert("name".to_string(), BoltType::from(a.name.clone()));
        if let Some(t) = &a.area_type {
            props.insert("type".to_string(), BoltType::from(t.clone()));
        }
        let q = query("MERGE (ar:Area {mbid: $mbid}) SET ar += $props")
            .param("mbid", a.mbid.clone())
            .param("props", props);
        self.graph.run(q).await?;
        Ok(())
    }

    pub async fn link_performed(&self, artist_mbid: &str, recording_mbid: &str) -> AppResult<()> {
        let q = query(
            "MATCH (a:Artist {mbid: $artist}), (r:Recording {mbid: $recording}) \
             MERGE (a)-[:PERFORMED]->(r)",
        )
        .param("artist", artist_mbid)
        .param("recording", recording_mbid);
        self.graph.run(q).await?;
        Ok(())
    }

    pub async fn link_featured_on(&self, artist_mbid: &str, recording_mbid: &str) -> AppResult<()> {
        let q = query(
            "MATCH (a:Artist {mbid: $artist}), (r:Recording {mbid: $recording}) \
             MERGE (a)-[:FEATURED_ON]->(r)",
        )
        .param("artist", artist_mbid)
        .param("recording", recording_mbid);
        self.graph.run(q).await?;
        Ok(())
    }

    /// Records a collaboration between two artists on a given recording.
    /// The relationship is stored once per unordered pair, in a canonical
    /// (lexicographically sorted) direction, so repeated imports never
    /// create duplicate/reverse edges. `weight` counts distinct shared
    /// recordings; re-processing the same recording is a no-op.
    pub async fn link_collaborated(
        &self,
        artist_a: &str,
        artist_b: &str,
        recording_mbid: &str,
    ) -> AppResult<()> {
        if artist_a == artist_b {
            return Ok(());
        }
        let (from, to) = if artist_a < artist_b {
            (artist_a, artist_b)
        } else {
            (artist_b, artist_a)
        };
        let q = query(
            "MATCH (a:Artist {mbid: $from}), (b:Artist {mbid: $to}) \
             MERGE (a)-[rel:COLLABORATED_WITH]->(b) \
             ON CREATE SET rel.weight = 1, rel.sharedRecordings = [$recording] \
             ON MATCH SET \
                rel.weight = CASE WHEN $recording IN rel.sharedRecordings THEN rel.weight ELSE rel.weight + 1 END, \
                rel.sharedRecordings = CASE WHEN $recording IN rel.sharedRecordings \
                    THEN rel.sharedRecordings ELSE rel.sharedRecordings + $recording END",
        )
        .param("from", from)
        .param("to", to)
        .param("recording", recording_mbid);
        self.graph.run(q).await?;
        Ok(())
    }

    pub async fn link_appears_on(&self, recording_mbid: &str, release_mbid: &str) -> AppResult<()> {
        let q = query(
            "MATCH (r:Recording {mbid: $recording}), (rel:Release {mbid: $release}) \
             MERGE (r)-[:APPEARS_ON]->(rel)",
        )
        .param("recording", recording_mbid)
        .param("release", release_mbid);
        self.graph.run(q).await?;
        Ok(())
    }

    pub async fn link_released_by(&self, release_mbid: &str, label_mbid: &str) -> AppResult<()> {
        let q = query(
            "MATCH (rel:Release {mbid: $release}), (l:Label {mbid: $label}) \
             MERGE (rel)-[:RELEASED_BY]->(l)",
        )
        .param("release", release_mbid)
        .param("label", label_mbid);
        self.graph.run(q).await?;
        Ok(())
    }

    pub async fn link_associated_with_genre(&self, artist_mbid: &str, genre: &str) -> AppResult<()> {
        let q = query(
            "MATCH (a:Artist {mbid: $artist}), (g:Genre {name: $genre}) \
             MERGE (a)-[:ASSOCIATED_WITH_GENRE]->(g)",
        )
        .param("artist", artist_mbid)
        .param("genre", genre);
        self.graph.run(q).await?;
        Ok(())
    }

    pub async fn link_from_area(&self, artist_mbid: &str, area_mbid: &str) -> AppResult<()> {
        let q = query(
            "MATCH (a:Artist {mbid: $artist}), (ar:Area {mbid: $area}) \
             MERGE (a)-[:FROM_AREA]->(ar)",
        )
        .param("artist", artist_mbid)
        .param("area", area_mbid);
        self.graph.run(q).await?;
        Ok(())
    }

    pub async fn link_released_in(&self, release_mbid: &str, area_mbid: &str) -> AppResult<()> {
        let q = query(
            "MATCH (rel:Release {mbid: $release}), (ar:Area {mbid: $area}) \
             MERGE (rel)-[:RELEASED_IN]->(ar)",
        )
        .param("release", release_mbid)
        .param("area", area_mbid);
        self.graph.run(q).await?;
        Ok(())
    }

    // -- Reads: artists -------------------------------------------------------

    pub async fn list_artists(&self, limit: i64, offset: i64) -> AppResult<Vec<Artist>> {
        let q = query(&format!(
            "MATCH (a:Artist) RETURN {ARTIST_FIELDS} ORDER BY a.name SKIP $offset LIMIT $limit"
        ))
        .param("offset", offset)
        .param("limit", limit);
        let mut stream = self.graph.execute(q).await?;
        let mut out = Vec::new();
        while let Some(row) = stream.next().await? {
            out.push(artist_from_row(&row)?);
        }
        Ok(out)
    }

    pub async fn get_artist(&self, mbid: &str) -> AppResult<Option<Artist>> {
        let q = query(&format!("MATCH (a:Artist {{mbid: $mbid}}) RETURN {ARTIST_FIELDS}"))
            .param("mbid", mbid);
        let mut stream = self.graph.execute(q).await?;
        match stream.next().await? {
            Some(row) => Ok(Some(artist_from_row(&row)?)),
            None => Ok(None),
        }
    }

    pub async fn get_artist_recordings(&self, mbid: &str) -> AppResult<Vec<Recording>> {
        let q = query(&format!(
            "MATCH (a:Artist {{mbid: $mbid}})-[:PERFORMED|FEATURED_ON]->(r:Recording) \
             RETURN DISTINCT {RECORDING_FIELDS} ORDER BY r.popularity DESC"
        ))
        .param("mbid", mbid);
        let mut stream = self.graph.execute(q).await?;
        let mut out = Vec::new();
        while let Some(row) = stream.next().await? {
            out.push(recording_from_row(&row)?);
        }
        Ok(out)
    }

    pub async fn get_artist_releases(&self, mbid: &str) -> AppResult<Vec<Release>> {
        let q = query(&format!(
            "MATCH (a:Artist {{mbid: $mbid}})-[:PERFORMED|FEATURED_ON]->(:Recording)-[:APPEARS_ON]->(rel:Release) \
             RETURN DISTINCT {RELEASE_FIELDS} ORDER BY rel.date"
        ))
        .param("mbid", mbid);
        let mut stream = self.graph.execute(q).await?;
        let mut out = Vec::new();
        while let Some(row) = stream.next().await? {
            out.push(release_from_row(&row)?);
        }
        Ok(out)
    }

    pub async fn get_artist_collaborations(&self, mbid: &str) -> AppResult<Vec<Collaboration>> {
        let q = query(&format!(
            "MATCH (a:Artist {{mbid: $mbid}})-[rel:COLLABORATED_WITH]-(b:Artist) \
             RETURN {fields}, rel.weight AS weight, rel.sharedRecordings AS sharedRecordings \
             ORDER BY rel.weight DESC",
            fields = ARTIST_FIELDS.replace("a.", "b.")
        ))
        .param("mbid", mbid);
        let mut stream = self.graph.execute(q).await?;
        let mut out = Vec::new();
        while let Some(row) = stream.next().await? {
            out.push(Collaboration {
                artist: artist_from_row(&row)?,
                weight: req(&row, "weight")?,
                shared_recordings: opt(&row, "sharedRecordings").unwrap_or_default(),
            });
        }
        Ok(out)
    }

    // -- Reads: recordings ----------------------------------------------------

    pub async fn list_recordings(&self, limit: i64, offset: i64) -> AppResult<Vec<Recording>> {
        let q = query(&format!(
            "MATCH (r:Recording) RETURN {RECORDING_FIELDS} ORDER BY r.popularity DESC SKIP $offset LIMIT $limit"
        ))
        .param("offset", offset)
        .param("limit", limit);
        let mut stream = self.graph.execute(q).await?;
        let mut out = Vec::new();
        while let Some(row) = stream.next().await? {
            out.push(recording_from_row(&row)?);
        }
        Ok(out)
    }

    pub async fn get_recording(&self, mbid: &str) -> AppResult<Option<Recording>> {
        let q = query(&format!("MATCH (r:Recording {{mbid: $mbid}}) RETURN {RECORDING_FIELDS}"))
            .param("mbid", mbid);
        let mut stream = self.graph.execute(q).await?;
        match stream.next().await? {
            Some(row) => Ok(Some(recording_from_row(&row)?)),
            None => Ok(None),
        }
    }

    pub async fn get_recording_artists(&self, mbid: &str) -> AppResult<Vec<Artist>> {
        let q = query(&format!(
            "MATCH (a:Artist)-[:PERFORMED|FEATURED_ON]->(r:Recording {{mbid: $mbid}}) RETURN {ARTIST_FIELDS}"
        ))
        .param("mbid", mbid);
        let mut stream = self.graph.execute(q).await?;
        let mut out = Vec::new();
        while let Some(row) = stream.next().await? {
            out.push(artist_from_row(&row)?);
        }
        Ok(out)
    }

    pub async fn get_recording_releases(&self, mbid: &str) -> AppResult<Vec<Release>> {
        let q = query(&format!(
            "MATCH (r:Recording {{mbid: $mbid}})-[:APPEARS_ON]->(rel:Release) RETURN {RELEASE_FIELDS}"
        ))
        .param("mbid", mbid);
        let mut stream = self.graph.execute(q).await?;
        let mut out = Vec::new();
        while let Some(row) = stream.next().await? {
            out.push(release_from_row(&row)?);
        }
        Ok(out)
    }

    // -- Reads: releases --------------------------------------------------------

    pub async fn list_releases(&self, limit: i64, offset: i64) -> AppResult<Vec<Release>> {
        let q = query(&format!(
            "MATCH (rel:Release) RETURN {RELEASE_FIELDS} ORDER BY rel.date DESC SKIP $offset LIMIT $limit"
        ))
        .param("offset", offset)
        .param("limit", limit);
        let mut stream = self.graph.execute(q).await?;
        let mut out = Vec::new();
        while let Some(row) = stream.next().await? {
            out.push(release_from_row(&row)?);
        }
        Ok(out)
    }

    pub async fn get_release(&self, mbid: &str) -> AppResult<Option<Release>> {
        let q = query(&format!("MATCH (rel:Release {{mbid: $mbid}}) RETURN {RELEASE_FIELDS}"))
            .param("mbid", mbid);
        let mut stream = self.graph.execute(q).await?;
        match stream.next().await? {
            Some(row) => Ok(Some(release_from_row(&row)?)),
            None => Ok(None),
        }
    }

    pub async fn get_release_recordings(&self, mbid: &str) -> AppResult<Vec<Recording>> {
        let q = query(&format!(
            "MATCH (r:Recording)-[:APPEARS_ON]->(rel:Release {{mbid: $mbid}}) RETURN {RECORDING_FIELDS}"
        ))
        .param("mbid", mbid);
        let mut stream = self.graph.execute(q).await?;
        let mut out = Vec::new();
        while let Some(row) = stream.next().await? {
            out.push(recording_from_row(&row)?);
        }
        Ok(out)
    }

    pub async fn get_release_artists(&self, mbid: &str) -> AppResult<Vec<Artist>> {
        let q = query(&format!(
            "MATCH (a:Artist)-[:PERFORMED|FEATURED_ON]->(:Recording)-[:APPEARS_ON]->(rel:Release {{mbid: $mbid}}) \
             RETURN DISTINCT {ARTIST_FIELDS}"
        ))
        .param("mbid", mbid);
        let mut stream = self.graph.execute(q).await?;
        let mut out = Vec::new();
        while let Some(row) = stream.next().await? {
            out.push(artist_from_row(&row)?);
        }
        Ok(out)
    }

    // -- Reads: search ------------------------------------------------------

    /// Search already-imported artists by (case-insensitive) name substring.
    /// This is distinct from `musicbrainz::search_artists`, which searches
    /// the external MusicBrainz catalog.
    pub async fn search_local_artists(&self, name_query: &str, limit: i64) -> AppResult<Vec<Artist>> {
        let q = query(&format!(
            "MATCH (a:Artist) WHERE toLower(a.name) CONTAINS toLower($q) \
             RETURN {ARTIST_FIELDS} ORDER BY a.name LIMIT $limit"
        ))
        .param("q", name_query)
        .param("limit", limit);
        let mut stream = self.graph.execute(q).await?;
        let mut out = Vec::new();
        while let Some(row) = stream.next().await? {
            out.push(artist_from_row(&row)?);
        }
        Ok(out)
    }

    // -- Reads: graph ---------------------------------------------------------

    /// A bounded snapshot of the whole graph (artists + their recordings,
    /// releases and collaborations), shaped for `react-force-graph`.
    pub async fn graph_full(&self, limit: i64) -> AppResult<GraphData> {
        let mut nodes: HashMap<String, GraphNode> = HashMap::new();
        let mut links: HashSet<(String, String, String)> = HashSet::new();
        let mut out_links = Vec::new();

        let q = query(&format!("MATCH (a:Artist) RETURN {ARTIST_FIELDS} LIMIT $limit"))
            .param("limit", limit);
        let mut stream = self.graph.execute(q).await?;
        let mut artist_ids = Vec::new();
        while let Some(row) = stream.next().await? {
            let a = artist_from_row(&row)?;
            artist_ids.push(a.mbid.clone());
            nodes.insert(a.mbid.clone(), artist_node(&a));
        }

        if artist_ids.is_empty() {
            return Ok(GraphData::default());
        }

        let q = query(&format!(
            "MATCH (a:Artist)-[rel:PERFORMED|FEATURED_ON]->(r:Recording) WHERE a.mbid IN $ids \
             RETURN a.mbid AS artistId, type(rel) AS relType, {RECORDING_FIELDS}"
        ))
        .param("ids", artist_ids.clone());
        let mut stream = self.graph.execute(q).await?;
        let mut recording_ids = Vec::new();
        while let Some(row) = stream.next().await? {
            let r = recording_from_row(&row)?;
            let artist_id: String = req(&row, "artistId")?;
            let rel_type: String = req(&row, "relType")?;
            recording_ids.push(r.mbid.clone());
            nodes.insert(r.mbid.clone(), recording_node(&r));
            if links.insert((artist_id.clone(), r.mbid.clone(), rel_type.clone())) {
                out_links.push(GraphLink {
                    source: artist_id,
                    target: r.mbid,
                    link_type: rel_type,
                    weight: None,
                });
            }
        }

        if !recording_ids.is_empty() {
            let q = query(&format!(
                "MATCH (r:Recording)-[:APPEARS_ON]->(rel:Release) WHERE r.mbid IN $ids \
                 RETURN r.mbid AS recordingId, {RELEASE_FIELDS}"
            ))
            .param("ids", recording_ids);
            let mut stream = self.graph.execute(q).await?;
            while let Some(row) = stream.next().await? {
                let rel = release_from_row(&row)?;
                let recording_id: String = req(&row, "recordingId")?;
                nodes.insert(rel.mbid.clone(), release_node(&rel));
                if links.insert((recording_id.clone(), rel.mbid.clone(), "APPEARS_ON".to_string())) {
                    out_links.push(GraphLink {
                        source: recording_id,
                        target: rel.mbid,
                        link_type: "APPEARS_ON".to_string(),
                        weight: None,
                    });
                }
            }
        }

        let q = query(
            "MATCH (a:Artist)-[c:COLLABORATED_WITH]->(b:Artist) WHERE a.mbid IN $ids AND b.mbid IN $ids \
             RETURN a.mbid AS aId, b.mbid AS bId, c.weight AS weight",
        )
        .param("ids", artist_ids);
        let mut stream = self.graph.execute(q).await?;
        while let Some(row) = stream.next().await? {
            let a_id: String = req(&row, "aId")?;
            let b_id: String = req(&row, "bId")?;
            let weight: i64 = req(&row, "weight")?;
            if links.insert((a_id.clone(), b_id.clone(), "COLLABORATED_WITH".to_string())) {
                out_links.push(GraphLink {
                    source: a_id,
                    target: b_id,
                    link_type: "COLLABORATED_WITH".to_string(),
                    weight: Some(weight),
                });
            }
        }

        Ok(GraphData {
            nodes: nodes.into_values().collect(),
            links: out_links,
        })
    }

    /// Neighborhood graph centered on one artist: the artist, its recordings,
    /// those recordings' releases, and its direct collaborators.
    pub async fn graph_for_artist(&self, mbid: &str) -> AppResult<GraphData> {
        let mut nodes: HashMap<String, GraphNode> = HashMap::new();
        let mut out_links = Vec::new();

        let Some(center) = self.get_artist(mbid).await? else {
            return Ok(GraphData::default());
        };
        nodes.insert(center.mbid.clone(), artist_node(&center));

        let q = query(&format!(
            "MATCH (a:Artist {{mbid: $mbid}})-[rel:PERFORMED|FEATURED_ON]->(r:Recording) \
             RETURN type(rel) AS relType, {RECORDING_FIELDS}"
        ))
        .param("mbid", mbid);
        let mut stream = self.graph.execute(q).await?;
        let mut recording_ids = Vec::new();
        while let Some(row) = stream.next().await? {
            let r = recording_from_row(&row)?;
            let rel_type: String = req(&row, "relType")?;
            recording_ids.push(r.mbid.clone());
            nodes.insert(r.mbid.clone(), recording_node(&r));
            out_links.push(GraphLink {
                source: center.mbid.clone(),
                target: r.mbid,
                link_type: rel_type,
                weight: None,
            });
        }

        if !recording_ids.is_empty() {
            let q = query(&format!(
                "MATCH (r:Recording)-[:APPEARS_ON]->(rel:Release) WHERE r.mbid IN $ids \
                 RETURN r.mbid AS recordingId, {RELEASE_FIELDS}"
            ))
            .param("ids", recording_ids);
            let mut stream = self.graph.execute(q).await?;
            while let Some(row) = stream.next().await? {
                let rel = release_from_row(&row)?;
                let recording_id: String = req(&row, "recordingId")?;
                nodes.insert(rel.mbid.clone(), release_node(&rel));
                out_links.push(GraphLink {
                    source: recording_id,
                    target: rel.mbid,
                    link_type: "APPEARS_ON".to_string(),
                    weight: None,
                });
            }
        }

        for collab in self.get_artist_collaborations(mbid).await? {
            nodes.insert(collab.artist.mbid.clone(), artist_node(&collab.artist));
            out_links.push(GraphLink {
                source: center.mbid.clone(),
                target: collab.artist.mbid,
                link_type: "COLLABORATED_WITH".to_string(),
                weight: Some(collab.weight),
            });
        }

        Ok(GraphData {
            nodes: nodes.into_values().collect(),
            links: out_links,
        })
    }

    /// The collaboration network only (all artists that have at least one
    /// collaboration, and the edges between them) — used by the dedicated
    /// Graph page's "collaborations" view and by `/api/stats/*`.
    pub async fn graph_collaborations(&self, limit: i64) -> AppResult<GraphData> {
        let a_fields = "a.mbid AS aMbid, a.name AS aName, a.type AS aType, a.country AS aCountry, \
             a.gender AS aGender, a.beginDate AS aBeginDate, a.endDate AS aEndDate, a.disambiguation AS aDisambiguation";
        let b_fields = "b.mbid AS bMbid, b.name AS bName, b.type AS bType, b.country AS bCountry, \
             b.gender AS bGender, b.beginDate AS bBeginDate, b.endDate AS bEndDate, b.disambiguation AS bDisambiguation";

        let q = query(&format!(
            "MATCH (a:Artist)-[c:COLLABORATED_WITH]->(b:Artist) \
             WITH a, b, c LIMIT $limit \
             RETURN {a_fields}, {b_fields}, c.weight AS weight"
        ))
        .param("limit", limit);

        let mut nodes: HashMap<String, GraphNode> = HashMap::new();
        let mut out_links = Vec::new();
        let mut stream = self.graph.execute(q).await?;
        while let Some(row) = stream.next().await? {
            let a = artist_from_row_prefixed(&row, "a")?;
            let b = artist_from_row_prefixed(&row, "b")?;
            let weight: i64 = req(&row, "weight")?;
            nodes.insert(a.mbid.clone(), artist_node(&a));
            nodes.insert(b.mbid.clone(), artist_node(&b));
            out_links.push(GraphLink {
                source: a.mbid,
                target: b.mbid,
                link_type: "COLLABORATED_WITH".to_string(),
                weight: Some(weight),
            });
        }

        Ok(GraphData {
            nodes: nodes.into_values().collect(),
            links: out_links,
        })
    }

    // -- Reads: stats ---------------------------------------------------------

    async fn count_nodes(&self, label: &str) -> AppResult<i64> {
        let q = query(&format!("MATCH (n:{label}) RETURN count(n) AS c"));
        let mut stream = self.graph.execute(q).await?;
        match stream.next().await? {
            Some(row) => req(&row, "c"),
            None => Ok(0),
        }
    }

    pub async fn stats_overview(&self) -> AppResult<OverviewStats> {
        let q = query("MATCH (:Artist)-[c:COLLABORATED_WITH]->(:Artist) RETURN count(c) AS c");
        let mut stream = self.graph.execute(q).await?;
        let collaboration_count = match stream.next().await? {
            Some(row) => req(&row, "c")?,
            None => 0,
        };

        Ok(OverviewStats {
            artist_count: self.count_nodes("Artist").await?,
            recording_count: self.count_nodes("Recording").await?,
            release_count: self.count_nodes("Release").await?,
            label_count: self.count_nodes("Label").await?,
            genre_count: self.count_nodes("Genre").await?,
            area_count: self.count_nodes("Area").await?,
            collaboration_count,
        })
    }

    pub async fn stats_top_artists(&self, limit: i64) -> AppResult<Vec<ArtistStat>> {
        let q = query(&format!(
            "MATCH (a:Artist)-[rel:COLLABORATED_WITH]-() \
             WITH a, count(rel) AS cnt \
             RETURN {ARTIST_FIELDS}, cnt ORDER BY cnt DESC LIMIT $limit"
        ))
        .param("limit", limit);
        let mut stream = self.graph.execute(q).await?;
        let mut out = Vec::new();
        while let Some(row) = stream.next().await? {
            out.push(ArtistStat {
                artist: artist_from_row(&row)?,
                count: req(&row, "cnt")?,
            });
        }
        Ok(out)
    }

    pub async fn stats_top_collaborations(&self, limit: i64) -> AppResult<Vec<CollaborationStat>> {
        let a_fields = "a.mbid AS aMbid, a.name AS aName, a.type AS aType, a.country AS aCountry, \
             a.gender AS aGender, a.beginDate AS aBeginDate, a.endDate AS aEndDate, a.disambiguation AS aDisambiguation";
        let b_fields = "b.mbid AS bMbid, b.name AS bName, b.type AS bType, b.country AS bCountry, \
             b.gender AS bGender, b.beginDate AS bBeginDate, b.endDate AS bEndDate, b.disambiguation AS bDisambiguation";
        let q = query(&format!(
            "MATCH (a:Artist)-[rel:COLLABORATED_WITH]->(b:Artist) \
             RETURN {a_fields}, {b_fields}, rel.weight AS weight \
             ORDER BY rel.weight DESC LIMIT $limit"
        ))
        .param("limit", limit);
        let mut stream = self.graph.execute(q).await?;
        let mut out = Vec::new();
        while let Some(row) = stream.next().await? {
            out.push(CollaborationStat {
                artist_a: artist_from_row_prefixed(&row, "a")?,
                artist_b: artist_from_row_prefixed(&row, "b")?,
                weight: req(&row, "weight")?,
            });
        }
        Ok(out)
    }

    pub async fn stats_top_genres(&self, limit: i64) -> AppResult<Vec<GenreStat>> {
        let q = query(
            "MATCH (:Artist)-[:ASSOCIATED_WITH_GENRE]->(g:Genre) \
             RETURN g.name AS name, count(*) AS cnt ORDER BY cnt DESC LIMIT $limit",
        )
        .param("limit", limit);
        let mut stream = self.graph.execute(q).await?;
        let mut out = Vec::new();
        while let Some(row) = stream.next().await? {
            out.push(GenreStat {
                genre: req(&row, "name")?,
                count: req(&row, "cnt")?,
            });
        }
        Ok(out)
    }
}
