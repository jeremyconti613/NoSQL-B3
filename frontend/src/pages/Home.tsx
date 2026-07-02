import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { api, ApiError } from '../api/client'
import type { OverviewStats } from '../api/types'
import { StatTile } from '../components/StatTile'

export function Home() {
  const [stats, setStats] = useState<OverviewStats | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    api
      .statsOverview()
      .then(setStats)
      .catch((e) => setError(e instanceof ApiError ? e.message : 'Erreur inconnue'))
  }, [])

  return (
    <div className="page">
      <div className="page-header">
        <h1>MusicGraph</h1>
        <p>
          Explorez les artistes, morceaux, albums et collaborations musicales à partir des données
          MusicBrainz, modélisées sous forme de graphe dans Neo4j.
        </p>
      </div>

      <div className="section">
        <h2>Vue d'ensemble</h2>
        {error && <div className="error-box">{error}</div>}
        {!error && !stats && <p className="muted">Chargement…</p>}
        {stats && (
          <div className="stat-tile-row">
            <StatTile label="Artistes" value={stats.artistCount} />
            <StatTile label="Morceaux" value={stats.recordingCount} />
            <StatTile label="Releases" value={stats.releaseCount} />
            <StatTile label="Labels" value={stats.labelCount} />
            <StatTile label="Genres" value={stats.genreCount} />
            <StatTile label="Collaborations" value={stats.collaborationCount} />
          </div>
        )}
      </div>

      <div className="section">
        <h2>Prise en main</h2>
        <p>
          Commencez par <Link to="/search">rechercher un artiste</Link> et l'importer, puis explorez
          ses <Link to="/artists">morceaux et collaborations</Link>, visualisez le{' '}
          <Link to="/graph">graphe des relations</Link>, ou consultez les{' '}
          <Link to="/stats">statistiques</Link>.
        </p>
      </div>
    </div>
  )
}
