# Jeu de données

`seed.json` est un instantané des réponses MusicBrainz pour 10 artistes cités en exemple dans le
sujet (Daft Punk, Beyoncé, JAY‑Z, Kendrick Lamar, Angèle, Stromae, Ninho, Damso, SCH, PNL), avec
jusqu'à 20 morceaux (recordings) par artiste et leurs releases associées.

## Comment il a été produit

```bash
cd backend
MUSICBRAINZ_USER_AGENT="MusicGraph/0.1 ( you@example.com )" \
  cargo run --bin seed -- --fetch-only
```

Ce mode :
1. résout chaque nom d'artiste en MBID via une recherche MusicBrainz (`GET /artist?query=`) ;
2. récupère la fiche complète de l'artiste (genres, aire, relations) ;
3. récupère jusqu'à 20 morceaux de l'artiste, chacun enrichi de ses releases (un lookup MusicBrainz
   par morceau — voir `docs/data-model.md` § *"Enrichissement des releases"*) ;
4. écrit le tout dans `data/seed.json`, **sans toucher à Neo4j** (`--fetch-only`).

## Comment il est utilisé

Au démarrage de `docker compose up`, le service `seed` exécute `cargo run --bin seed` (sans
`--fetch-only`) : si `data/seed.json` existe, il est rejoué tel quel (aucun appel MusicBrainz pour la
recherche/les morceaux — seuls les labels et pochettes, en bonus, déclenchent encore quelques appels
bornés, voir `docs/data-model.md`). L'import est idempotent : le relancer plusieurs fois ne crée pas
de doublons (`MERGE` sur les MBID).

Pour régénérer le fichier avec des données plus fraîches ou d'autres artistes, éditez la liste
`SEED_ARTISTS` dans `backend/src/bin/seed.rs` et relancez la commande ci-dessus.
