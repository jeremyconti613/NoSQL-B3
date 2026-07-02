import type {
  Artist,
  ArtistSearchResult,
  ArtistStat,
  Collaboration,
  CollaborationStat,
  GenreStat,
  GraphData,
  OverviewStats,
  Recording,
  Release,
} from './types'

const BASE_URL = (import.meta.env.VITE_API_URL as string | undefined) ?? 'http://localhost:8080/api'

export class ApiError extends Error {
  status: number
  constructor(status: number, message: string) {
    super(message)
    this.status = status
  }
}

function qs(params?: Record<string, string | number | undefined>): string {
  if (!params) return ''
  const entries = Object.entries(params).filter(([, v]) => v !== undefined) as [string, string | number][]
  if (entries.length === 0) return ''
  const search = new URLSearchParams(entries.map(([k, v]) => [k, String(v)]))
  return `?${search.toString()}`
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE_URL}${path}`, {
    headers: { 'Content-Type': 'application/json', ...(init?.headers ?? {}) },
    ...init,
  })

  if (!res.ok) {
    let message = res.statusText
    try {
      const body = (await res.json()) as { error?: string }
      if (body?.error) message = body.error
    } catch {
      // response wasn't JSON — fall back to statusText
    }
    throw new ApiError(res.status, message)
  }

  if (res.status === 204) return undefined as T
  return (await res.json()) as T
}

export const api = {
  listArtists: (params?: { limit?: number; offset?: number }) =>
    request<Artist[]>(`/artists${qs(params)}`),
  getArtist: (id: string) => request<Artist>(`/artists/${encodeURIComponent(id)}`),
  getArtistRecordings: (id: string) =>
    request<Recording[]>(`/artists/${encodeURIComponent(id)}/recordings`),
  getArtistReleases: (id: string) =>
    request<Release[]>(`/artists/${encodeURIComponent(id)}/releases`),
  getArtistCollaborations: (id: string) =>
    request<Collaboration[]>(`/artists/${encodeURIComponent(id)}/collaborations`),

  searchArtists: (q: string, limit = 15) =>
    request<ArtistSearchResult[]>(`/search/artists${qs({ q, limit })}`),
  importArtist: (mbid: string) =>
    request<Artist>(`/import/artists`, { method: 'POST', body: JSON.stringify({ mbid }) }),

  listRecordings: (params?: { limit?: number; offset?: number }) =>
    request<Recording[]>(`/recordings${qs(params)}`),
  getRecording: (id: string) => request<Recording>(`/recordings/${encodeURIComponent(id)}`),
  getRecordingArtists: (id: string) =>
    request<Artist[]>(`/recordings/${encodeURIComponent(id)}/artists`),
  getRecordingReleases: (id: string) =>
    request<Release[]>(`/recordings/${encodeURIComponent(id)}/releases`),

  listReleases: (params?: { limit?: number; offset?: number }) =>
    request<Release[]>(`/releases${qs(params)}`),
  getRelease: (id: string) => request<Release>(`/releases/${encodeURIComponent(id)}`),
  getReleaseRecordings: (id: string) =>
    request<Recording[]>(`/releases/${encodeURIComponent(id)}/recordings`),
  getReleaseArtists: (id: string) =>
    request<Artist[]>(`/releases/${encodeURIComponent(id)}/artists`),

  graphFull: (limit?: number) => request<GraphData>(`/graph${qs({ limit })}`),
  graphForArtist: (id: string) => request<GraphData>(`/graph/artists/${encodeURIComponent(id)}`),
  graphCollaborations: (limit?: number) =>
    request<GraphData>(`/graph/collaborations${qs({ limit })}`),

  statsOverview: () => request<OverviewStats>(`/stats/overview`),
  statsTopCollaborations: (limit = 10) =>
    request<CollaborationStat[]>(`/stats/top-collaborations${qs({ limit })}`),
  statsTopArtists: (limit = 10) => request<ArtistStat[]>(`/stats/top-artists${qs({ limit })}`),
  statsTopGenres: (limit = 10) => request<GenreStat[]>(`/stats/top-genres${qs({ limit })}`),
}
