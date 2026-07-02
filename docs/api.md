# Référence API

Base URL : `http://localhost:8080/api` (configurable via `BACKEND_PORT`). Toutes les réponses sont en
JSON ; les erreurs renvoient `{"error": "..."}` avec un code HTTP approprié (400, 404, 502, 500).

## Artistes

| Méthode | Route | Description |
|---|---|---|
| GET | `/artists?limit=&offset=` | Liste les artistes importés (triés par nom). |
| GET | `/artists/:id` | Fiche d'un artiste (404 si non importé). |
| GET | `/artists/:id/recordings` | Morceaux de l'artiste, triés par popularité. |
| GET | `/artists/:id/releases` | Releases sur lesquelles l'artiste apparaît. |
| GET | `/artists/:id/collaborations` | Artistes collaborateurs + poids + morceaux partagés. |
| GET | `/search/artists?q=&limit=` | Recherche **live** dans le catalogue MusicBrainz (pas la base locale). |
| POST | `/import/artists` `{"mbid": "..."}` | Importe/rafraîchit un artiste (idempotent). |

## Morceaux (recordings)

| Méthode | Route | Description |
|---|---|---|
| GET | `/recordings?limit=&offset=` | Liste les morceaux importés, par popularité décroissante. |
| GET | `/recordings/:id` | Détail d'un morceau. |
| GET | `/recordings/:id/artists` | Artistes crédités sur ce morceau. |
| GET | `/recordings/:id/releases` | Releases contenant ce morceau. |

## Releases

| Méthode | Route | Description |
|---|---|---|
| GET | `/releases?limit=&offset=` | Liste les releases importées. |
| GET | `/releases/:id` | Détail d'une release. |
| GET | `/releases/:id/recordings` | Morceaux contenus dans cette release. |
| GET | `/releases/:id/artists` | Artistes apparaissant sur cette release. |

## Graphe

| Méthode | Route | Description |
|---|---|---|
| GET | `/graph?limit=` | Extrait borné du graphe complet (artistes + morceaux + releases + collaborations), au format `{nodes,links}` consommé directement par `react-force-graph`. |
| GET | `/graph/artists/:id` | Sous-graphe centré sur un artiste (ses morceaux, leurs releases, ses collaborateurs directs). |
| GET | `/graph/collaborations?limit=` | Réseau de collaborations uniquement (nœuds `Artist` + arêtes `COLLABORATED_WITH`). |

Format `GraphNode` : `{ id, label, type: "Artist" | "Recording" | "Release" }`.
Format `GraphLink` : `{ source, target, type, weight? }`.

## Statistiques

| Méthode | Route | Description |
|---|---|---|
| GET | `/stats/overview` | Comptes globaux (artistes, morceaux, releases, labels, genres, aires, collaborations). |
| GET | `/stats/top-collaborations?limit=` | Paires d'artistes avec le plus de morceaux partagés. |
| GET | `/stats/top-artists?limit=` | Artistes les plus connectés (nombre de collaborateurs distincts). |
| GET | `/stats/top-genres?limit=` | Genres les plus représentés parmi les artistes importés. |

## Divers

| Méthode | Route | Description |
|---|---|---|
| GET | `/health` | Vérification de santé (200 `"ok"`). |

Voir `docs/data-model.md` pour la forme exacte des objets `Artist`/`Recording`/`Release`/`Collaboration`.
