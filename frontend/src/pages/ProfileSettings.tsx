import React, { useState, useEffect } from 'react'
import { User, Mail, Lock, AlertCircle, CheckCircle, Loader, Eye, EyeOff, LogOut } from 'lucide-react'
import { apiClient } from '../lib/api'
import { useNavigate } from 'react-router-dom'

interface UserProfile {
  id: string
  username: string
  email: string
  tier: 'free' | 'premium' | 'ultra'
  created_at: string
  avatar_url?: string
  bio?: string
}

export const ProfileSettings: React.FC = () => {
  const navigate = useNavigate()
  const [profile, setProfile] = useState<UserProfile | null>(null)
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState('')
  const [success, setSuccess] = useState('')
  const [activeTab, setActiveTab] = useState<'profile' | 'security' | 'preferences'>('profile')

  // Profile form
  const [username, setUsername] = useState('')
  const [bio, setBio] = useState('')
  const [avatarPreview, setAvatarPreview] = useState('')

  // Security form
  const [currentPassword, setCurrentPassword] = useState('')
  const [newPassword, setNewPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [showCurrentPassword, setShowCurrentPassword] = useState(false)
  const [showNewPassword, setShowNewPassword] = useState(false)
  const [showConfirmPassword, setShowConfirmPassword] = useState(false)
  const [passwordStrength, setPasswordStrength] = useState(0)

  // Preferences
  const [emailNotifications, setEmailNotifications] = useState(true)
  const [securityAlerts, setSecurityAlerts] = useState(true)
  const [twoFactorEnabled, setTwoFactorEnabled] = useState(false)

  useEffect(() => {
    fetchProfile()
  }, [])

  const fetchProfile = async () => {
    try {
      setLoading(true)
      const data = await apiClient.getUserProfile()
      setProfile(data)
      setUsername(data.username)
      setBio(data.bio || '')
      setAvatarPreview(data.avatar_url || '')
    } catch (err: any) {
      setError(err.response?.data?.message || 'Failed to load profile')
    } finally {
      setLoading(false)
    }
  }

  const calculatePasswordStrength = (password: string) => {
    let strength = 0
    if (password.length >= 8) strength++
    if (password.length >= 12) strength++
    if (/[a-z]/.test(password) && /[A-Z]/.test(password)) strength++
    if (/\d/.test(password)) strength++
    if (/[!@#$%^&*]/.test(password)) strength++
    setPasswordStrength(strength)
  }

  const handleAvatarChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (file) {
      const reader = new FileReader()
      reader.onloadend = () => {
        setAvatarPreview(reader.result as string)
      }
      reader.readAsDataURL(file)
    }
  }

  const handleUpdateProfile = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setSuccess('')

    try {
      setSaving(true)
      await apiClient.updateUserProfile({
        username,
      })
      setSuccess('Profile updated successfully!')
      setTimeout(() => setSuccess(''), 3000)
    } catch (err: any) {
      setError(err.response?.data?.message || 'Failed to update profile')
    } finally {
      setSaving(false)
    }
  }

  const handleChangePassword = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setSuccess('')

    if (!currentPassword || !newPassword || !confirmPassword) {
      setError('Please fill in all password fields')
      return
    }

    if (newPassword !== confirmPassword) {
      setError('New passwords do not match')
      return
    }

    if (newPassword.length < 8) {
      setError('New password must be at least 8 characters long')
      return
    }

    try {
      setSaving(true)
      await apiClient.request({
        method: 'POST',
        url: '/api/auth/change-password',
        data: {
          current_password: currentPassword,
          new_password: newPassword,
        },
      })
      setSuccess('Password changed successfully!')
      setCurrentPassword('')
      setNewPassword('')
      setConfirmPassword('')
      setTimeout(() => setSuccess(''), 3000)
    } catch (err: any) {
      setError(err.response?.data?.message || 'Failed to change password')
    } finally {
      setSaving(false)
    }
  }

  const handleLogout = async () => {
    try {
      await apiClient.logout()
      localStorage.removeItem('token')
      navigate('/login')
    } catch (err) {
      setError('Failed to logout')
    }
  }

  const getPasswordStrengthColor = () => {
    if (passwordStrength <= 1) return 'bg-red-500'
    if (passwordStrength <= 2) return 'bg-yellow-500'
    if (passwordStrength <= 3) return 'bg-blue-500'
    return 'bg-green-500'
  }

  const getPasswordStrengthText = () => {
    if (passwordStrength <= 1) return 'Weak'
    if (passwordStrength <= 2) return 'Fair'
    if (passwordStrength <= 3) return 'Good'
    return 'Strong'
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-96">
        <Loader className="h-8 w-8 text-neon-purple animate-spin" />
      </div>
    )
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-white">Settings</h1>
        <p className="text-gray-400 mt-1">Manage your account and preferences</p>
      </div>

      {/* Error/Success Messages */}
      {error && (
        <div className="p-4 bg-red-500/10 border border-red-500/30 rounded-lg text-red-400 flex items-start gap-3">
          <AlertCircle className="h-5 w-5 flex-shrink-0 mt-0.5" />
          <span>{error}</span>
        </div>
      )}

      {success && (
        <div className="p-4 bg-green-500/10 border border-green-500/30 rounded-lg text-green-400 flex items-start gap-3">
          <CheckCircle className="h-5 w-5 flex-shrink-0 mt-0.5" />
          <span>{success}</span>
        </div>
      )}

      {/* Tabs */}
      <div className="flex gap-4 border-b border-dark-accent">
        <button
          onClick={() => setActiveTab('profile')}
          className={`px-4 py-3 font-medium transition-colors ${
            activeTab === 'profile'
              ? 'text-neon-purple border-b-2 border-neon-purple'
              : 'text-gray-400 hover:text-gray-300'
          }`}
        >
          <User className="h-5 w-5 inline mr-2" />
          Profile
        </button>
        <button
          onClick={() => setActiveTab('security')}
          className={`px-4 py-3 font-medium transition-colors ${
            activeTab === 'security'
              ? 'text-neon-purple border-b-2 border-neon-purple'
              : 'text-gray-400 hover:text-gray-300'
          }`}
        >
          <Lock className="h-5 w-5 inline mr-2" />
          Security
        </button>
        <button
          onClick={() => setActiveTab('preferences')}
          className={`px-4 py-3 font-medium transition-colors ${
            activeTab === 'preferences'
              ? 'text-neon-purple border-b-2 border-neon-purple'
              : 'text-gray-400 hover:text-gray-300'
          }`}
        >
          <Mail className="h-5 w-5 inline mr-2" />
          Preferences
        </button>
      </div>

      {/* Profile Tab */}
      {activeTab === 'profile' && profile && (
        <div className="bg-dark-secondary rounded-lg border border-dark-accent p-6">
          <form onSubmit={handleUpdateProfile} className="space-y-6">
            {/* Avatar */}
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-3">Avatar</label>
              <div className="flex items-center gap-4">
                <div className="w-16 h-16 rounded-full bg-neon-purple/20 flex items-center justify-center overflow-hidden">
                  {avatarPreview ? (
                    <img src={avatarPreview} alt="Avatar" className="w-full h-full object-cover" />
                  ) : (
                    <User className="h-8 w-8 text-neon-purple" />
                  )}
                </div>
                <input
                  type="file"
                  accept="image/*"
                  onChange={handleAvatarChange}
                  className="flex-1 px-4 py-2 bg-dark-primary border border-dark-accent rounded-lg text-white cursor-pointer"
                />
              </div>
            </div>

            {/* Username */}
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">Username</label>
              <input
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                className="w-full px-4 py-2 bg-dark-primary border border-dark-accent rounded-lg text-white focus:outline-none focus:border-neon-purple transition-colors"
              />
            </div>

            {/* Bio */}
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">Bio</label>
              <textarea
                value={bio}
                onChange={(e) => setBio(e.target.value)}
                rows={4}
                placeholder="Tell us about yourself..."
                className="w-full px-4 py-2 bg-dark-primary border border-dark-accent rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-neon-purple transition-colors resize-none"
              />
            </div>

            {/* Account Info */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">Email</label>
                <input
                  type="email"
                  value={profile.email}
                  disabled
                  className="w-full px-4 py-2 bg-dark-primary border border-dark-accent rounded-lg text-gray-400 cursor-not-allowed"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">Tier</label>
                <input
                  type="text"
                  value={profile.tier.toUpperCase()}
                  disabled
                  className="w-full px-4 py-2 bg-dark-primary border border-dark-accent rounded-lg text-gray-400 cursor-not-allowed"
                />
              </div>
            </div>

            {/* Save Button */}
            <button
              type="submit"
              disabled={saving}
              className="w-full px-4 py-2 bg-neon-purple hover:bg-neon-purple/80 disabled:bg-gray-600 text-white font-medium rounded-lg transition-colors flex items-center justify-center gap-2"
            >
              {saving && <Loader className="h-4 w-4 animate-spin" />}
              {saving ? 'Saving...' : 'Save Changes'}
            </button>
          </form>
        </div>
      )}

      {/* Security Tab */}
      {activeTab === 'security' && (
        <div className="space-y-6">
          {/* Change Password */}
          <div className="bg-dark-secondary rounded-lg border border-dark-accent p-6">
            <h2 className="text-lg font-bold text-white mb-4">Change Password</h2>
            <form onSubmit={handleChangePassword} className="space-y-4">
              {/* Current Password */}
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">Current Password</label>
                <div className="relative">
                  <Lock className="absolute left-3 top-3 h-5 w-5 text-gray-500" />
                  <input
                    type={showCurrentPassword ? 'text' : 'password'}
                    value={currentPassword}
                    onChange={(e) => setCurrentPassword(e.target.value)}
                    className="w-full pl-10 pr-10 py-2 bg-dark-primary border border-dark-accent rounded-lg text-white focus:outline-none focus:border-neon-purple transition-colors"
                  />
                  <button
                    type="button"
                    onClick={() => setShowCurrentPassword(!showCurrentPassword)}
                    className="absolute right-3 top-3 text-gray-500 hover:text-gray-300"
                  >
                    {showCurrentPassword ? (
                      <EyeOff className="h-5 w-5" />
                    ) : (
                      <Eye className="h-5 w-5" />
                    )}
                  </button>
                </div>
              </div>

              {/* New Password */}
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">New Password</label>
                <div className="relative">
                  <Lock className="absolute left-3 top-3 h-5 w-5 text-gray-500" />
                  <input
                    type={showNewPassword ? 'text' : 'password'}
                    value={newPassword}
                    onChange={(e) => {
                      setNewPassword(e.target.value)
                      calculatePasswordStrength(e.target.value)
                    }}
                    className="w-full pl-10 pr-10 py-2 bg-dark-primary border border-dark-accent rounded-lg text-white focus:outline-none focus:border-neon-purple transition-colors"
                  />
                  <button
                    type="button"
                    onClick={() => setShowNewPassword(!showNewPassword)}
                    className="absolute right-3 top-3 text-gray-500 hover:text-gray-300"
                  >
                    {showNewPassword ? (
                      <EyeOff className="h-5 w-5" />
                    ) : (
                      <Eye className="h-5 w-5" />
                    )}
                  </button>
                </div>

                {/* Password Strength */}
                {newPassword && (
                  <div className="mt-2">
                    <div className="flex items-center justify-between mb-1">
                      <span className="text-xs text-gray-400">Strength</span>
                      <span className="text-xs font-medium text-gray-300">{getPasswordStrengthText()}</span>
                    </div>
                    <div className="w-full h-2 bg-dark-primary rounded-full overflow-hidden">
                      <div
                        className={`h-full ${getPasswordStrengthColor()} transition-all duration-300`}
                        style={{ width: `${(passwordStrength / 5) * 100}%` }}
                      />
                    </div>
                  </div>
                )}
              </div>

              {/* Confirm Password */}
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">Confirm Password</label>
                <div className="relative">
                  <Lock className="absolute left-3 top-3 h-5 w-5 text-gray-500" />
                  <input
                    type={showConfirmPassword ? 'text' : 'password'}
                    value={confirmPassword}
                    onChange={(e) => setConfirmPassword(e.target.value)}
                    className="w-full pl-10 pr-10 py-2 bg-dark-primary border border-dark-accent rounded-lg text-white focus:outline-none focus:border-neon-purple transition-colors"
                  />
                  <button
                    type="button"
                    onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                    className="absolute right-3 top-3 text-gray-500 hover:text-gray-300"
                  >
                    {showConfirmPassword ? (
                      <EyeOff className="h-5 w-5" />
                    ) : (
                      <Eye className="h-5 w-5" />
                    )}
                  </button>
                </div>
              </div>

              <button
                type="submit"
                disabled={saving}
                className="w-full px-4 py-2 bg-neon-purple hover:bg-neon-purple/80 disabled:bg-gray-600 text-white font-medium rounded-lg transition-colors flex items-center justify-center gap-2"
              >
                {saving && <Loader className="h-4 w-4 animate-spin" />}
                {saving ? 'Updating...' : 'Update Password'}
              </button>
            </form>
          </div>

          {/* Logout */}
          <div className="bg-dark-secondary rounded-lg border border-dark-accent p-6">
            <h2 className="text-lg font-bold text-white mb-4">Session</h2>
            <button
              onClick={handleLogout}
              className="w-full px-4 py-2 bg-red-500/20 hover:bg-red-500/30 text-red-400 font-medium rounded-lg transition-colors flex items-center justify-center gap-2"
            >
              <LogOut className="h-5 w-5" />
              Logout
            </button>
          </div>
        </div>
      )}

      {/* Preferences Tab */}
      {activeTab === 'preferences' && (
        <div className="bg-dark-secondary rounded-lg border border-dark-accent p-6">
          <h2 className="text-lg font-bold text-white mb-6">Notification Preferences</h2>
          <div className="space-y-4">
            {/* Email Notifications */}
            <div className="flex items-center justify-between p-4 bg-dark-primary rounded-lg">
              <div>
                <p className="font-medium text-white">Email Notifications</p>
                <p className="text-sm text-gray-400">Receive updates about your account</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={emailNotifications}
                  onChange={(e) => setEmailNotifications(e.target.checked)}
                  className="sr-only peer"
                />
                <div className="w-11 h-6 bg-gray-600 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-neon-purple rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-neon-purple"></div>
              </label>
            </div>

            {/* Security Alerts */}
            <div className="flex items-center justify-between p-4 bg-dark-primary rounded-lg">
              <div>
                <p className="font-medium text-white">Security Alerts</p>
                <p className="text-sm text-gray-400">Get notified of suspicious activity</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={securityAlerts}
                  onChange={(e) => setSecurityAlerts(e.target.checked)}
                  className="sr-only peer"
                />
                <div className="w-11 h-6 bg-gray-600 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-neon-purple rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-neon-purple"></div>
              </label>
            </div>

            {/* Two Factor Authentication */}
            <div className="flex items-center justify-between p-4 bg-dark-primary rounded-lg">
              <div>
                <p className="font-medium text-white">Two-Factor Authentication</p>
                <p className="text-sm text-gray-400">Add an extra layer of security</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={twoFactorEnabled}
                  onChange={(e) => setTwoFactorEnabled(e.target.checked)}
                  className="sr-only peer"
                />
                <div className="w-11 h-6 bg-gray-600 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-neon-purple rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-neon-purple"></div>
              </label>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default ProfileSettings
