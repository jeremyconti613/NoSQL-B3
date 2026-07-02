import { type FormEvent, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { api, ApiError } from '../api/client'
import type { ArtistSearchResult } from '../api/types'

export function Search() {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<ArtistSearchResult[] | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [importingMbid, setImportingMbid] = useState<string | null>(null)
  const [importedMbids, setImportedMbids] = useState<Set<string>>(new Set())
  const navigate = useNavigate()

  async function handleSearch(e: FormEvent) {
    e.preventDefault()
    if (!query.trim()) return
    setLoading(true)
    setError(null)
    try {
      const found = await api.searchArtists(query.trim())
      setResults(found)
    } catch (e) {
      setError(e instanceof ApiError ? e.message : 'La recherche a échoué.')
    } finally {
      setLoading(false)
    }
  }

  async function handleImport(mbid: string) {
    setImportingMbid(mbid)
    setError(null)
    try {
      await api.importArtist(mbid)
      setImportedMbids((prev) => new Set(prev).add(mbid))
    } catch (e) {
      setError(e instanceof ApiError ? e.message : "L'import a échoué.")
    } finally {
      setImportingMbid(null)
    }
  }

  return (
    <div className="page">
      <div className="page-header">
        <h1>Recherche d'artistes</h1>
        <p>
          Recherchez un artiste dans MusicBrainz (ex. Daft Punk, Beyoncé, Stromae, PNL…) puis
          importez-le dans le graphe.
        </p>
      </div>

      <form className="search-form" onSubmit={handleSearch}>
        <input
          type="search"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Nom d'artiste…"
        />
        <button className="btn" type="submit" disabled={loading}>
          {loading ? 'Recherche…' : 'Rechercher'}
        </button>
      </form>

      {error && <div className="error-box">{error}</div>}

      {results && results.length === 0 && !loading && (
        <p className="empty-state">Aucun résultat MusicBrainz pour cette recherche.</p>
      )}

      {results && results.length > 0 && (
        <table className="data-table">
          <thead>
            <tr>
              <th>Nom</th>
              <th>Type</th>
              <th>Pays</th>
              <th>Début</th>
              <th>Score</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {results.map((r) => {
              const imported = importedMbids.has(r.mbid)
              return (
                <tr key={r.mbid}>
                  <td>
                    <strong>{r.name}</strong>
                    {r.disambiguation && <div className="muted">{r.disambiguation}</div>}
                  </td>
                  <td>{r.type ?? '—'}</td>
                  <td>{r.country ?? '—'}</td>
                  <td>{r.beginDate ?? '—'}</td>
                  <td>{r.score ?? '—'}</td>
                  <td>
                    {imported ? (
                      <button className="btn secondary" onClick={() => navigate(`/artists/${encodeURIComponent(r.mbid)}`)}>
                        Voir la fiche
                      </button>
                    ) : (
                      <button
                        className="btn"
                        disabled={importingMbid === r.mbid}
                        onClick={() => handleImport(r.mbid)}
                      >
                        {importingMbid === r.mbid ? 'Import…' : 'Importer'}
                      </button>
                    )}
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
      )}
    </div>
  )
}
