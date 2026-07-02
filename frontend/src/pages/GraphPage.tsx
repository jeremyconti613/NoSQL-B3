import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { api, ApiError } from '../api/client'
import type { GraphData } from '../api/types'
import { GraphView } from '../components/GraphView'

type Mode = 'full' | 'collaborations'

export function GraphPage() {
  const [mode, setMode] = useState<Mode>('full')
  const [data, setData] = useState<GraphData | null>(null)
  const [error, setError] = useState<string | null>(null)
  const navigate = useNavigate()

  useEffect(() => {
    setData(null)
    setError(null)
    const request = mode === 'full' ? api.graphFull(80) : api.graphCollaborations(300)
    request.then(setData).catch((e) => setError(e instanceof ApiError ? e.message : 'Erreur inconnue'))
  }, [mode])

  return (
    <div className="page">
      <div className="page-header">
        <h1>Graphe des relations</h1>
        <p>
          Visualisation des artistes, morceaux, releases et collaborations. Cliquez sur un artiste
          pour ouvrir sa fiche.
        </p>
      </div>

      <div className="tabs">
        <button className={mode === 'full' ? 'active' : ''} onClick={() => setMode('full')}>
          Vue d'ensemble
        </button>
        <button className={mode === 'collaborations' ? 'active' : ''} onClick={() => setMode('collaborations')}>
          Collaborations uniquement
        </button>
      </div>

      {error && <div className="error-box">{error}</div>}
      {!error && !data && <p className="muted">Chargement du graphe…</p>}
      {data && (
        <GraphView
          data={data}
          height={560}
          onNodeClick={(node) => {
            if (node.type === 'Artist') navigate(`/artists/${encodeURIComponent(node.id)}`)
          }}
        />
      )}
    </div>
  )
}
