pub mod config;
pub mod db;
pub mod error;
pub mod importer;
pub mod models;
pub mod musicbrainz;
pub mod repo;
pub mod routes;

use std::net::SocketAddr;

use axum::http::HeaderValue;
use tower_http::cors::{Any, CorsLayer};

use musicbrainz::MusicBrainzClient;
use repo::Repo;
use routes::AppState;

/// Connects to Neo4j, builds the MusicBrainz client, and serves the API.
/// Used by `main.rs`; kept in the library crate so `src/bin/seed.rs` can
/// reuse `config`/`db`/`repo`/`importer`/`musicbrainz` without duplicating them.
pub async fn run() -> anyhow::Result<()> {
    let config = config::Config::from_env();

    let graph = db::connect_and_migrate(&config).await?;
    let repo = Repo::new(graph);
    let mb = MusicBrainzClient::new(
        config.musicbrainz_base_url.clone(),
        config.musicbrainz_user_agent.clone(),
    )?;

    let state = AppState { repo, mb };

    // Allow the configured frontend origin; fall back to permissive CORS if
    // the configured value doesn't parse as a header (e.g. "*").
    let cors = match config.frontend_origin.parse::<HeaderValue>() {
        Ok(origin) => CorsLayer::new()
            .allow_origin(origin)
            .allow_methods(Any)
            .allow_headers(Any),
        Err(_) => CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any),
    };

    let app = routes::router(state).layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.backend_port));
    tracing::info!(%addr, "starting musicgraph backend");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
