import { useEffect, useMemo, useRef, useState } from 'react'
import ForceGraph2D from 'react-force-graph-2d'
import type { GraphData, GraphNode, GraphNodeType } from '../api/types'

const NODE_TYPE_ORDER: GraphNodeType[] = ['Artist', 'Recording', 'Release']
// Fixed categorical slots (1, 2, 3) — never reassigned by rank, per the
// dataviz skill's color formula.
const NODE_TYPE_SERIES: Record<GraphNodeType, string> = {
  Artist: '--series-1',
  Recording: '--series-2',
  Release: '--series-3',
}

function useCssVar(name: string, fallback: string): string {
  const [value, setValue] = useState(fallback)
  useEffect(() => {
    const resolved = getComputedStyle(document.documentElement).getPropertyValue(name).trim()
    if (resolved) setValue(resolved)
  }, [name])
  return value
}

function useNodeColors(): Record<GraphNodeType, string> {
  const artist = useCssVar(NODE_TYPE_SERIES.Artist, '#2a78d6')
  const recording = useCssVar(NODE_TYPE_SERIES.Recording, '#1baf7a')
  const release = useCssVar(NODE_TYPE_SERIES.Release, '#eda100')
  return useMemo(() => ({ Artist: artist, Recording: recording, Release: release }), [artist, recording, release])
}

export function GraphView({
  data,
  height = 520,
  onNodeClick,
}: {
  data: GraphData
  height?: number
  onNodeClick?: (node: GraphNode) => void
}) {
  const containerRef = useRef<HTMLDivElement>(null)
  const [width, setWidth] = useState(800)
  const colors = useNodeColors()
  const linkColor = useCssVar('--baseline', '#c3c2b7')

  useEffect(() => {
    const el = containerRef.current
    if (!el) return
    const observer = new ResizeObserver((entries) => {
      const entry = entries[0]
      if (entry) setWidth(entry.contentRect.width)
    })
    observer.observe(el)
    return () => observer.disconnect()
  }, [])

  const graphData = useMemo(
    () => ({
      nodes: data.nodes.map((n) => ({ ...n })),
      links: data.links.map((l) => ({ ...l })),
    }),
    [data],
  )

  const presentTypes = useMemo(
    () => NODE_TYPE_ORDER.filter((t) => data.nodes.some((n) => n.type === t)),
    [data.nodes],
  )

  if (data.nodes.length === 0) {
    return <p className="muted">Aucune donnée de graphe pour le moment — importez des artistes.</p>
  }

  return (
    <div>
      <div className="graph-legend">
        {presentTypes.map((type) => (
          <span className="swatch" key={type}>
            <span className="dot" style={{ background: colors[type] }} />
            {labelForType(type)}
          </span>
        ))}
      </div>
      <div className="graph-container" ref={containerRef}>
        <ForceGraph2D
          width={width}
          height={height}
          graphData={graphData}
          nodeId="id"
          nodeLabel={(n) => `${(n as GraphNode).label} (${labelForType((n as GraphNode).type)})`}
          nodeColor={(n) => colors[(n as GraphNode).type] ?? '#999'}
          nodeRelSize={5}
          linkColor={() => linkColor}
          linkWidth={(l) => Math.min(1 + Math.log1p(Number(l.weight ?? 1)), 6)}
          onNodeClick={(n) => onNodeClick?.(n as GraphNode)}
          cooldownTicks={100}
        />
      </div>
    </div>
  )
}

function labelForType(type: GraphNodeType): string {
  switch (type) {
    case 'Artist':
      return 'Artiste'
    case 'Recording':
      return 'Morceau'
    case 'Release':
      return 'Release'
    default:
      return type
  }
}
