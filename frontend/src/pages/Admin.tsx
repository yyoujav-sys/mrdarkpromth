import React, { useEffect, useMemo, useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Input } from '@/components/ui/Input'
import { 
  Users, 
  Shield, 
  Activity, 
  Settings,
  Search,
  Ban,
  TrendingUp,
  Eye,
  Edit
} from 'lucide-react'
import apiClient from '@/lib/api'

interface User {
  id: string
  username: string
  email: string
  tier: 'Free' | 'Premium' | 'Ultra'
  status: 'active' | 'suspended' | 'pending'
  joinDate: Date
  lastActive: Date
  usageCount: number
  riskScore: number
}

interface SystemMetric {
  name: string
  value: string | number
  change: string
  status: 'normal' | 'warning' | 'critical'
}

export const Admin: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('')
  const [selectedTier, setSelectedTier] = useState('all')
  const [selectedStatus, setSelectedStatus] = useState('all')

  const [users, setUsers] = useState<User[]>([])
  const [systemMetrics, setSystemMetrics] = useState<SystemMetric[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)

  useEffect(() => {
    let isActive = true

    const loadAdminData = async () => {
      setIsLoading(true)
      setErrorMessage(null)

      try {
        const [usersResponse, metricsResponse] = await Promise.all([
          apiClient.getUsers(),
          apiClient.getSystemMetrics()
        ])

        const userList = Array.isArray(usersResponse.users) ? usersResponse.users : usersResponse.users ?? []

        const mappedUsers: User[] = userList.map((user: any) => {
          const tier = user.tier?.toLowerCase() === 'ultra'
            ? 'Ultra'
            : user.tier?.toLowerCase() === 'premium'
            ? 'Premium'
            : 'Free'
          const status = user.is_active ? 'active' : 'suspended'

          return {
            id: user.id,
            username: user.username,
            email: user.email,
            tier,
            status,
            joinDate: new Date(user.created_at ?? Date.now()),
            lastActive: new Date(user.created_at ?? Date.now()),
            usageCount: 0,
            riskScore: status === 'suspended' ? 80 : 15
          }
        })

        const metrics: SystemMetric[] = [
          {
            name: 'Total Users',
            value: metricsResponse.total_users?.toLocaleString?.() ?? metricsResponse.total_users ?? 0,
            change: 'All time',
            status: 'normal'
          },
          {
            name: 'Active Users',
            value: metricsResponse.active_users?.toLocaleString?.() ?? metricsResponse.active_users ?? 0,
            change: 'Currently active',
            status: 'normal'
          },
          {
            name: 'CPU Usage',
            value: `${metricsResponse.cpu_usage_percent?.toFixed?.(1) ?? metricsResponse.cpu_usage_percent ?? 0}%`,
            change: 'Live metrics',
            status: metricsResponse.cpu_usage_percent > 80 ? 'warning' : 'normal'
          },
          {
            name: 'Memory Usage',
            value: `${metricsResponse.memory_usage_mb ?? 0} MB`,
            change: 'Live metrics',
            status: metricsResponse.memory_usage_mb > 1024 ? 'warning' : 'normal'
          }
        ]

        if (isActive) {
          setUsers(mappedUsers)
          setSystemMetrics(metrics)
        }
      } catch (error) {
        if (isActive) {
          setErrorMessage('Unable to load admin data. Please try again later.')
        }
      } finally {
        if (isActive) {
          setIsLoading(false)
        }
      }
    }

    loadAdminData()

    return () => {
      isActive = false
    }
  }, [])

  const filteredUsers = useMemo(() => {
    return users.filter(user => {
      const matchesSearch = user.username.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         user.email.toLowerCase().includes(searchTerm.toLowerCase())
      const matchesTier = selectedTier === 'all' || user.tier === selectedTier
      const matchesStatus = selectedStatus === 'all' || user.status === selectedStatus
      
      return matchesSearch && matchesTier && matchesStatus
    })
  }, [users, searchTerm, selectedTier, selectedStatus])

  const getTierColor = (tier: User['tier']) => {
    switch (tier) {
      case 'Ultra': return 'text-neon-purple bg-neon-purple/10'
      case 'Premium': return 'text-neon-blue bg-neon-blue/10'
      case 'Free': return 'text-gray-400 bg-gray-400/10'
      default: return 'text-gray-400 bg-gray-400/10'
    }
  }

  const getStatusColor = (status: User['status']) => {
    switch (status) {
      case 'active': return 'text-green-400 bg-green-400/10'
      case 'suspended': return 'text-red-400 bg-red-400/10'
      case 'pending': return 'text-yellow-400 bg-yellow-400/10'
      default: return 'text-gray-400 bg-gray-400/10'
    }
  }

  const getRiskColor = (score: number) => {
    if (score < 30) return 'text-green-400'
    if (score < 70) return 'text-yellow-400'
    return 'text-red-400'
  }

  const getMetricStatusColor = (status: SystemMetric['status']) => {
    switch (status) {
      case 'normal': return 'text-green-400'
      case 'warning': return 'text-yellow-400'
      case 'critical': return 'text-red-400'
      default: return 'text-gray-400'
    }
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-gray-100">Admin Dashboard</h1>
        <p className="mt-2 text-gray-400">
          System administration and user management
        </p>
      </div>

      {errorMessage && (
        <Card>
          <CardContent className="p-4 text-sm text-red-300">
            {errorMessage}
          </CardContent>
        </Card>
      )}

      {/* System Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {isLoading ? (
          <Card className="col-span-full">
            <CardContent className="p-6 text-center text-gray-400">
              Loading metrics...
            </CardContent>
          </Card>
        ) : (
          systemMetrics.map((metric) => (
            <Card key={metric.name} variant="glass">
              <CardContent className="p-6">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-400">{metric.name}</p>
                    <p className="text-2xl font-bold text-gray-100 mt-1">{metric.value}</p>
                    <p className="text-xs text-gray-500 mt-1">{metric.change}</p>
                  </div>
                  <div className="p-2 rounded-lg bg-dark-accent">
                    <Activity className={`h-5 w-5 ${getMetricStatusColor(metric.status)}`} />
                  </div>
                </div>
              </CardContent>
            </Card>
          ))
        )}
      </div>

      {/* User Management */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center space-x-2">
              <Users className="h-5 w-5 text-neon-purple" />
              <span>User Management</span>
            </CardTitle>
            <Button variant="primary" size="sm">
              Add User
            </Button>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Filters */}
          <div className="flex flex-col lg:flex-row gap-4">
            <div className="flex-1">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
                <Input
                  placeholder="Search users..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="pl-10"
                />
              </div>
            </div>
            
            <div className="flex gap-2">
              <select
                value={selectedTier}
                onChange={(e) => setSelectedTier(e.target.value)}
                className="px-3 py-2 bg-dark-secondary border border-dark-secondary rounded-lg text-gray-100 text-sm focus:border-neon-purple focus:outline-none"
              >
                <option value="all">All Tiers</option>
                <option value="Free">Free</option>
                <option value="Premium">Premium</option>
                <option value="Ultra">Ultra</option>
              </select>
              
              <select
                value={selectedStatus}
                onChange={(e) => setSelectedStatus(e.target.value)}
                className="px-3 py-2 bg-dark-secondary border border-dark-secondary rounded-lg text-gray-100 text-sm focus:border-neon-purple focus:outline-none"
              >
                <option value="all">All Status</option>
                <option value="active">Active</option>
                <option value="pending">Pending</option>
                <option value="suspended">Suspended</option>
              </select>
            </div>
          </div>

          {/* Users Table */}
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-dark-accent">
                  <th className="text-left py-3 px-4 text-sm font-medium text-gray-400">User</th>
                  <th className="text-left py-3 px-4 text-sm font-medium text-gray-400">Tier</th>
                  <th className="text-left py-3 px-4 text-sm font-medium text-gray-400">Status</th>
                  <th className="text-left py-3 px-4 text-sm font-medium text-gray-400">Usage</th>
                  <th className="text-left py-3 px-4 text-sm font-medium text-gray-400">Risk Score</th>
                  <th className="text-left py-3 px-4 text-sm font-medium text-gray-400">Last Active</th>
                  <th className="text-left py-3 px-4 text-sm font-medium text-gray-400">Actions</th>
                </tr>
              </thead>
              <tbody>
                {isLoading ? (
                  <tr>
                    <td colSpan={7} className="py-6 text-center text-gray-400">
                      Loading users...
                    </td>
                  </tr>
                ) : (
                  filteredUsers.map((user) => (
                    <tr key={user.id} className="border-b border-dark-accent hover:bg-dark-secondary/50">
                    <td className="py-3 px-4">
                      <div>
                        <p className="font-medium text-gray-100">{user.username}</p>
                        <p className="text-sm text-gray-400">{user.email}</p>
                      </div>
                    </td>
                    <td className="py-3 px-4">
                      <span className={`px-2 py-1 text-xs rounded-full ${getTierColor(user.tier)}`}>
                        {user.tier}
                      </span>
                    </td>
                    <td className="py-3 px-4">
                      <span className={`px-2 py-1 text-xs rounded-full ${getStatusColor(user.status)}`}>
                        {user.status}
                      </span>
                    </td>
                    <td className="py-3 px-4 text-gray-100">{user.usageCount.toLocaleString()}</td>
                    <td className="py-3 px-4">
                      <div className="flex items-center space-x-2">
                        <div className="w-16 bg-dark-accent rounded-full h-2">
                          <div 
                            className={`h-2 rounded-full ${
                              user.riskScore < 30 ? 'bg-green-400' : 
                              user.riskScore < 70 ? 'bg-yellow-400' : 'bg-red-400'
                            }`}
                            style={{ width: `${user.riskScore}%` }}
                          />
                        </div>
                        <span className={`text-sm font-medium ${getRiskColor(user.riskScore)}`}>
                          {user.riskScore}
                        </span>
                      </div>
                    </td>
                    <td className="py-3 px-4 text-sm text-gray-400">
                      {user.lastActive.toLocaleDateString()}
                    </td>
                    <td className="py-3 px-4">
                      <div className="flex items-center space-x-2">
                        <Button variant="ghost" size="sm">
                          <Eye className="h-4 w-4" />
                        </Button>
                        <Button variant="ghost" size="sm">
                          <Edit className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          className="text-red-400 hover:text-red-300"
                          onClick={async () => {
                            const nextStatus = user.status === 'active' ? 'suspended' : 'active'
                            try {
                              await apiClient.updateUserStatus(user.id, nextStatus)
                              setUsers(prev => prev.map(item =>
                                item.id === user.id
                                  ? { ...item, status: nextStatus === 'active' ? 'active' : 'suspended' }
                                  : item
                              ))
                            } catch (error) {
                              setErrorMessage('Failed to update user status.')
                            }
                          }}
                        >
                          <Ban className="h-4 w-4" />
                        </Button>
                      </div>
                    </td>
                  </tr>
                ))
                )}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>

      {/* Quick Actions */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center space-x-4">
              <div className="p-3 bg-neon-purple/10 rounded-lg">
                <Shield className="h-6 w-6 text-neon-purple" />
              </div>
              <div>
                <h3 className="font-medium text-gray-100">Security</h3>
                <p className="text-sm text-gray-400">Review security alerts</p>
              </div>
            </div>
            <Button variant="ghost" className="w-full mt-4">
              View Alerts
            </Button>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center space-x-4">
              <div className="p-3 bg-neon-blue/10 rounded-lg">
                <Settings className="h-6 w-6 text-neon-blue" />
              </div>
              <div>
                <h3 className="font-medium text-gray-100">Settings</h3>
                <p className="text-sm text-gray-400">System configuration</p>
              </div>
            </div>
            <Button variant="ghost" className="w-full mt-4">
              Configure
            </Button>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center space-x-4">
              <div className="p-3 bg-neon-green/10 rounded-lg">
                <TrendingUp className="h-6 w-6 text-neon-green" />
              </div>
              <div>
                <h3 className="font-medium text-gray-100">Analytics</h3>
                <p className="text-sm text-gray-400">View detailed reports</p>
              </div>
            </div>
            <Button variant="ghost" className="w-full mt-4">
              View Reports
            </Button>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
