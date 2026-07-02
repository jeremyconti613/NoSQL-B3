import { NavLink, Outlet } from 'react-router-dom'

const NAV_ITEMS: { to: string; label: string }[] = [
  { to: '/', label: 'Accueil' },
  { to: '/search', label: 'Recherche' },
  { to: '/artists', label: 'Artistes' },
  { to: '/recordings', label: 'Morceaux' },
  { to: '/graph', label: 'Graphe' },
  { to: '/stats', label: 'Statistiques' },
]

export function Layout() {
  return (
    <div className="layout">
      <nav className="top-nav">
        <span className="brand">MusicGraph</span>
        {NAV_ITEMS.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            end={item.to === '/'}
            className={({ isActive }) => (isActive ? 'active' : undefined)}
          >
            {item.label}
          </NavLink>
        ))}
      </nav>
      <Outlet />
    </div>
  )
}
