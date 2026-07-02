import { useEffect, useState } from 'react'
import { Link, useNavigate, useParams } from 'react-router-dom'
import { api, ApiError } from '../api/client'
import type { Artist, Collaboration, GraphData, Recording, Release } from '../api/types'
import { GraphView } from '../components/GraphView'

type Tab = 'recordings' | 'releases' | 'collaborations' | 'graph'

export function ArtistDetail() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [artist, setArtist] = useState<Artist | null>(null)
  const [recordings, setRecordings] = useState<Recording[]>([])
  const [releases, setReleases] = useState<Release[]>([])
  const [collaborations, setCollaborations] = useState<Collaboration[]>([])
  const [graph, setGraph] = useState<GraphData | null>(null)
  const [tab, setTab] = useState<Tab>('recordings')
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!id) return
    setError(null)
    setArtist(null)
    Promise.all([
      api.getArtist(id),
      api.getArtistRecordings(id),
      api.getArtistReleases(id),
      api.getArtistCollaborations(id),
    ])
      .then(([a, recs, rels, collabs]) => {
        setArtist(a)
        setRecordings(recs)
        setReleases(rels)
        setCollaborations(collabs)
      })
      .catch((e) => setError(e instanceof ApiError ? e.message : 'Erreur inconnue'))
  }, [id])

  useEffect(() => {
    if (tab !== 'graph' || !id || graph) return
    api.graphForArtist(id).then(setGraph).catch(() => setGraph({ nodes: [], links: [] }))
  }, [tab, id, graph])

  if (error) return <div className="page"><div className="error-box">{error}</div></div>
  if (!artist) return <div className="page"><p className="muted">Chargement…</p></div>

  return (
    <div className="page">
      <div className="page-header">
        <h1>{artist.name}</h1>
        <p>
          {[artist.type, artist.country, artist.beginDate && `depuis ${artist.beginDate}`]
            .filter(Boolean)
            .join(' · ') || 'Détails limités'}
        </p>
        {artist.disambiguation && <p className="muted">{artist.disambiguation}</p>}
      </div>

      <div className="tabs">
        <button className={tab === 'recordings' ? 'active' : ''} onClick={() => setTab('recordings')}>
          Morceaux ({recordings.length})
        </button>
        <button className={tab === 'releases' ? 'active' : ''} onClick={() => setTab('releases')}>
          Releases ({releases.length})
        </button>
        <button
          className={tab === 'collaborations' ? 'active' : ''}
          onClick={() => setTab('collaborations')}
        >
          Collaborations ({collaborations.length})
        </button>
        <button className={tab === 'graph' ? 'active' : ''} onClick={() => setTab('graph')}>
          Graphe
        </button>
      </div>

      {tab === 'recordings' && (
        recordings.length === 0 ? (
          <p className="empty-state">Aucun morceau importé pour cet artiste.</p>
        ) : (
          <table className="data-table">
            <thead>
              <tr>
                <th>Titre</th>
                <th>Durée</th>
                <th>1ère sortie</th>
                <th>Popularité</th>
              </tr>
            </thead>
            <tbody>
              {recordings.map((r) => (
                <tr key={r.mbid}>
                  <td>{r.title}</td>
                  <td>{formatDuration(r.length)}</td>
                  <td>{r.firstReleaseDate ?? '—'}</td>
                  <td>{r.popularity?.toFixed(0) ?? '—'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )
      )}

      {tab === 'releases' && (
        releases.length === 0 ? (
          <p className="empty-state">Aucune release importée pour cet artiste.</p>
        ) : (
          <table className="data-table">
            <thead>
              <tr>
                <th>Titre</th>
                <th>Date</th>
                <th>Pays</th>
                <th>Type</th>
                <th>Statut</th>
              </tr>
            </thead>
            <tbody>
              {releases.map((r) => (
                <tr key={r.mbid}>
                  <td>{r.title}</td>
                  <td>{r.date ?? '—'}</td>
                  <td>{r.country ?? '—'}</td>
                  <td>{r.releaseType ?? '—'}</td>
                  <td>{r.status ?? '—'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )
      )}

      {tab === 'collaborations' && (
        collaborations.length === 0 ? (
          <p className="empty-state">Aucune collaboration détectée pour cet artiste.</p>
        ) : (
          <table className="data-table">
            <thead>
              <tr>
                <th>Artiste</th>
                <th>Morceaux partagés</th>
              </tr>
            </thead>
            <tbody>
              {collaborations.map((c) => (
                <tr key={c.artist.mbid}>
                  <td>
                    <Link to={`/artists/${encodeURIComponent(c.artist.mbid)}`}>{c.artist.name}</Link>
                  </td>
                  <td>{c.weight}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )
      )}

      {tab === 'graph' && (
        graph ? (
          <GraphView
            data={graph}
            height={420}
            onNodeClick={(node) => {
              if (node.type === 'Artist' && node.id !== id) {
                navigate(`/artists/${encodeURIComponent(node.id)}`)
              }
            }}
          />
        ) : (
          <p className="muted">Chargement du graphe…</p>
        )
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
