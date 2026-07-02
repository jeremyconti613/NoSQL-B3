import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { api, ApiError } from '../api/client'
import type { Artist } from '../api/types'
import { ArtistCard } from '../components/ArtistCard'

export function Artists() {
  const [artists, setArtists] = useState<Artist[] | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    api
      .listArtists({ limit: 200 })
      .then(setArtists)
      .catch((e) => setError(e instanceof ApiError ? e.message : 'Erreur inconnue'))
  }, [])

  return (
    <div className="page">
      <div className="page-header">
        <h1>Artistes importés</h1>
        <p>Les artistes déjà présents dans le graphe Neo4j.</p>
      </div>

      {error && <div className="error-box">{error}</div>}
      {!error && !artists && <p className="muted">Chargement…</p>}
      {artists && artists.length === 0 && (
        <p className="empty-state">
          Aucun artiste importé pour le moment. <Link to="/search">Recherchez-en un</Link> pour commencer.
        </p>
      )}
      {artists && artists.length > 0 && (
        <div className="card-grid">
          {artists.map((a) => (
            <ArtistCard artist={a} key={a.mbid} />
          ))}
        </div>
      )}
    </div>
  )
}
