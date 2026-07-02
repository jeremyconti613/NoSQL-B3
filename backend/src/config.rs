use std::env;

/// Application configuration, loaded from environment variables (see `.env.example`).
#[derive(Debug, Clone)]
pub struct Config {
    pub neo4j_uri: String,
    pub neo4j_user: String,
    pub neo4j_password: String,
    pub backend_port: u16,
    pub musicbrainz_base_url: String,
    pub musicbrainz_user_agent: String,
    pub frontend_origin: String,
    pub seed_file: Option<String>,
}

impl Config {
    /// Loads configuration from the process environment, applying sensible
    /// local-dev defaults for anything not required to be explicit.
    pub fn from_env() -> Self {
        // Best-effort: load a local .env if present (no-op in prod/containers
        // where env vars are already injected by docker-compose).
        let _ = dotenvy::dotenv();

        Self {
            neo4j_uri: env_or("NEO4J_URI", "bolt://localhost:7687"),
            neo4j_user: env_or("NEO4J_USER", "neo4j"),
            neo4j_password: env_or("NEO4J_PASSWORD", "musicgraph_password"),
            backend_port: env_or("BACKEND_PORT", "8080")
                .parse()
                .expect("BACKEND_PORT must be a valid u16"),
            musicbrainz_base_url: env_or("MUSICBRAINZ_BASE_URL", "https://musicbrainz.org/ws/2"),
            musicbrainz_user_agent: env_or(
                "MUSICBRAINZ_USER_AGENT",
                "MusicGraph/0.1 ( https://github.com/example/musicgraph )",
            ),
            frontend_origin: env_or("FRONTEND_ORIGIN", "http://localhost:5173"),
            seed_file: env::var("SEED_FILE").ok(),
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}
