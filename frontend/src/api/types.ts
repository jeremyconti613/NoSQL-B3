// Mirrors backend/src/models.rs — kept in sync by hand since the backend
// doesn't (yet) generate an OpenAPI schema. Field names match the JSON the
// Rust API actually serializes (see the `#[serde(rename...)]` attributes).

export interface Artist {
  mbid: string
  name: string
  type?: string | null
  country?: string | null
  gender?: string | null
  beginDate?: string | null
  endDate?: string | null
  disambiguation?: string | null
}

export interface ArtistSearchResult {
  mbid: string
  name: string
  country?: string | null
  type?: string | null
  beginDate?: string | null
  score?: number | null
  disambiguation?: string | null
}

export interface Recording {
  mbid: string
  title: string
  length?: number | null
  firstReleaseDate?: string | null
  popularity?: number | null
  source?: string | null
}

export interface Release {
  mbid: string
  title: string
  date?: string | null
  country?: string | null
  status?: string | null
  releaseType?: string | null
  coverArtUrl?: string | null
}

export interface Collaboration {
  artist: Artist
  weight: number
  sharedRecordings: string[]
}

export type GraphNodeType = 'Artist' | 'Recording' | 'Release'

export interface GraphNode {
  id: string
  label: string
  type: GraphNodeType
}

export interface GraphLink {
  source: string
  target: string
  type: string
  weight?: number | null
}

export interface GraphData {
  nodes: GraphNode[]
  links: GraphLink[]
}

export interface OverviewStats {
  artistCount: number
  recordingCount: number
  releaseCount: number
  labelCount: number
  genreCount: number
  areaCount: number
  collaborationCount: number
}

export interface ArtistStat {
  artist: Artist
  count: number
}

export interface CollaborationStat {
  artistA: Artist
  artistB: Artist
  weight: number
}

export interface GenreStat {
  genre: string
  count: number
}
