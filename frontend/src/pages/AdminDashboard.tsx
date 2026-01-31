import React, { useState, useEffect } from 'react'
import { Users, Activity, TrendingUp, AlertCircle, Search, Filter, MoreVertical, Shield, Trash2, Edit2, Eye } from 'lucide-react'
import { apiClient } from '../lib/api'

interface User {
  id: string
  username: string
  email: string
  tier: 'free' | 'premium' | 'ultra'
  status: 'active' | 'inactive' | 'suspended'
  created_at: string
  last_login: string
  jailbreak_attempts: number
}

interface SystemMetrics {
  total_users: number
  active_users: number
  total_requests: number
  api_health: number
  security_events: number
}

export const AdminDashboard: React.FC = () => {
  const [users, setUsers] = useState<User[]>([])
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [searchTerm, setSearchTerm] = useState('')
  const [filterTier, setFilterTier] = useState<string>('all')
  const [filterStatus, setFilterStatus] = useState<string>('all')
  const [selectedUser, setSelectedUser] = useState<User | null>(null)
  const [showUserModal, setShowUserModal] = useState(false)
  const [actionLoading, setActionLoading] = useState(false)

  useEffect(() => {
    fetchData()
    const interval = setInterval(fetchData, 30000) // Refresh every 30 seconds
    return () => clearInterval(interval)
  }, [])

  const fetchData = async () => {
    try {
      setLoading(true)
      const [usersRes, metricsRes] = await Promise.all([
        apiClient.getUsers(),
        apiClient.getSystemMetrics(),
      ])
      setUsers(usersRes.users || [])
      setMetrics(metricsRes)
      setError('')
    } catch (err: any) {
      setError(err.response?.data?.message || 'Failed to load dashboard data')
    } finally {
      setLoading(false)
    }
  }

  const filteredUsers = users.filter((user) => {
    const matchesSearch =
      user.username.toLowerCase().includes(searchTerm.toLowerCase()) ||
      user.email.toLowerCase().includes(searchTerm.toLowerCase())
    const matchesTier = filterTier === 'all' || user.tier === filterTier
    const matchesStatus = filterStatus === 'all' || user.status === filterStatus
    return matchesSearch && matchesTier && matchesStatus
  })

  const handleUpdateUserStatus = async (userId: string, newStatus: string) => {
    try {
      setActionLoading(true)
      await apiClient.updateUserStatus(userId, newStatus)
      setUsers(
        users.map((u) =>
          u.id === userId ? { ...u, status: newStatus as any } : u
        )
      )
      setShowUserModal(false)
    } catch (err: any) {
      setError(err.response?.data?.message || 'Failed to update user status')
    } finally {
      setActionLoading(false)
    }
  }

  const handleSuspendUser = async (userId: string) => {
    if (window.confirm('Are you sure you want to suspend this user?')) {
      await handleUpdateUserStatus(userId, 'suspended')
    }
  }

  const handleDeleteUser = async (userId: string) => {
    if (window.confirm('Are you sure you want to delete this user? This action cannot be undone.')) {
      try {
        setActionLoading(true)
        await apiClient.request({
          method: 'DELETE',
          url: `/api/admin/users/${userId}`,
        })
        setUsers(users.filter((u) => u.id !== userId))
        setShowUserModal(false)
      } catch (err: any) {
        setError(err.response?.data?.message || 'Failed to delete user')
      } finally {
        setActionLoading(false)
      }
    }
  }

  const getTierColor = (tier: string) => {
    switch (tier) {
      case 'ultra':
        return 'bg-purple-500/20 text-purple-400 border border-purple-500/30'
      case 'premium':
        return 'bg-blue-500/20 text-blue-400 border border-blue-500/30'
      default:
        return 'bg-gray-500/20 text-gray-400 border border-gray-500/30'
    }
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active':
        return 'bg-green-500/20 text-green-400 border border-green-500/30'
      case 'suspended':
        return 'bg-red-500/20 text-red-400 border border-red-500/30'
      default:
        return 'bg-gray-500/20 text-gray-400 border border-gray-500/30'
    }
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-white">Admin Dashboard</h1>
          <p className="text-gray-400 mt-1">Manage users and monitor system health</p>
        </div>
        <button
          onClick={fetchData}
          className="px-4 py-2 bg-neon-purple hover:bg-neon-purple/80 text-white rounded-lg transition-colors"
        >
          Refresh
        </button>
      </div>

      {/* Error Alert */}
      {error && (
        <div className="p-4 bg-red-500/10 border border-red-500/30 rounded-lg text-red-400 flex items-start gap-3">
          <AlertCircle className="h-5 w-5 flex-shrink-0 mt-0.5" />
          <span>{error}</span>
        </div>
      )}

      {/* Metrics Grid */}
      {metrics && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
          <MetricCard
            icon={Users}
            label="Total Users"
            value={metrics.total_users}
            color="text-blue-400"
          />
          <MetricCard
            icon={Activity}
            label="Active Users"
            value={metrics.active_users}
            color="text-green-400"
          />
          <MetricCard
            icon={TrendingUp}
            label="Total Requests"
            value={metrics.total_requests}
            color="text-purple-400"
          />
          <MetricCard
            icon={Shield}
            label="API Health"
            value={`${metrics.api_health}%`}
            color="text-yellow-400"
          />
          <MetricCard
            icon={AlertCircle}
            label="Security Events"
            value={metrics.security_events}
            color="text-red-400"
          />
        </div>
      )}

      {/* Users Table */}
      <div className="bg-dark-secondary rounded-lg border border-dark-accent overflow-hidden">
        {/* Table Header */}
        <div className="p-6 border-b border-dark-accent">
          <h2 className="text-xl font-bold text-white mb-4">Users</h2>

          {/* Filters */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {/* Search */}
            <div className="relative">
              <Search className="absolute left-3 top-3 h-5 w-5 text-gray-500" />
              <input
                type="text"
                placeholder="Search by username or email..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="w-full pl-10 pr-4 py-2 bg-dark-primary border border-dark-accent rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-neon-purple transition-colors"
              />
            </div>

            {/* Tier Filter */}
            <select
              value={filterTier}
              onChange={(e) => setFilterTier(e.target.value)}
              className="px-4 py-2 bg-dark-primary border border-dark-accent rounded-lg text-white focus:outline-none focus:border-neon-purple transition-colors"
            >
              <option value="all">All Tiers</option>
              <option value="free">Free</option>
              <option value="premium">Premium</option>
              <option value="ultra">Ultra</option>
            </select>

            {/* Status Filter */}
            <select
              value={filterStatus}
              onChange={(e) => setFilterStatus(e.target.value)}
              className="px-4 py-2 bg-dark-primary border border-dark-accent rounded-lg text-white focus:outline-none focus:border-neon-purple transition-colors"
            >
              <option value="all">All Status</option>
              <option value="active">Active</option>
              <option value="inactive">Inactive</option>
              <option value="suspended">Suspended</option>
            </select>
          </div>
        </div>

        {/* Table Content */}
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-dark-accent bg-dark-primary/50">
                <th className="px-6 py-3 text-left text-sm font-semibold text-gray-300">User</th>
                <th className="px-6 py-3 text-left text-sm font-semibold text-gray-300">Tier</th>
                <th className="px-6 py-3 text-left text-sm font-semibold text-gray-300">Status</th>
                <th className="px-6 py-3 text-left text-sm font-semibold text-gray-300">Last Login</th>
                <th className="px-6 py-3 text-left text-sm font-semibold text-gray-300">Jailbreak Attempts</th>
                <th className="px-6 py-3 text-left text-sm font-semibold text-gray-300">Actions</th>
              </tr>
            </thead>
            <tbody>
              {loading ? (
                <tr>
                  <td colSpan={6} className="px-6 py-8 text-center text-gray-400">
                    Loading users...
                  </td>
                </tr>
              ) : filteredUsers.length === 0 ? (
                <tr>
                  <td colSpan={6} className="px-6 py-8 text-center text-gray-400">
                    No users found
                  </td>
                </tr>
              ) : (
                filteredUsers.map((user) => (
                  <tr key={user.id} className="border-b border-dark-accent hover:bg-dark-primary/50 transition-colors">
                    <td className="px-6 py-4">
                      <div>
                        <p className="font-medium text-white">{user.username}</p>
                        <p className="text-sm text-gray-400">{user.email}</p>
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <span className={`px-3 py-1 rounded-full text-xs font-medium ${getTierColor(user.tier)}`}>
                        {user.tier.toUpperCase()}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      <span className={`px-3 py-1 rounded-full text-xs font-medium ${getStatusColor(user.status)}`}>
                        {user.status.charAt(0).toUpperCase() + user.status.slice(1)}
                      </span>
                    </td>
                    <td className="px-6 py-4 text-sm text-gray-400">
                      {new Date(user.last_login).toLocaleDateString()}
                    </td>
                    <td className="px-6 py-4 text-sm text-gray-400">
                      {user.jailbreak_attempts}
                    </td>
                    <td className="px-6 py-4">
                      <div className="flex items-center gap-2">
                        <button
                          onClick={() => {
                            setSelectedUser(user)
                            setShowUserModal(true)
                          }}
                          className="p-2 hover:bg-dark-accent rounded-lg transition-colors"
                          title="View details"
                        >
                          <Eye className="h-4 w-4 text-gray-400" />
                        </button>
                        <button
                          onClick={() => handleSuspendUser(user.id)}
                          className="p-2 hover:bg-red-500/10 rounded-lg transition-colors"
                          title="Suspend user"
                        >
                          <Shield className="h-4 w-4 text-red-400" />
                        </button>
                        <button
                          onClick={() => handleDeleteUser(user.id)}
                          className="p-2 hover:bg-red-500/10 rounded-lg transition-colors"
                          title="Delete user"
                        >
                          <Trash2 className="h-4 w-4 text-red-400" />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* User Modal */}
      {showUserModal && selectedUser && (
        <UserModal
          user={selectedUser}
          onClose={() => setShowUserModal(false)}
          onUpdateStatus={handleUpdateUserStatus}
          loading={actionLoading}
        />
      )}
    </div>
  )
}

interface MetricCardProps {
  icon: React.ComponentType<{ className?: string }>
  label: string
  value: string | number
  color: string
}

const MetricCard: React.FC<MetricCardProps> = ({ icon: Icon, label, value, color }) => {
  return (
    <div className="bg-dark-secondary rounded-lg border border-dark-accent p-4">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-gray-400 text-sm">{label}</p>
          <p className="text-2xl font-bold text-white mt-1">{value}</p>
        </div>
        <Icon className={`h-8 w-8 ${color}`} />
      </div>
    </div>
  )
}

interface UserModalProps {
  user: User
  onClose: () => void
  onUpdateStatus: (userId: string, status: string) => Promise<void>
  loading: boolean
}

const UserModal: React.FC<UserModalProps> = ({ user, onClose, onUpdateStatus, loading }) => {
  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-dark-secondary rounded-lg border border-dark-accent max-w-md w-full p-6">
        <h2 className="text-xl font-bold text-white mb-4">User Details</h2>

        <div className="space-y-4 mb-6">
          <div>
            <p className="text-gray-400 text-sm">Username</p>
            <p className="text-white font-medium">{user.username}</p>
          </div>
          <div>
            <p className="text-gray-400 text-sm">Email</p>
            <p className="text-white font-medium">{user.email}</p>
          </div>
          <div>
            <p className="text-gray-400 text-sm">Tier</p>
            <p className="text-white font-medium capitalize">{user.tier}</p>
          </div>
          <div>
            <p className="text-gray-400 text-sm">Status</p>
            <p className="text-white font-medium capitalize">{user.status}</p>
          </div>
          <div>
            <p className="text-gray-400 text-sm">Created</p>
            <p className="text-white font-medium">{new Date(user.created_at).toLocaleDateString()}</p>
          </div>
          <div>
            <p className="text-gray-400 text-sm">Jailbreak Attempts</p>
            <p className="text-white font-medium">{user.jailbreak_attempts}</p>
          </div>
        </div>

        <div className="space-y-2">
          <button
            onClick={() => onUpdateStatus(user.id, 'active')}
            disabled={loading}
            className="w-full px-4 py-2 bg-green-500/20 hover:bg-green-500/30 disabled:bg-gray-600 text-green-400 rounded-lg transition-colors"
          >
            Activate
          </button>
          <button
            onClick={() => onUpdateStatus(user.id, 'suspended')}
            disabled={loading}
            className="w-full px-4 py-2 bg-red-500/20 hover:bg-red-500/30 disabled:bg-gray-600 text-red-400 rounded-lg transition-colors"
          >
            Suspend
          </button>
          <button
            onClick={onClose}
            className="w-full px-4 py-2 bg-dark-accent hover:bg-dark-accent/80 text-gray-300 rounded-lg transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  )
}

export default AdminDashboard
