import React, { useState, useEffect } from 'react'
import { Activity, TrendingUp, AlertCircle, Search, Shield, Trash2, Eye, CheckCircle, XCircle, CreditCard, Users, FileCheck, Crown, Clock } from 'lucide-react'
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

interface Verification {
  id: string
  payment_id: string
  user_id: string
  slip_image_path: string
  status: string
  submitted_at: string
  reviewed_by?: string
  reviewed_at?: string
  review_notes?: string
}

interface Payment {
  id: string
  amount: number
  status: string
  payment_method: string
  reference: string
  created_at: string
  verified_at?: string
}

interface Subscription {
  id: string
  tier: string
  status: string
  start_date: string
  end_date: string
  auto_renew: boolean
}

interface SystemMetrics {
  cpu_usage: number
  memory_usage: number
}

interface RequestMetrics {
  total: number
  successful: number
  failed: number
  avg_latency_ms: number
  requests_per_second: number
  error_rate: number
}

interface LearningMetrics {
  total_corrections: number
  successful_corrections: number
  failed_corrections: number
  average_confidence: number
  average_validation_score: number
  most_common_errors: string[]
}

interface DashboardSummary {
  agent_id: string
  status: string
  system: SystemMetrics
  requests: RequestMetrics
  telemetry?: any
  learning: LearningMetrics
  timestamp: string
}

type TabType = 'overview' | 'users' | 'verifications' | 'subscriptions'

