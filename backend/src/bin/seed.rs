//! Seeds Neo4j with a fixed set of well-known artists.
//!
//! If `data/seed.json` exists, it's loaded and replayed offline (no
//! MusicBrainz calls, no rate limiting, deterministic — this is what
//! `docker compose up` uses so the app is populated even without internet
//! access). Otherwise, this binary resolves each artist name to an MBID via
//! a live MusicBrainz search, fetches its full bundle, imports it, and
//! writes the snapshot to `data/seed.json` for next time.
//!
//! Run with `cargo run --bin seed` (from `backend/`), or `SEED_FILE=... cargo
//! run --bin seed` to point at a specific snapshot path.

use std::path::Path;

use serde::{Deserialize, Serialize};
use tracing_subscriber::EnvFilter;

use musicgraph_backend::config::Config;
use musicgraph_backend::db;
use musicgraph_backend::importer::{self, ArtistBundle};
use musicgraph_backend::musicbrainz::MusicBrainzClient;
use musicgraph_backend::repo::Repo;

/// The artists named in the project spec's search examples.
const SEED_ARTISTS: &[&str] = &[
    "Daft Punk",
    "Beyoncé",
    "Jay-Z",
    "Kendrick Lamar",
    "Angèle",
    "Stromae",
    "Ninho",
    "Damso",
    "SCH",
    "PNL",
];

#[derive(Debug, Serialize, Deserialize)]
struct SeedFile {
    artists: Vec<ArtistBundle>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    // `--fetch-only` regenerates data/seed.json from live MusicBrainz data
    // without touching Neo4j at all — useful for refreshing the committed
    // snapshot from a machine that has internet access but no local Neo4j.
    let fetch_only = std::env::args().any(|a| a == "--fetch-only");

    let config = Config::from_env();
    let mb = MusicBrainzClient::new(
        config.musicbrainz_base_url.clone(),
        config.musicbrainz_user_agent.clone(),
    )?;

    let default_seed_path = format!("{}/../data/seed.json", env!("CARGO_MANIFEST_DIR"));
    let seed_path = config.seed_file.clone().unwrap_or(default_seed_path);

    let bundles = if fetch_only || !Path::new(&seed_path).exists() {
        tracing::info!("fetching live from MusicBrainz");
        let bundles = fetch_all(&mb).await;
        save_snapshot(&seed_path, &bundles)?;
        bundles
    } else {
        tracing::info!(path = %seed_path, "loading cached seed snapshot (offline import)");
        load_snapshot(&seed_path)?
    };

    if bundles.is_empty() {
        tracing::warn!("no artist bundles to import (all fetches failed?)");
    }

    if fetch_only {
        tracing::info!(count = bundles.len(), "fetch-only mode: snapshot written, skipping Neo4j import");
        return Ok(());
    }

    let graph = db::connect_and_migrate(&config).await?;
    let repo = Repo::new(graph);

    for bundle in &bundles {
        tracing::info!(artist = %bundle.artist.name, mbid = %bundle.artist.id, "importing");
        if let Err(e) = importer::import_bundle(&repo, &mb, bundle).await {
            tracing::error!(artist = %bundle.artist.name, error = %e, "failed to import artist, skipping");
        }
    }

    tracing::info!(count = bundles.len(), "seed complete");
    Ok(())
}

fn load_snapshot(path: &str) -> anyhow::Result<Vec<ArtistBundle>> {
    let content = std::fs::read_to_string(path)?;
    let seed_file: SeedFile = serde_json::from_str(&content)?;
    Ok(seed_file.artists)
}

fn save_snapshot(path: &str, bundles: &[ArtistBundle]) -> anyhow::Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(&SeedFile {
        artists: bundles.to_vec(),
    })?;
    std::fs::write(path, json)?;
    tracing::info!(path, "wrote seed snapshot");
    Ok(())
}

async fn fetch_all(mb: &MusicBrainzClient) -> Vec<ArtistBundle> {
    let mut bundles = Vec::new();
    for name in SEED_ARTISTS {
        match resolve_and_fetch(mb, name).await {
            Ok(bundle) => bundles.push(bundle),
            Err(e) => tracing::warn!(artist = name, error = %e, "failed to fetch artist, skipping"),
        }
    }
    bundles
}

/// Resolves an artist name to an MBID via a live MusicBrainz search (taking
/// the top-scored hit), then fetches its full import bundle.
async fn resolve_and_fetch(mb: &MusicBrainzClient, name: &str) -> anyhow::Result<ArtistBundle> {
    let search = mb.search_artists(name, 1).await?;
    let top = search
        .artists
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("no MusicBrainz results for '{name}'"))?;
    let bundle = importer::fetch_artist_bundle(mb, &top.id).await?;
    Ok(bundle)
}
