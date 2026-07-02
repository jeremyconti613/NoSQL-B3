import { Route, Routes } from 'react-router-dom'
import { Layout } from './components/Layout'
import { Home } from './pages/Home'
import { Search } from './pages/Search'
import { Artists } from './pages/Artists'
import { ArtistDetail } from './pages/ArtistDetail'
import { Tracks } from './pages/Tracks'
import { GraphPage } from './pages/GraphPage'
import { Stats } from './pages/Stats'

function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route index element={<Home />} />
        <Route path="search" element={<Search />} />
        <Route path="artists" element={<Artists />} />
        <Route path="artists/:id" element={<ArtistDetail />} />
        <Route path="recordings" element={<Tracks />} />
        <Route path="graph" element={<GraphPage />} />
        <Route path="stats" element={<Stats />} />
      </Route>
    </Routes>
  )
}

export default App