export const AdminDashboard: React.FC = () => {
  const [activeTab, setActiveTab] = useState<TabType>('overview')
  const [users, setUsers] = useState<User[]>([])
  const [verifications, setVerifications] = useState<Verification[]>([])
  const [dashboardData, setDashboardData] = useState<DashboardSummary | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [searchTerm, setSearchTerm] = useState('')
  const [filterTier, setFilterTier] = useState<string>('all')
  const [filterStatus, setFilterStatus] = useState<string>('all')
  const [selectedUser, setSelectedUser] = useState<User | null>(null)
  const [showUserModal, setShowUserModal] = useState(false)
  const [actionLoading, setActionLoading] = useState(false)
  const [userSubscription, setUserSubscription] = useState<Subscription | null>(null)
  const [userPayments, setUserPayments] = useState<Payment[]>([])
  const [selectedVerification, setSelectedVerification] = useState<Verification | null>(null)
  const [showVerificationModal, setShowVerificationModal] = useState(false)
  const [verificationNotes, setVerificationNotes] = useState('')

  useEffect(() => {
    fetchData()
    const interval = setInterval(() => {
      if (activeTab === 'overview') fetchDashboardData()
    }, 5000)
    return () => clearInterval(interval)
  }, [activeTab])

  useEffect(() => {
    if (activeTab === 'users') fetchUsers()
    if (activeTab === 'verifications') fetchVerifications()
  }, [activeTab])

  const fetchData = async () => {
    await Promise.all([fetchUsers(), fetchDashboardData()])
  }

  const fetchUsers = async () => {
    try {
      const response = await apiClient.getUsers()
      setUsers(response.users || [])
    } catch (err: any) {
      console.error('Failed to fetch users:', err)
    }
  }

  const fetchDashboardData = async () => {
    try {
      if (!dashboardData) setLoading(true)
      const response = await apiClient.getDashboardSummary()
      setDashboardData(response)
      setError('')
    } catch (err: any) {
      console.error('Failed to fetch dashboard:', err)
      if (!dashboardData) setError(err.response?.data?.message || 'Failed to load dashboard')
    } finally {
      setLoading(false)
    }
  }

  const fetchVerifications = async () => {
    try {
      setLoading(true)
      const response = await apiClient.getPendingVerifications()
      setVerifications(response.verifications || [])
    } catch (err: any) {
      console.error('Failed to fetch verifications:', err)
      setError(err.response?.data?.message || 'Failed to load verifications')
    } finally {
      setLoading(false)
    }
  }

  const fetchUserDetails = async (userId: string) => {
    try {
      const [subRes, paymentsRes] = await Promise.all([
        apiClient.getUserSubscription(userId),
        apiClient.getUserPayments(userId)
      ])
      setUserSubscription(subRes.subscription)
      setUserPayments(paymentsRes.payments || [])
    } catch (err: any) {
      console.error('Failed to fetch user details:', err)
    }
  }

  const filteredUsers = users.filter((user) => {
    const matchesSearch = user.username.toLowerCase().includes(searchTerm.toLowerCase()) || user.email.toLowerCase().includes(searchTerm.toLowerCase())
    const matchesTier = filterTier === 'all' || user.tier === filterTier
    const matchesStatus = filterStatus === 'all' || user.status === filterStatus
    return matchesSearch && matchesTier && matchesStatus
  })

  const handleUpdateUserStatus = async (userId: string, newStatus: string) => {
    try {
      setActionLoading(true)
      await apiClient.updateUserStatus(userId, newStatus)
      setUsers(users.map((u) => u.id === userId ? { ...u, status: newStatus as any } : u))
      setShowUserModal(false)
    } catch (err: any) {
      setError(err.response?.data?.message || 'Failed to update user status')
    } finally {
      setActionLoading(false)
    }
  }

  const handleUpdateUserTier = async (userId: string, newTier: string) => {
    try {
      setActionLoading(true)
      await apiClient.updateUserTier(userId, newTier)
      setUsers(users.map((u) => u.id === userId ? { ...u, tier: newTier as any } : u))
      await fetchUserDetails(userId)
    } catch (err: any) {
      setError(err.response?.data?.message || 'Failed to update user tier')
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
        await apiClient.request({ method: 'DELETE', url: `/api/admin/users/${userId}` })
        setUsers(users.filter((u) => u.id !== userId))
        setShowUserModal(false)
      } catch (err: any) {
        setError(err.response?.data?.message || 'Failed to delete user')
      } finally {
        setActionLoading(false)
      }
    }
  }

  const handleApproveVerification = async (verificationId: string, approved: boolean) => {
    try {
      setActionLoading(true)
      await apiClient.approveVerification(verificationId, approved, verificationNotes)
      setVerifications(verifications.filter((v) => v.id !== verificationId))
      setShowVerificationModal(false)
      setVerificationNotes('')
    } catch (err: any) {
      setError(err.response?.data?.message || 'Failed to process verification')
    } finally {
      setActionLoading(false)
    }
  }

  const openUserModal = async (user: User) => {
    setSelectedUser(user)
    setShowUserModal(true)
    await fetchUserDetails(user.id)
  }

  const openVerificationModal = (verification: Verification) => {
    setSelectedVerification(verification)
    setShowVerificationModal(true)
    setVerificationNotes('')
  }

  const getTierColor = (tier: string) => {
    switch (tier) {
      case 'ultra': return 'bg-purple-500/20 text-purple-400 border border-purple-500/30'
      case 'premium': return 'bg-blue-500/20 text-blue-400 border border-blue-500/30'
      default: return 'bg-gray-500/20 text-gray-400 border border-gray-500/30'
    }
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': case 'verified': case 'approved': return 'bg-green-500/20 text-green-400 border border-green-500/30'
      case 'suspended': case 'rejected': return 'bg-red-500/20 text-red-400 border border-red-500/30'
      case 'pending': return 'bg-yellow-500/20 text-yellow-400 border border-yellow-500/30'
      default: return 'bg-gray-500/20 text-gray-400 border border-gray-500/30'
    }
  }

  const renderOverviewTab = () => (
    <>
      {dashboardData && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <MetricCard icon={Activity} label="CPU Usage" value={`${dashboardData.system.cpu_usage.toFixed(1)}%`} color="text-blue-400" />
          <MetricCard icon={TrendingUp} label="Requests / Sec" value={dashboardData.requests.requests_per_second.toFixed(2)} color="text-green-400" />
          <MetricCard icon={TrendingUp} label="Avg Latency" value={`${dashboardData.requests.avg_latency_ms.toFixed(0)} ms`} color="text-purple-400" />
          <MetricCard icon={AlertCircle} label="Error Rate" value={`${dashboardData.requests.error_rate.toFixed(2)}%`} color={dashboardData.requests.error_rate > 5 ? "text-red-400" : "text-green-400"} />
          <MetricCard icon={Shield} label="Self-Corrections" value={dashboardData.learning.total_corrections} color="text-yellow-400" />
          <MetricCard icon={Shield} label="Fix Success Rate" value={`${(dashboardData.learning.successful_corrections / (dashboardData.learning.total_corrections || 1) * 100).toFixed(1)}%`} color="text-neon-purple" />
          <MetricCard icon={Users} label="Total Users" value={users.length} color="text-cyan-400" />
          <MetricCard icon={FileCheck} label="Pending Verifications" value={verifications.length} color="text-orange-400" />
        </div>
      )}
    </>
  )

  const renderUsersTab = () => (
    <div className="bg-dark-secondary rounded-lg border border-dark-accent overflow-hidden">
      <div className="p-6 border-b border-dark-accent">
        <h2 className="text-xl font-bold text-white mb-4">User Management</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="relative">
            <Search className="absolute left-3 top-3 h-5 w-5 text-gray-500" />
            <input type="text" placeholder="Search by username or email..." value={searchTerm} onChange={(e) => setSearchTerm(e.target.value)} className="w-full pl-10 pr-4 py-2 bg-dark-primary border border-dark-accent rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-neon-purple transition-colors" />
          </div>
          <select value={filterTier} onChange={(e) => setFilterTier(e.target.value)} className="px-4 py-2 bg-dark-primary border border-dark-accent rounded-lg text-white focus:outline-none focus:border-neon-purple transition-colors">
            <option value="all">All Tiers</option>
            <option value="free">Free</option>
            <option value="premium">Premium</option>
            <option value="ultra">Ultra</option>
          </select>
          <select value={filterStatus} onChange={(e) => setFilterStatus(e.target.value)} className="px-4 py-2 bg-dark-primary border border-dark-accent rounded-lg text-white focus:outline-none focus:border-neon-purple transition-colors">
            <option value="all">All Status</option>
            <option value="active">Active</option>
            <option value="inactive">Inactive</option>
            <option value="suspended">Suspended</option>
          </select>
        </div>
      </div>
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
              <tr><td colSpan={6} className="px-6 py-8 text-center text-gray-400">Loading users...</td></tr>
            ) : filteredUsers.length === 0 ? (
              <tr><td colSpan={6} className="px-6 py-8 text-center text-gray-400">No users found</td></tr>
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
                    <span className={`px-3 py-1 rounded-full text-xs font-medium ${getTierColor(user.tier)}`}>{user.tier.toUpperCase()}</span>
                  </td>
                  <td className="px-6 py-4">
                    <span className={`px-3 py-1 rounded-full text-xs font-medium ${getStatusColor(user.status)}`}>{user.status.charAt(0).toUpperCase() + user.status.slice(1)}</span>
                  </td>
                  <td className="px-6 py-4 text-sm text-gray-400">{new Date(user.last_login).toLocaleDateString()}</td>
                  <td className="px-6 py-4 text-sm text-gray-400">{user.jailbreak_attempts}</td>
                  <td className="px-6 py-4">
                    <div className="flex items-center gap-2">
                      <button onClick={() => openUserModal(user)} className="p-2 hover:bg-dark-accent rounded-lg transition-colors" title="View details"><Eye className="h-4 w-4 text-gray-400" /></button>
                      <button onClick={() => handleSuspendUser(user.id)} className="p-2 hover:bg-red-500/10 rounded-lg transition-colors" title="Suspend user"><Shield className="h-4 w-4 text-red-400" /></button>
                      <button onClick={() => handleDeleteUser(user.id)} className="p-2 hover:bg-red-500/10 rounded-lg transition-colors" title="Delete user"><Trash2 className="h-4 w-4 text-red-400" /></button>
                    </div>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  )

  const renderVerificationsTab = () => (
    <div className="bg-dark-secondary rounded-lg border border-dark-accent overflow-hidden">
      <div className="p-6 border-b border-dark-accent flex justify-between items-center">
        <div>
          <h2 className="text-xl font-bold text-white">Verify Slip Management</h2>
          <p className="text-gray-400 text-sm mt-1">Review and approve payment slip verifications</p>
        </div>
        <button onClick={fetchVerifications} className="px-4 py-2 bg-neon-purple hover:bg-neon-purple/80 text-white rounded-lg transition-colors flex items-center gap-2">
          <Clock className="h-4 w-4" /> Refresh
        </button>
      </div>
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead>
            <tr className="border-b border-dark-accent bg-dark-primary/50">
              <th className="px-6 py-3 text-left text-sm font-semibold text-gray-300">Verification ID</th>
              <th className="px-6 py-3 text-left text-sm font-semibold text-gray-300">Payment ID</th>
              <th className="px-6 py-3 text-left text-sm font-semibold text-gray-300">User ID</th>
              <th className="px-6 py-3 text-left text-sm font-semibold text-gray-300">Submitted At</th>
              <th className="px-6 py-3 text-left text-sm font-semibold text-gray-300">Status</th>
              <th className="px-6 py-3 text-left text-sm font-semibold text-gray-300">Actions</th>
            </tr>
          </thead>
          <tbody>
            {loading ? (
              <tr><td colSpan={6} className="px-6 py-8 text-center text-gray-400">Loading verifications...</td></tr>
            ) : verifications.length === 0 ? (
              <tr><td colSpan={6} className="px-6 py-8 text-center text-gray-400">No pending verifications</td></tr>
            ) : (
              verifications.map((v) => (
                <tr key={v.id} className="border-b border-dark-accent hover:bg-dark-primary/50 transition-colors">
                  <td className="px-6 py-4 text-sm text-gray-300 font-mono">{v.id.slice(0, 8)}...</td>
                  <td className="px-6 py-4 text-sm text-gray-300 font-mono">{v.payment_id.slice(0, 8)}...</td>
                  <td className="px-6 py-4 text-sm text-gray-300 font-mono">{v.user_id.slice(0, 8)}...</td>
                  <td className="px-6 py-4 text-sm text-gray-400">{new Date(v.submitted_at).toLocaleString()}</td>
                  <td className="px-6 py-4">
                    <span className={`px-3 py-1 rounded-full text-xs font-medium ${getStatusColor(v.status)}`}>{v.status.charAt(0).toUpperCase() + v.status.slice(1)}</span>
                  </td>
                  <td className="px-6 py-4">
                    <div className="flex items-center gap-2">
                      <button onClick={() => openVerificationModal(v)} className="p-2 hover:bg-neon-purple/20 rounded-lg transition-colors" title="Review"><Eye className="h-4 w-4 text-neon-purple" /></button>
                    </div>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  )

  const renderSubscriptionsTab = () => (
    <div className="bg-dark-secondary rounded-lg border border-dark-accent p-6">
      <h2 className="text-xl font-bold text-white mb-4">Subscription Overview</h2>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-dark-primary rounded-lg p-4 border border-dark-accent">
          <div className="flex items-center gap-3 mb-2">
            <Users className="h-5 w-5 text-gray-400" />
            <span className="text-gray-400">Free Users</span>
          </div>
          <p className="text-2xl font-bold text-white">{users.filter(u => u.tier === 'free').length}</p>
        </div>
        <div className="bg-dark-primary rounded-lg p-4 border border-dark-accent">
          <div className="flex items-center gap-3 mb-2">
            <Crown className="h-5 w-5 text-blue-400" />
            <span className="text-gray-400">Premium Users</span>
          </div>
          <p className="text-2xl font-bold text-blue-400">{users.filter(u => u.tier === 'premium').length}</p>
        </div>
        <div className="bg-dark-primary rounded-lg p-4 border border-dark-accent">
          <div className="flex items-center gap-3 mb-2">
            <Crown className="h-5 w-5 text-purple-400" />
            <span className="text-gray-400">Ultra Users</span>
          </div>
          <p className="text-2xl font-bold text-purple-400">{users.filter(u => u.tier === 'ultra').length}</p>
        </div>
      </div>
    </div>
  )

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-white">Admin Dashboard</h1>
          <p className="text-gray-400 mt-1">Manage users, verify payments, and monitor system health</p>
        </div>
      </div>

      {error && (
        <div className="p-4 bg-red-500/10 border border-red-500/30 rounded-lg text-red-400 flex items-start gap-3">
          <AlertCircle className="h-5 w-5 flex-shrink-0 mt-0.5" />
          <span>{error}</span>
        </div>
      )}

      <div className="border-b border-dark-accent">
        <nav className="flex gap-1">
          {[
            { id: 'overview', label: 'Overview', icon: Activity },
            { id: 'users', label: 'Users', icon: Users },
            { id: 'verifications', label: 'Verify Slips', icon: FileCheck },
            { id: 'subscriptions', label: 'Subscriptions', icon: CreditCard }
          ].map((tab) => (
            <button key={tab.id} onClick={() => setActiveTab(tab.id as TabType)} className={`flex items-center gap-2 px-4 py-3 text-sm font-medium transition-colors border-b-2 ${activeTab === tab.id ? 'text-neon-purple border-neon-purple' : 'text-gray-400 border-transparent hover:text-white'}`}>
              <tab.icon className="h-4 w-4" /> {tab.label}
            </button>
          ))}
        </nav>
      </div>

      {activeTab === 'overview' && renderOverviewTab()}
      {activeTab === 'users' && renderUsersTab()}
      {activeTab === 'verifications' && renderVerificationsTab()}
      {activeTab === 'subscriptions' && renderSubscriptionsTab()}

      {showUserModal && selectedUser && (
        <UserModal user={selectedUser} subscription={userSubscription} payments={userPayments} onClose={() => setShowUserModal(false)} onUpdateStatus={handleUpdateUserStatus} onUpdateTier={handleUpdateUserTier} loading={actionLoading} />
      )}

      {showVerificationModal && selectedVerification && (
        <VerificationModal verification={selectedVerification} onClose={() => setShowVerificationModal(false)} onApprove={(id) => handleApproveVerification(id, true)} onReject={(id) => handleApproveVerification(id, false)} notes={verificationNotes} setNotes={setVerificationNotes} loading={actionLoading} />
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

const MetricCard: React.FC<MetricCardProps> = ({ icon: Icon, label, value, color }) => (
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

interface UserModalProps {
  user: User
  subscription: Subscription | null
  payments: Payment[]
  onClose: () => void
  onUpdateStatus: (userId: string, status: string) => Promise<void>
  onUpdateTier: (userId: string, tier: string) => Promise<void>
  loading: boolean
}

const UserModal: React.FC<UserModalProps> = ({ user, subscription, payments, onClose, onUpdateStatus, onUpdateTier, loading }) => {
  const getTierColor = (tier: string) => {
    switch (tier) {
      case 'ultra': return 'text-purple-400'
      case 'premium': return 'text-blue-400'
      default: return 'text-gray-400'
    }
  }

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-dark-secondary rounded-lg border border-dark-accent max-w-2xl w-full max-h-[90vh] overflow-y-auto p-6">
        <h2 className="text-xl font-bold text-white mb-4">User Details</h2>
        <div className="space-y-4 mb-6">
          <div className="grid grid-cols-2 gap-4">
            <div><p className="text-gray-400 text-sm">Username</p><p className="text-white font-medium">{user.username}</p></div>
            <div><p className="text-gray-400 text-sm">Email</p><p className="text-white font-medium">{user.email}</p></div>
            <div><p className="text-gray-400 text-sm">Tier</p><p className={`font-medium capitalize ${getTierColor(user.tier)}`}>{user.tier}</p></div>
            <div><p className="text-gray-400 text-sm">Status</p><p className="text-white font-medium capitalize">{user.status}</p></div>
            <div><p className="text-gray-400 text-sm">Created</p><p className="text-white font-medium">{new Date(user.created_at).toLocaleDateString()}</p></div>
            <div><p className="text-gray-400 text-sm">Jailbreak Attempts</p><p className="text-white font-medium">{user.jailbreak_attempts}</p></div>
          </div>

          {subscription && (
            <div className="border-t border-dark-accent pt-4">
              <h3 className="text-lg font-semibold text-white mb-2">Subscription</h3>
              <div className="grid grid-cols-2 gap-4">
                <div><p className="text-gray-400 text-sm">Plan</p><p className={`font-medium capitalize ${getTierColor(subscription.tier)}`}>{subscription.tier}</p></div>
                <div><p className="text-gray-400 text-sm">Status</p><p className="text-white font-medium capitalize">{subscription.status}</p></div>
                <div><p className="text-gray-400 text-sm">Start Date</p><p className="text-white font-medium">{new Date(subscription.start_date).toLocaleDateString()}</p></div>
                <div><p className="text-gray-400 text-sm">End Date</p><p className="text-white font-medium">{new Date(subscription.end_date).toLocaleDateString()}</p></div>
              </div>
            </div>
          )}

          {payments.length > 0 && (
            <div className="border-t border-dark-accent pt-4">
              <h3 className="text-lg font-semibold text-white mb-2">Recent Payments</h3>
              <div className="space-y-2 max-h-40 overflow-y-auto">
                {payments.slice(0, 5).map((payment) => (
                  <div key={payment.id} className="bg-dark-primary rounded p-2 text-sm">
                    <div className="flex justify-between">
                      <span className="text-gray-400">{payment.reference}</span>
                      <span className={`font-medium ${payment.status === 'verified' ? 'text-green-400' : payment.status === 'pending' ? 'text-yellow-400' : 'text-red-400'}`}>{payment.status}</span>
                    </div>
                    <div className="flex justify-between mt-1">
                      <span className="text-white">{payment.amount} THB</span>
                      <span className="text-gray-500">{new Date(payment.created_at).toLocaleDateString()}</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          <div className="border-t border-dark-accent pt-4 space-y-2">
            <p className="text-gray-400 text-sm">Change Tier</p>
            <div className="flex gap-2">
              {['free', 'premium', 'ultra'].map((tier) => (
                <button key={tier} onClick={() => onUpdateTier(user.id, tier)} disabled={loading || user.tier === tier} className={`px-3 py-1 rounded text-sm capitalize transition-colors ${user.tier === tier ? 'bg-neon-purple text-white' : 'bg-dark-accent text-gray-300 hover:bg-neon-purple/20'}`}>
                  {tier}
                </button>
              ))}
            </div>
          </div>
        </div>

        <div className="space-y-2">
          <button onClick={() => onUpdateStatus(user.id, 'active')} disabled={loading || user.status === 'active'} className="w-full px-4 py-2 bg-green-500/20 hover:bg-green-500/30 disabled:bg-gray-600 text-green-400 rounded-lg transition-colors">Activate</button>
          <button onClick={() => onUpdateStatus(user.id, 'suspended')} disabled={loading || user.status === 'suspended'} className="w-full px-4 py-2 bg-red-500/20 hover:bg-red-500/30 disabled:bg-gray-600 text-red-400 rounded-lg transition-colors">Suspend</button>
          <button onClick={onClose} className="w-full px-4 py-2 bg-dark-accent hover:bg-dark-accent/80 text-gray-300 rounded-lg transition-colors">Close</button>
        </div>
      </div>
    </div>
  )
}

interface VerificationModalProps {
  verification: Verification
  onClose: () => void
  onApprove: (id: string) => void
  onReject: (id: string) => void
  notes: string
  setNotes: (notes: string) => void
  loading: boolean
}

const VerificationModal: React.FC<VerificationModalProps> = ({ verification, onClose, onApprove, onReject, notes, setNotes, loading }) => (
  <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
    <div className="bg-dark-secondary rounded-lg border border-dark-accent max-w-md w-full p-6">
      <h2 className="text-xl font-bold text-white mb-4">Review Payment Slip</h2>
      <div className="space-y-4 mb-6">
        <div className="bg-dark-primary rounded-lg p-4 border border-dark-accent">
          <div className="space-y-2 text-sm">
            <div className="flex justify-between"><span className="text-gray-400">Verification ID:</span><span className="text-white font-mono">{verification.id.slice(0, 16)}...</span></div>
            <div className="flex justify-between"><span className="text-gray-400">Payment ID:</span><span className="text-white font-mono">{verification.payment_id.slice(0, 16)}...</span></div>
            <div className="flex justify-between"><span className="text-gray-400">User ID:</span><span className="text-white font-mono">{verification.user_id.slice(0, 16)}...</span></div>
            <div className="flex justify-between"><span className="text-gray-400">Submitted:</span><span className="text-white">{new Date(verification.submitted_at).toLocaleString()}</span></div>
          </div>
        </div>
        <div>
          <label className="block text-gray-400 text-sm mb-2">Review Notes (optional)</label>
          <textarea value={notes} onChange={(e) => setNotes(e.target.value)} placeholder="Add notes about this verification..." className="w-full px-3 py-2 bg-dark-primary border border-dark-accent rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-neon-purple transition-colors" rows={3} />
        </div>
      </div>
      <div className="space-y-2">
        <button onClick={() => onApprove(verification.id)} disabled={loading} className="w-full px-4 py-2 bg-green-500/20 hover:bg-green-500/30 disabled:bg-gray-600 text-green-400 rounded-lg transition-colors flex items-center justify-center gap-2">
          <CheckCircle className="h-4 w-4" /> {loading ? 'Processing...' : 'Approve & Upgrade Tier'}
        </button>
        <button onClick={() => onReject(verification.id)} disabled={loading} className="w-full px-4 py-2 bg-red-500/20 hover:bg-red-500/30 disabled:bg-gray-600 text-red-400 rounded-lg transition-colors flex items-center justify-center gap-2">
          <XCircle className="h-4 w-4" /> {loading ? 'Processing...' : 'Reject'}
        </button>
        <button onClick={onClose} disabled={loading} className="w-full px-4 py-2 bg-dark-accent hover:bg-dark-accent/80 text-gray-300 rounded-lg transition-colors">Cancel</button>
      </div>
    </div>
  </div>
)

export default AdminDashboard
