# MusicGraph — frontend

React + Vite + TypeScript interface for MusicGraph. See the [repository root README](../README.md)
for the full project overview, architecture, and how to run this alongside the backend/Neo4j.

```bash
cp .env.example .env   # set VITE_API_URL if the backend isn't on localhost:8080
bun install
bun dev                 # http://localhost:5173
```

`bun run build` type-checks (`tsc -b`) and produces a production bundle in `dist/`.
`bun run lint` runs oxlint.

## Structure

```
src/
├── api/         Typed fetch client + response types (mirrors backend/src/models.rs)
├── components/  Layout, ArtistCard, StatTile, BarChart, GraphView (react-force-graph-2d)
├── pages/       Home, Search, Artists, ArtistDetail, Tracks, GraphPage, Stats
└── styles/      Design tokens + shared CSS (see the dataviz-driven palette in index.css)
```
