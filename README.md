# MusicGraph

> Exploration des collaborations musicales avec MusicBrainz et Neo4j — projet B3 Dev & B3 Data.

MusicGraph recherche des artistes sur [MusicBrainz](https://musicbrainz.org/), les importe dans
[Neo4j](https://neo4j.com/) avec leurs morceaux, releases et collaborations détectées, et expose le
tout via une API REST et une interface web permettant d'explorer le réseau musical résultant :
qui a collaboré avec qui, quels morceaux font le pont entre plusieurs artistes, quels genres
dominent, qui est le plus connecté...

**Stack :** Rust (Axum + neo4rs + reqwest) · Neo4j · React (Vite + TypeScript) · Docker Compose.

## Sommaire

- [Quickstart](#quickstart)
- [Architecture](#architecture)
- [Modèle de données](#modèle-de-données)
- [Développement local (sans Docker)](#développement-local-sans-docker)
- [Documentation](#documentation)
- [Problématique → réponses](#problématique--réponses)
- [Limites connues](#limites-connues)

## Quickstart

```bash
cp .env.example .env
docker compose up --build
```

Au premier démarrage, le service `seed` importe automatiquement 10 artistes (Daft Punk, Beyoncé,
JAY‑Z, Kendrick Lamar, Angèle, Stromae, Ninho, Damso, SCH, PNL) depuis le snapshot committé
`data/seed.json` — pas besoin d'accès internet pour avoir un graphe déjà peuplé.

| Service | URL |
|---|---|
| Frontend | http://localhost:5173 |
| API backend | http://localhost:8080/api |
| Neo4j Browser | http://localhost:7474 (identifiants dans `.env`) |

## Architecture

```
┌────────────┐      recherche/import       ┌──────────────┐
│  MusicBrainz│◄────────────────────────────│              │
│  + Cover Art│─────────────────────────────►   Backend    │
│   Archive   │      artistes/morceaux/     │  (Rust/Axum) │
└────────────┘      releases/relations      └──────┬───────┘
                                                     │ Bolt (neo4rs)
                                                     ▼
┌────────────┐        REST JSON /api/*      ┌──────────────┐
│  Frontend  │◄──────────────────────────────┤    Neo4j     │
│(React/Vite)│                               │   (graphe)   │
└────────────┘                               └──────────────┘
```

```
musicgraph/
├── backend/    API Rust (Axum) : import MusicBrainz, accès Neo4j, endpoints REST
├── frontend/   Interface web React (recherche, fiches artiste, graphe, stats)
├── data/       Jeu de données (snapshot MusicBrainz, voir data/README.md)
├── docs/       Modèle de données, référence API, analyse data
└── docker-compose.yml
```

## Modèle de données

```
(Artist)-[:FROM_AREA]->(Area)
(Artist)-[:ASSOCIATED_WITH_GENRE]->(Genre)
(Artist)-[:PERFORMED]->(Recording)-[:APPEARS_ON]->(Release)-[:RELEASED_BY]->(Label)
(Artist)-[:FEATURED_ON]->(Recording)
(Artist)-[:COLLABORATED_WITH]->(Artist)   { weight, sharedRecordings }
```

Chaque nœud est déduplicé par son `mbid` MusicBrainz via une contrainte d'unicité Neo4j + des
écritures `MERGE`. Détail complet, choix de modélisation et limites assumées : **[docs/data-model.md](docs/data-model.md)**.

## Développement local (sans Docker)

Prérequis : [Rust](https://rustup.rs/) (édition 2024), [Bun](https://bun.sh/), une instance Neo4j
accessible (locale ou via `docker compose up neo4j`).

```bash
# Backend
cd backend
cp ../.env.example .env   # ou exportez les variables directement
cargo run                  # sert l'API sur :8080

# Seed (dans un autre terminal, une fois Neo4j démarré)
cargo run --bin seed

# Frontend
cd frontend
cp .env.example .env
bun install
bun dev                     # sert l'UI sur :5173
```

Commandes utiles : `cargo test`, `cargo clippy --all-targets`, `bun run build`, `bun run lint`.

## Documentation

- **[docs/data-model.md](docs/data-model.md)** — nœuds, relations, détection des collaborations, choix et limites.
- **[docs/api.md](docs/api.md)** — référence complète des endpoints `/api/*`.
- **[docs/analysis.md](docs/analysis.md)** — analyse du jeu de données livré (top collaborations, genres, artistes connectés, limites).
- **[data/README.md](data/README.md)** — comment `data/seed.json` a été produit et comment le régénérer.

## Problématique → réponses

| Question | Où la trouver |
|---|---|
| Quels morceaux sont associés à un artiste ? | `GET /api/artists/:id/recordings`, onglet *Morceaux* d'une fiche artiste |
| Quels artistes ont collaboré ensemble ? | `GET /api/artists/:id/collaborations`, `GET /api/graph/collaborations` |
| Quels artistes apparaissent en featuring ? | Relations `FEATURED_ON` (distinguées de `PERFORMED`) |
| Quels albums/releases contiennent ces morceaux ? | `GET /api/artists/:id/releases`, `GET /api/recordings/:id/releases` |
| Quels artistes sont les plus connectés ? | `GET /api/stats/top-artists`, page *Statistiques* |
| Quels genres musicaux sont les plus présents ? | `GET /api/stats/top-genres`, page *Statistiques* |
| Quels chemins relient deux artistes ? | Page *Graphe* (vue d'ensemble ou centrée sur un artiste), navigable de proche en proche |
| Quels morceaux créent des ponts entre plusieurs artistes ? | Recordings à crédits multiples — voir `docs/analysis.md` (ex. featurings, posse cuts) |

## Limites connues

Voir `docs/analysis.md` § *Limites générales du graphe* pour le détail (doublons de morceaux/éditions,
score de popularité interne non représentatif d'une écoute réelle, `Release -[:RELEASED_IN]-> Area`
non implémenté, enrichissement labels/pochettes borné). Ce dépôt a été développé et vérifié dans un
environnement sans Docker/Neo4j local : le backend (build, clippy, et un run réel du binaire `seed`
contre l'API MusicBrainz en direct) et le frontend (typecheck, lint, build, démarrage du serveur de
dev) ont été validés individuellement, mais l'enchaînement complet via `docker compose up` n'a pas pu
être exécuté de bout en bout dans cet environnement — à vérifier en premier lieu par quiconque reprend
le projet.
