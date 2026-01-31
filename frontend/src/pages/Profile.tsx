import React, { useEffect, useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Input } from '@/components/ui/Input'
import { 
  User, 
  Mail, 
  Shield, 
  Zap,
  Calendar,
  Activity,
  Settings,
  Save,
  Camera,
  Lock
} from 'lucide-react'
import apiClient from '@/lib/api'
import { useAuthStore } from '@/store/authStore'

interface UserProfile {
  id: string
  username: string
  email: string
  tier: 'Free' | 'Premium' | 'Ultra'
  joinDate: Date
  lastActive: Date
  usageStats: {
    totalRequests: number
    thisMonth: number
    jailbreakUsage: number
  }
  preferences: {
    emailNotifications: boolean
    twoFactorAuth: boolean
    dataSharing: boolean
  }
}

export const Profile: React.FC = () => {
  const [profile, setProfile] = useState<UserProfile | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const { updateUser } = useAuthStore()

  const [isEditing, setIsEditing] = useState(false)
  const [formData, setFormData] = useState({
    username: '',
    email: ''
  })

  useEffect(() => {
    let isActive = true

    const loadProfile = async () => {
      setIsLoading(true)
      setErrorMessage(null)

      try {
        const response = await apiClient.getUserProfile()
        const normalizedTier = response.tier?.toLowerCase()
        const tier: UserProfile['tier'] = normalizedTier === 'ultra'
          ? 'Ultra'
          : normalizedTier === 'premium'
          ? 'Premium'
          : 'Free'

        const profileData: UserProfile = {
          id: response.id,
          username: response.username,
          email: response.email,
          tier,
          joinDate: new Date(response.created_at ?? Date.now()),
          lastActive: new Date(response.created_at ?? Date.now()),
          usageStats: {
            totalRequests: 0,
            thisMonth: 0,
            jailbreakUsage: 0
          },
          preferences: {
            emailNotifications: true,
            twoFactorAuth: false,
            dataSharing: false
          }
        }

        if (isActive) {
          setProfile(profileData)
          setFormData({ username: profileData.username, email: profileData.email })
        }
      } catch (error) {
        if (isActive) {
          setErrorMessage('Unable to load profile. Please try again later.')
        }
      } finally {
        if (isActive) {
          setIsLoading(false)
        }
      }
    }

    loadProfile()

    return () => {
      isActive = false
    }
  }, [])

  const handleSave = async () => {
    if (!profile) return

    try {
      const response = await apiClient.updateUserProfile({
        username: formData.username,
        email: formData.email
      })

      const normalizedTier = response.tier?.toLowerCase()
      const tier: UserProfile['tier'] = normalizedTier === 'ultra'
        ? 'Ultra'
        : normalizedTier === 'premium'
        ? 'Premium'
        : 'Free'

      setProfile(prev => {
        if (!prev) return prev
        return {
          ...prev,
          username: response.username,
          email: response.email,
          tier
        }
      })
      updateUser({ username: response.username, email: response.email, tier })
      setIsEditing(false)
      setErrorMessage(null)
    } catch (error) {
      setErrorMessage('Failed to update profile. Please try again.')
    }
  }

  const handleCancel = () => {
    if (!profile) return
    setFormData({
      username: profile.username,
      email: profile.email
    })
    setIsEditing(false)
  }

  const getTierColor = (tier: UserProfile['tier']) => {
    switch (tier) {
      case 'Ultra': return 'text-neon-purple bg-neon-purple/10 border-neon-purple/30'
      case 'Premium': return 'text-neon-blue bg-neon-blue/10 border-neon-blue/30'
      case 'Free': return 'text-gray-400 bg-gray-400/10 border-gray-400/30'
      default: return 'text-gray-400 bg-gray-400/10 border-gray-400/30'
    }
  }

  const getTierFeatures = (tier: UserProfile['tier']) => {
    switch (tier) {
      case 'Ultra':
        return [
          'Unlimited API requests',
          'Advanced jailbreak prompts',
          'Priority support',
          'Custom model training',
          'Advanced analytics',
          'API access'
        ]
      case 'Premium':
        return [
          '10,000 requests/month',
          'Basic jailbreak access',
          'Email support',
          'Standard analytics',
          'Community access'
        ]
      case 'Free':
        return [
          '100 requests/month',
          'Basic features only',
          'Community support',
          'Limited analytics'
        ]
      default:
        return []
    }
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-gray-100">Profile</h1>
        <p className="mt-2 text-gray-400">
          Manage your account settings and preferences
        </p>
      </div>

      {errorMessage && (
        <Card>
          <CardContent className="p-4 text-sm text-red-300">
            {errorMessage}
          </CardContent>
        </Card>
      )}

      {isLoading ? (
        <Card>
          <CardContent className="p-8 text-center text-gray-400">
            Loading profile...
          </CardContent>
        </Card>
      ) : profile ? (
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Profile Information */}
        <div className="lg:col-span-2 space-y-6">
          {/* Basic Info */}
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="flex items-center space-x-2">
                  <User className="h-5 w-5 text-neon-purple" />
                  <span>Profile Information</span>
                </CardTitle>
                <Button
                  variant={isEditing ? 'ghost' : 'primary'}
                  size="sm"
                  onClick={isEditing ? handleCancel : () => setIsEditing(true)}
                >
                  {isEditing ? 'Cancel' : 'Edit'}
                </Button>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* Avatar */}
              <div className="flex items-center space-x-4">
                <div className="relative">
                  <div className="w-20 h-20 bg-neon-purple/20 rounded-full flex items-center justify-center">
                    <User className="h-10 w-10 text-neon-purple" />
                  </div>
                  <button className="absolute bottom-0 right-0 p-1 bg-neon-purple rounded-full text-white hover:bg-neon-purple/80">
                    <Camera className="h-3 w-3" />
                  </button>
                </div>
                <div>
                  <div className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium border ${getTierColor(profile.tier)}`}>
                    <Zap className="h-3 w-3 mr-1" />
                    {profile.tier} Tier
                  </div>
                </div>
              </div>

              {/* Form Fields */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-200 mb-2">
                    Username
                  </label>
                  {isEditing ? (
                    <Input
                      value={formData.username}
                      onChange={(e) => setFormData(prev => ({ ...prev, username: e.target.value }))}
                      placeholder="Enter username"
                    />
                  ) : (
                    <div className="p-3 bg-dark-secondary rounded-lg text-gray-100">
                      {profile.username}
                    </div>
                  )}
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-200 mb-2">
                    Email
                  </label>
                  {isEditing ? (
                    <Input
                      type="email"
                      value={formData.email}
                      onChange={(e) => setFormData(prev => ({ ...prev, email: e.target.value }))}
                      placeholder="Enter email"
                    />
                  ) : (
                    <div className="p-3 bg-dark-secondary rounded-lg text-gray-100">
                      {profile.email}
                    </div>
                  )}
                </div>
              </div>

              {isEditing && (
                <div className="flex space-x-3">
                  <Button onClick={handleSave} variant="primary">
                    <Save className="h-4 w-4 mr-2" />
                    Save Changes
                  </Button>
                  <Button onClick={handleCancel} variant="ghost">
                    Cancel
                  </Button>
                </div>
              )}
            </CardContent>
          </Card>

          {/* Usage Statistics */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Activity className="h-5 w-5 text-neon-purple" />
                <span>Usage Statistics</span>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div className="text-center">
                  <p className="text-2xl font-bold text-gray-100">
                    {profile.usageStats.totalRequests.toLocaleString()}
                  </p>
                  <p className="text-sm text-gray-400">Total Requests</p>
                </div>
                <div className="text-center">
                  <p className="text-2xl font-bold text-gray-100">
                    {profile.usageStats.thisMonth.toLocaleString()}
                  </p>
                  <p className="text-sm text-gray-400">This Month</p>
                </div>
                <div className="text-center">
                  <p className="text-2xl font-bold text-neon-purple">
                    {profile.usageStats.jailbreakUsage.toLocaleString()}
                  </p>
                  <p className="text-sm text-gray-400">Jailbreak Usage</p>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Preferences */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Settings className="h-5 w-5 text-neon-purple" />
                <span>Preferences</span>
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              {[
                { key: 'emailNotifications', label: 'Email Notifications', description: 'Receive email updates about your account' },
                { key: 'twoFactorAuth', label: 'Two-Factor Authentication', description: 'Add an extra layer of security to your account' },
                { key: 'dataSharing', label: 'Data Sharing', description: 'Share anonymous usage data to improve the service' }
              ].map(({ key, label, description }) => (
                <div key={key} className="flex items-center justify-between py-3 border-b border-dark-accent last:border-0">
                  <div>
                    <p className="font-medium text-gray-100">{label}</p>
                    <p className="text-sm text-gray-400">{description}</p>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input
                      type="checkbox"
                      checked={profile.preferences[key as keyof typeof profile.preferences]}
                      onChange={(e) => setProfile(prev => {
                        if (!prev) return prev
                        return {
                          ...prev,
                          preferences: {
                            ...prev.preferences,
                            [key]: e.target.checked
                          }
                        }
                      })}
                      className="sr-only peer"
                    />
                    <div className="w-11 h-6 bg-gray-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-neon-purple"></div>
                  </label>
                </div>
              ))}
            </CardContent>
          </Card>
        </div>

        {/* Sidebar */}
        <div className="space-y-6">
          {/* Account Info */}
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">Account Info</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center space-x-3">
                <Calendar className="h-4 w-4 text-gray-400" />
                <div>
                  <p className="text-sm text-gray-400">Member Since</p>
                  <p className="text-gray-100">{profile.joinDate.toLocaleDateString()}</p>
                </div>
              </div>
              <div className="flex items-center space-x-3">
                <Activity className="h-4 w-4 text-gray-400" />
                <div>
                  <p className="text-sm text-gray-400">Last Active</p>
                  <p className="text-gray-100">{profile.lastActive.toLocaleDateString()}</p>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Tier Features */}
          <Card>
            <CardHeader>
              <CardTitle className="text-lg flex items-center space-x-2">
                <Shield className="h-5 w-5 text-neon-purple" />
                <span>{profile.tier} Features</span>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <ul className="space-y-2">
                {getTierFeatures(profile.tier).map((feature, index) => (
                  <li key={index} className="flex items-center space-x-2">
                    <div className="w-2 h-2 bg-neon-purple rounded-full" />
                    <span className="text-sm text-gray-300">{feature}</span>
                  </li>
                ))}
              </ul>
              {profile.tier !== 'Ultra' && (
                <Button variant="neon" className="w-full mt-4">
                  Upgrade to Ultra
                </Button>
              )}
            </CardContent>
          </Card>

          {/* Quick Actions */}
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">Quick Actions</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <Button variant="ghost" className="w-full justify-start">
                <Mail className="h-4 w-4 mr-2" />
                Change Email
              </Button>
              <Button variant="ghost" className="w-full justify-start">
                <Lock className="h-4 w-4 mr-2" />
                Change Password
              </Button>
              <Button variant="ghost" className="w-full justify-start text-red-400 hover:text-red-300">
                <Shield className="h-4 w-4 mr-2" />
                Delete Account
              </Button>
            </CardContent>
          </Card>
        </div>
      </div>
      ) : (
        <Card>
          <CardContent className="p-8 text-center text-gray-400">
            No profile data available.
          </CardContent>
        </Card>
      )}
    </div>
  )
}
