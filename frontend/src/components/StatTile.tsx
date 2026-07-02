/** Stat tile contract: label (sentence case, no trailing colon) + value
 * (compact, semibold). See dataviz skill § marks-and-anatomy.md. */
export function StatTile({ label, value }: { label: string; value: number }) {
  return (
    <div className="stat-tile">
      <div className="label">{label}</div>
      <div className="value">{formatCompact(value)}</div>
    </div>
  )
}

function formatCompact(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
  return n.toLocaleString('fr-FR')
}
