import { useEffect, useState } from 'react'
import { api, ApiError } from '../api/client'
import type { Recording } from '../api/types'

export function Tracks() {
  const [recordings, setRecordings] = useState<Recording[] | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    api
      .listRecordings({ limit: 200 })
      .then(setRecordings)
      .catch((e) => setError(e instanceof ApiError ? e.message : 'Erreur inconnue'))
  }, [])

  return (
    <div className="page">
      <div className="page-header">
        <h1>Morceaux</h1>
        <p>Les morceaux (recordings) importés, triés par popularité interne.</p>
      </div>

      {error && <div className="error-box">{error}</div>}
      {!error && !recordings && <p className="muted">Chargement…</p>}
      {recordings && recordings.length === 0 && (
        <p className="empty-state">Aucun morceau importé pour le moment.</p>
      )}
      {recordings && recordings.length > 0 && (
        <table className="data-table">
          <thead>
            <tr>
              <th>Titre</th>
              <th>Durée</th>
              <th>1ère sortie</th>
              <th>Popularité</th>
              <th>Source</th>
            </tr>
          </thead>
          <tbody>
            {recordings.map((r) => (
              <tr key={r.mbid}>
                <td>{r.title}</td>
                <td>{formatDuration(r.length)}</td>
                <td>{r.firstReleaseDate ?? '—'}</td>
                <td>{r.popularity?.toFixed(0) ?? '—'}</td>
                <td>{r.source ?? '—'}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  )
}

function formatDuration(ms?: number | null): string {
  if (!ms) return '—'
  const totalSeconds = Math.round(ms / 1000)
  const minutes = Math.floor(totalSeconds / 60)
  const seconds = totalSeconds % 60
  return `${minutes}:${seconds.toString().padStart(2, '0')}`
}
