import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import { Layout } from '@/components/layout/Layout'
import { ProtectedRoute } from '@/components/ProtectedRoute'
import { Chat } from '@/pages/Chat'
import { Sandbox } from '@/pages/Sandbox'
import { Tools } from '@/pages/Tools'
import { Admin } from '@/pages/Admin'
import { Login } from '@/pages/Login'
import { GitHubLogin } from '@/pages/GitHubLogin'
import { Profile } from '@/pages/Profile'
import { HackerDashboard } from '@/pages/HackerDashboard'

function App() {
  return (
    <Router>
      <Routes>
        <Route path="/login" element={<Login />} />
        <Route path="/login/github" element={<GitHubLogin />} />
        <Route path="/" element={
          <ProtectedRoute>
            <Layout />
          </ProtectedRoute>
        }>
          <Route index element={<HackerDashboard />} />
          <Route path="chat" element={<Chat />} />
          <Route path="sandbox" element={<Sandbox />} />
          <Route path="tools" element={<Tools />} />
          <Route path="admin" element={
            <ProtectedRoute requiredTier="Premium">
              <Admin />
            </ProtectedRoute>
          } />
          <Route path="profile" element={<Profile />} />
        </Route>
      </Routes>
    </Router>
  )
}

export default App
