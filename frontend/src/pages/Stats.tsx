import { useEffect, useState } from 'react'
import { api, ApiError } from '../api/client'
import type { ArtistStat, CollaborationStat, GenreStat, OverviewStats } from '../api/types'
import { BarChart } from '../components/BarChart'
import { StatTile } from '../components/StatTile'

export function Stats() {
  const [overview, setOverview] = useState<OverviewStats | null>(null)
  const [topArtists, setTopArtists] = useState<ArtistStat[] | null>(null)
  const [topCollabs, setTopCollabs] = useState<CollaborationStat[] | null>(null)
  const [topGenres, setTopGenres] = useState<GenreStat[] | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    Promise.all([
      api.statsOverview(),
      api.statsTopArtists(10),
      api.statsTopCollaborations(10),
      api.statsTopGenres(10),
    ])
      .then(([o, a, c, g]) => {
        setOverview(o)
        setTopArtists(a)
        setTopCollabs(c)
        setTopGenres(g)
      })
      .catch((e) => setError(e instanceof ApiError ? e.message : 'Erreur inconnue'))
  }, [])

  return (
    <div className="page">
      <div className="page-header">
        <h1>Statistiques</h1>
        <p>Analyse du graphe : artistes les plus connectés, collaborations et genres dominants.</p>
      </div>

      {error && <div className="error-box">{error}</div>}

      {overview && (
        <div className="section">
          <h2>Vue d'ensemble</h2>
          <div className="stat-tile-row">
            <StatTile label="Artistes" value={overview.artistCount} />
            <StatTile label="Morceaux" value={overview.recordingCount} />
            <StatTile label="Releases" value={overview.releaseCount} />
            <StatTile label="Labels" value={overview.labelCount} />
            <StatTile label="Genres" value={overview.genreCount} />
            <StatTile label="Collaborations" value={overview.collaborationCount} />
          </div>
        </div>
      )}

      <div className="section">
        <h2>Top artistes les plus connectés</h2>
        {topArtists ? (
          <BarChart
            color="var(--series-1)"
            rows={topArtists.map((s) => ({ key: s.artist.mbid, label: s.artist.name, value: s.count }))}
          />
        ) : (
          <p className="muted">Chargement…</p>
        )}
      </div>

      <div className="section">
        <h2>Top collaborations</h2>
        {topCollabs ? (
          <BarChart
            color="var(--series-5)"
            rows={topCollabs.map((s) => ({
              key: `${s.artistA.mbid}-${s.artistB.mbid}`,
              label: `${s.artistA.name} × ${s.artistB.name}`,
              value: s.weight,
            }))}
          />
        ) : (
          <p className="muted">Chargement…</p>
        )}
      </div>

      <div className="section">
        <h2>Top genres</h2>
        {topGenres ? (
          <BarChart
            color="var(--series-2)"
            rows={topGenres.map((s) => ({ key: s.genre, label: s.genre, value: s.count }))}
          />
        ) : (
          <p className="muted">Chargement…</p>
        )}
      </div>
    </div>
  )
}
