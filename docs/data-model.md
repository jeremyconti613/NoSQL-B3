# Modèle de données Neo4j

## Nœuds

| Label | Clé unique | Propriétés |
|---|---|---|
| `Artist` | `mbid` | `name`, `type`, `country`, `gender`, `beginDate`, `endDate`, `disambiguation` |
| `Recording` | `mbid` | `title`, `length` (ms), `firstReleaseDate`, `popularity`, `source` |
| `Release` | `mbid` | `title`, `date`, `country`, `status`, `releaseType`, `coverArtUrl` |
| `Label` | `mbid` | `name`, `country` |
| `Genre` | `name` | — |
| `Area` | `mbid` | `name`, `type` |

Chaque label a une **contrainte d'unicité** sur sa clé (`CREATE CONSTRAINT ... REQUIRE x.mbid IS UNIQUE`,
créées au démarrage par `backend/src/db.rs`). C'est ce qui garantit qu'un artiste (ou un morceau, une
release...) n'est jamais dupliqué : chaque écriture utilise `MERGE` sur cette clé.

Toutes les écritures utilisent `MERGE (n {mbid: $mbid}) SET n += $props`, où `$props` ne contient que
les champs effectivement connus (non `null`) — voir `backend/src/repo.rs`. Concrètement, si un artiste
est d'abord découvert via un featuring (on ne connaît alors que son `mbid`/`name`) puis importé
pleinement plus tard, le second import complète le nœud sans jamais écraser un champ déjà renseigné
par une valeur manquante.

## Relations

```
(:Artist)-[:PERFORMED]->(:Recording)
(:Artist)-[:FEATURED_ON]->(:Recording)
(:Artist)-[:COLLABORATED_WITH]->(:Artist)   { weight, sharedRecordings }
(:Recording)-[:APPEARS_ON]->(:Release)
(:Release)-[:RELEASED_BY]->(:Label)
(:Artist)-[:ASSOCIATED_WITH_GENRE]->(:Genre)
(:Artist)-[:FROM_AREA]->(:Area)
```

`COLLABORATED_WITH` est stockée **une seule fois par paire non ordonnée**, dans une direction
canonique (MBID le plus petit lexicographiquement en premier) — un artiste a une seule relation
vers un autre, jamais deux (aller et retour), ce qui évite de fausser les comptages. Son poids
(`weight`) compte le nombre d'enregistrements distincts partagés, et `sharedRecordings` liste leurs
MBID ; réimporter un morceau déjà connu n'incrémente pas le poids une deuxième fois.

### `Release -[:RELEASED_IN]-> Area` : non implémenté

Le modèle cible en prévoit une, mais elle n'est pas peuplée par l'import : l'API MusicBrainz n'expose
qu'un simple code pays (`"FR"`, `"XW"`...) sur une release, pas un MBID d'`Area`, et résoudre ce code
en `Area` demanderait un appel supplémentaire (limité à 1 req/s) par pays rencontré. Conformément à la
consigne de qualité *"limitation des appels à MusicBrainz"*, ce code est stocké tel quel dans
`Release.country` plutôt que modélisé comme une relation.

## Détection des collaborations

Trois signaux, combinés dans `backend/src/importer.rs` :

1. **Crédits multiples sur un même enregistrement** (signal principal) : MusicBrainz résout déjà
   chaque `artist-credit` en MBID. Le premier crédité obtient `PERFORMED`, les suivants `FEATURED_ON`,
   et chaque paire distincte de crédités reçoit une arête `COLLABORATED_WITH`.
2. **Marqueurs textuels** (`feat.`, `ft.`, `featuring`, `avec`, ` x `, ` & `) dans le titre ou les
   join-phrases MusicBrainz : utilisés en vérification défensive (journalisés), car MusicBrainz a en
   pratique déjà scindé le crédit correctement dans l'immense majorité des cas.
3. **Relations MusicBrainz explicites** (`inc=artist-rels`, ex. *"member of band"*) : quand présentes,
   elles créent une arête `COLLABORATED_WITH` supplémentaire vers l'artiste lié.

Un garde-fou explicite empêche toute auto-collaboration (`artist_a == artist_b`), utile par exemple
quand un même artiste apparaît deux fois dans une liste de crédits (morceaux multi-parties).

## Score de popularité interne

`Recording.popularity` est un **score interne**, pas une métrique MusicBrainz : nombre de releases sur
lesquelles le morceau apparaît + nombre d'artistes crédités. C'est un proxy volontairement simple pour
"combien ce morceau est diffusé/collaboratif", pas une mesure d'écoute réelle (MusicBrainz n'en fournit
pas).

## Enrichissement des releases (labels, pochettes)

L'endpoint MusicBrainz de *browse* des recordings par artiste ne renvoie pas les releases ; il faut un
lookup dédié par recording (`/recording/{id}?inc=artist-credits+releases`), puis, pour les labels, un
second lookup par release (`/release/{id}?inc=labels+release-groups`). Pour borner la durée d'un
import, cet enrichissement (labels + pochette Cover Art Archive, bonus) est **limité aux 10 premières
releases uniques** rencontrées par artiste importé (voir `MAX_RELEASE_ENRICHMENT_LOOKUPS`). Au-delà,
la release est tout de même créée (titre, date, pays, statut, type), simplement sans label ni pochette.

## Diagramme

```
(Artist)-[:FROM_AREA]->(Area)
(Artist)-[:ASSOCIATED_WITH_GENRE]->(Genre)
(Artist)-[:PERFORMED]->(Recording)-[:APPEARS_ON]->(Release)-[:RELEASED_BY]->(Label)
   ↑                        ↑
(Artist)-[:FEATURED_ON]-----┘
(Artist)-[:COLLABORATED_WITH]->(Artist)
```
