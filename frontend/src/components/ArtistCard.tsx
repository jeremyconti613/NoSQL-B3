import { Link } from 'react-router-dom'
import type { Artist } from '../api/types'

export function ArtistCard({ artist }: { artist: Artist }) {
  return (
    <Link className="card artist-card" to={`/artists/${encodeURIComponent(artist.mbid)}`}>
      <h3>{artist.name}</h3>
      <span className="meta">
        {[artist.type, artist.country, artist.beginDate].filter(Boolean).join(' · ') || 'Détails limités'}
      </span>
      {artist.disambiguation && <span className="meta">{artist.disambiguation}</span>}
    </Link>
  )
}
