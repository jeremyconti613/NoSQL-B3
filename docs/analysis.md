# Analyse des données

Cette analyse porte sur le jeu de données livré dans `data/seed.json` : 10 artistes
(Daft Punk, Beyoncé, JAY‑Z, Kendrick Lamar, Angèle, Stromae, Ninho, Damso, SCH, PNL) et 20 morceaux
récents/représentatifs par artiste (200 morceaux au total), avec leurs releases. Les chiffres
ci-dessous sont calculés directement sur ce jeu de données (script Python ad hoc sur le JSON brut,
avant import — les valeurs exactes en base peuvent différer légèrement une fois l'enrichissement des
releases et la déduplication `MERGE` appliqués).

## Top collaborations (par morceaux partagés)

| Paire | Morceaux partagés |
|---|---|
| Beyoncé × JAY‑Z | 20 |
| Dua Lipa × Angèle | 5 |
| Chimamanda Ngozi Adichie × Beyoncé | 4 |
| Big L × JAY‑Z | 4 |
| SZA × Kendrick Lamar | 3 |
| SCH × Zola | 3 |

Le couple Beyoncé/JAY‑Z domine largement : la plupart de leurs morceaux partagés sont en réalité des
**variantes du même titre** (`'03 Bonnie & Clyde`, `instrumental`, `radio edit`, `video`...), pas 20
morceaux distincts au sens artistique. C'est une limite réelle du modèle : MusicBrainz distingue
volontairement chaque variante audio par son propre MBID `Recording`, donc notre score de
collaboration (nombre de `Recording` partagés) surpondère les artistes dont les morceaux existent en
de nombreuses versions (remix, live, radio edit...).

## Top genres

| Genre | Artistes concernés (sur 10) |
|---|---|
| trap | 6 |
| hip hop | 6 |
| pop | 5 |
| pop rap | 5 |
| r&b | 4 |
| dance / disco / house | 3 chacun |

Sans surprise, le rap/hip-hop domine l'échantillon (6 des 10 artistes), avec un second pôle
électronique/pop (Daft Punk, Stromae, Angèle). Les tags de genre viennent directement des
folksonomies MusicBrainz (`genres` sur l'artiste) : ils sont donc communautaires, pas une
classification officielle, et peuvent être absents ou incohérents pour des artistes moins documentés.

## Artistes les plus connectés

Sur cet échantillon, **SCH** ressort avec le plus grand nombre de collaborateurs distincts (37), suivi
de près par plusieurs artistes crédités sur un même morceau collectif (`13 Organisé (Bonus Track)`, un
"posse cut" créditant plus de 30 artistes). C'est la limite la plus importante à connaître pour
l'interprétation du graphe : **un seul morceau à crédits multiples peut créer des dizaines d'arêtes
`COLLABORATED_WITH` d'un coup**, ce qui peut faire remonter artificiellement des artistes autrement
peu connectés dans le classement "top artistes". Une évolution possible serait de pondérer différemment
les collaborations à N > 2 artistes (compilations, bandes-originales) par rapport aux featurings à deux.

## Limites générales du graphe

- **Couverture partielle** : seuls les artistes explicitement recherchés puis importés (bouton
  "Importer") existent dans le graphe ; un featuring vers un artiste jamais importé n'apparaît que
  comme un nœud minimal (`mbid` + `name`), sans détail (pays, genres...) tant qu'il n'est pas
  lui-même importé.
- **Doublons de morceaux** : MusicBrainz modélise chaque édition/remix/instrumental d'un titre comme
  un `Recording` séparé ; le graphe ne les fusionne pas (voir ci-dessus), ce qui peut gonfler les
  compteurs de morceaux et de collaborations pour les artistes aux nombreuses rééditions.
- **Score de popularité** : purement interne (releases + crédits), pas une mesure d'écoute/de
  popularité réelle — voir `docs/data-model.md`.
- **Releases → Area** non modélisé (code pays brut conservé sur `Release.country`) et **labels**
  limités aux 10 premières releases uniques par import, pour borner le nombre d'appels MusicBrainz —
  voir `docs/data-model.md` pour la justification.
- **Genres/aires dépendent de la qualité des données MusicBrainz** elles-mêmes : certains artistes
  (surtout hors scène anglophone/franco-belge) ont des fiches nettement moins complètes.
