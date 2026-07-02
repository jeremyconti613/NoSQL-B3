use neo4rs::Graph;

use crate::config::Config;

/// Connects to Neo4j and ensures the schema (uniqueness constraints) exists.
///
/// Uniqueness constraints on every `mbid` are what make our writes idempotent:
/// every import uses `MERGE` keyed on `mbid`, so re-importing the same artist
/// (or seeding twice) never creates duplicate nodes.
pub async fn connect_and_migrate(config: &Config) -> anyhow::Result<Graph> {
    let graph = Graph::new(&config.neo4j_uri, &config.neo4j_user, &config.neo4j_password).await?;

    let constraints = [
        "CREATE CONSTRAINT artist_mbid IF NOT EXISTS FOR (a:Artist) REQUIRE a.mbid IS UNIQUE",
        "CREATE CONSTRAINT recording_mbid IF NOT EXISTS FOR (r:Recording) REQUIRE r.mbid IS UNIQUE",
        "CREATE CONSTRAINT release_mbid IF NOT EXISTS FOR (r:Release) REQUIRE r.mbid IS UNIQUE",
        "CREATE CONSTRAINT label_mbid IF NOT EXISTS FOR (l:Label) REQUIRE l.mbid IS UNIQUE",
        "CREATE CONSTRAINT area_mbid IF NOT EXISTS FOR (a:Area) REQUIRE a.mbid IS UNIQUE",
        "CREATE CONSTRAINT genre_name IF NOT EXISTS FOR (g:Genre) REQUIRE g.name IS UNIQUE",
    ];

    for stmt in constraints {
        graph.run(neo4rs::query(stmt)).await?;
        tracing::debug!(statement = stmt, "ensured constraint");
    }

    tracing::info!("connected to Neo4j and ensured schema constraints");
    Ok(graph)
}
