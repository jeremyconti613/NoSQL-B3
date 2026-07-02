import type { ReactNode } from 'react'

export interface BarChartRow {
  key: string
  label: ReactNode
  value: number
}

/** Horizontal bar chart, single series per chart — the chart's own title
 * names the series, so no legend box is needed (dataviz skill: "a single
 * series needs no legend"). Value is direct-labeled at the bar's tip. */
export function BarChart({
  rows,
  color = 'var(--series-1)',
  emptyLabel = 'Aucune donnée pour le moment.',
}: {
  rows: BarChartRow[]
  color?: string
  emptyLabel?: string
}) {
  if (rows.length === 0) {
    return <p className="muted">{emptyLabel}</p>
  }

  const max = Math.max(...rows.map((r) => r.value), 1)

  return (
    <div className="bar-chart">
      {rows.map((row) => (
        <div className="bar-chart-row" key={row.key}>
          <span className="row-label">{row.label}</span>
          <div className="bar-chart-track">
            <div
              className="bar-chart-fill"
              style={{ width: `${Math.max((row.value / max) * 100, 4)}%`, background: color }}
            />
          </div>
          <span className="row-value">{row.value}</span>
        </div>
      ))}
    </div>
  )
}
