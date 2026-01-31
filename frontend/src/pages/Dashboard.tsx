import React from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { 
  Activity, 
  Users, 
  MessageSquare, 
  TrendingUp,
  Zap,
  Shield
} from 'lucide-react'

export const Dashboard: React.FC = () => {
  const stats = [
    {
      title: 'Active Chats',
      value: '12',
      change: '+2 from yesterday',
      icon: MessageSquare,
      color: 'text-neon-blue'
    },
    {
      title: 'Total Users',
      value: '1,234',
      change: '+15% this week',
      icon: Users,
      color: 'text-neon-purple'
    },
    {
      title: 'System Health',
      value: '98%',
      change: 'All systems operational',
      icon: Activity,
      color: 'text-neon-green'
    },
    {
      title: 'API Calls',
      value: '45.2K',
      change: '+8% from last hour',
      icon: TrendingUp,
      color: 'text-neon-pink'
    }
  ]

  const quickActions = [
    {
      title: 'Start New Chat',
      description: 'Begin a conversation with AI',
      icon: MessageSquare,
      href: '/chat',
      variant: 'primary' as const
    },
    {
      title: 'Open Sandbox',
      description: 'Test AI capabilities',
      icon: Zap,
      href: '/sandbox',
      variant: 'secondary' as const
    },
    {
      title: 'Browse Tools',
      description: 'Explore available tools',
      icon: Shield,
      href: '/tools',
      variant: 'secondary' as const
    }
  ]

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-gray-100">Command Center</h1>
        <p className="mt-2 text-gray-400">
          Welcome back! Here's an overview of your MR.DarkPromth system.
        </p>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {stats.map((stat) => {
          const Icon = stat.icon
          return (
            <Card key={stat.title} variant="glass">
              <CardContent className="p-6">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-400">{stat.title}</p>
                    <p className="text-2xl font-bold text-gray-100 mt-1">{stat.value}</p>
                    <p className="text-xs text-gray-500 mt-1">{stat.change}</p>
                  </div>
                  <Icon className={`h-8 w-8 ${stat.color}`} />
                </div>
              </CardContent>
            </Card>
          )
        })}
      </div>

      {/* Quick Actions */}
      <div>
        <h2 className="text-xl font-semibold text-gray-100 mb-4">Quick Actions</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {quickActions.map((action) => {
            const Icon = action.icon
            return (
              <Card key={action.title} className="hover:shadow-neon-purple/20 transition-shadow">
                <CardContent className="p-6">
                  <div className="flex items-center space-x-4">
                    <div className="p-3 bg-neon-purple/10 rounded-lg">
                      <Icon className="h-6 w-6 text-neon-purple" />
                    </div>
                    <div className="flex-1">
                      <h3 className="font-medium text-gray-100">{action.title}</h3>
                      <p className="text-sm text-gray-400 mt-1">{action.description}</p>
                    </div>
                  </div>
                  <div className="mt-4">
                    <Button variant={action.variant} className="w-full">
                      {action.title}
                    </Button>
                  </div>
                </CardContent>
              </Card>
            )
          })}
        </div>
      </div>

      {/* Recent Activity */}
      <Card>
        <CardHeader>
          <CardTitle>Recent Activity</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {[
              { user: 'John Doe', action: 'Started new chat session', time: '2 minutes ago' },
              { user: 'Jane Smith', action: 'Accessed admin panel', time: '5 minutes ago' },
              { user: 'System', action: 'Completed backup', time: '10 minutes ago' },
              { user: 'Alice Johnson', action: 'Updated profile settings', time: '15 minutes ago' }
            ].map((activity, index) => (
              <div key={index} className="flex items-center justify-between py-3 border-b border-dark-accent last:border-0">
                <div>
                  <p className="text-sm font-medium text-gray-100">{activity.user}</p>
                  <p className="text-xs text-gray-400">{activity.action}</p>
                </div>
                <span className="text-xs text-gray-500">{activity.time}</span>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
