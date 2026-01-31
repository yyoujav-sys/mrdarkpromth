import React, { useState, useEffect } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { useNavigate, useSearchParams } from 'react-router-dom'
import { useAuthStore } from '@/store/authStore'
import { 
  Github, 
  User, 
  Mail, 
  Shield,
  Zap,
  Crown,
  AlertCircle,
  CheckCircle,
  Loader2
} from 'lucide-react'

interface GitHubProfile {
  github_id: number
  username: string
  name?: string
  email: string
  avatar_url?: string
  bio?: string
  location?: string
  company?: string
  blog?: string
  public_repos: number
  followers: number
  following: number
  tier: string
}

interface GitHubAuthResponse {
  user: {
    id: string
    username: string
    email: string
    tier: string
    avatar_url?: string
    github_username: string
    github_followers: number
    github_repos: number
    created_at: string
  }
  token: string
  expires_in: number
  github_profile: GitHubProfile
}

export const GitHubLogin: React.FC = () => {
  const [searchParams] = useSearchParams()
  const navigate = useNavigate()
  const authStore = useAuthStore()
  const [isGitHubLoading, setIsGitHubLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)

  // Handle GitHub OAuth callback
  useEffect(() => {
    const code = searchParams.get('code')
    const state = searchParams.get('state')
    const error_param = searchParams.get('error')

    if (error_param) {
      setError(`GitHub authentication failed: ${error_param}`)
      return
    }

    if (code && state) {
      handleGitHubCallback(code, state)
    }
  }, [searchParams])

  const handleGitHubCallback = async (code: string, state: string) => {
    setIsGitHubLoading(true)
    setError(null)
    setSuccess(null)

    try {
      const response = await fetch('/api/auth/github/callback', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ code, state }),
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || 'GitHub authentication failed')
      }

      const data: GitHubAuthResponse = await response.json()
      
      // Update auth store with GitHub user data
      await loginWithGitHub(data)
      
      setSuccess(`Successfully logged in as ${data.github_profile.username}!`)
      
      // Redirect to dashboard after 2 seconds
      setTimeout(() => {
        navigate('/')
      }, 2000)
      
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An error occurred during GitHub authentication')
    } finally {
      setIsGitHubLoading(false)
    }
  }

  const loginWithGitHub = async (data: GitHubAuthResponse) => {
    // Store user data in auth store
    localStorage.setItem('token', data.token)
    localStorage.setItem('user', JSON.stringify(data.user))
    localStorage.setItem('github_profile', JSON.stringify(data.github_profile))
    
    // Update auth store state
    useAuthStore.setState({
      user: {
        id: data.user.id,
        username: data.user.username,
        email: data.user.email,
        tier: data.user.tier as 'Free' | 'Premium' | 'Ultra',
        avatar: data.user.avatar_url,
      },
      token: data.token,
      isAuthenticated: true,
      isLoading: false,
    })
  }

  const initiateGitHubLogin = async () => {
    setIsGitHubLoading(true)
    setError(null)

    try {
      const response = await fetch('/api/auth/github/url')
      const data = await response.json()
      
      // Store state in sessionStorage for verification
      sessionStorage.setItem('github_oauth_state', data.state)
      
      // Redirect to GitHub
      window.location.href = data.authorization_url
      
    } catch (err) {
      setError('Failed to initiate GitHub authentication')
      setIsGitHubLoading(false)
    }
  }

  const getTierIcon = (tier: string) => {
    switch (tier.toLowerCase()) {
      case 'ultra':
        return <Crown className="w-5 h-5 text-purple-500" />
      case 'premium':
        return <Shield className="w-5 h-5 text-blue-500" />
      default:
        return <User className="w-5 h-5 text-gray-500" />
    }
  }

  const getTierColor = (tier: string) => {
    switch (tier.toLowerCase()) {
      case 'ultra':
        return 'text-purple-600 bg-purple-50 border-purple-200'
      case 'premium':
        return 'text-blue-600 bg-blue-50 border-blue-200'
      default:
        return 'text-gray-600 bg-gray-50 border-gray-200'
    }
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-900 via-purple-900 to-violet-900 flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        <Card className="bg-white/10 backdrop-blur-lg border-white/20">
          <CardHeader className="text-center">
            <div className="flex justify-center mb-4">
              <div className="w-16 h-16 bg-gradient-to-r from-purple-500 to-pink-500 rounded-full flex items-center justify-center">
                <Github className="w-8 h-8 text-white" />
              </div>
            </div>
            <CardTitle className="text-2xl font-bold text-white">
              GitHub Login
            </CardTitle>
            <p className="text-gray-300">
              Connect your GitHub account for instant access
            </p>
          </CardHeader>
          
          <CardContent className="space-y-6">
            {/* Error Message */}
            {error && (
              <div className="flex items-center gap-2 p-3 bg-red-500/20 border border-red-500/50 rounded-lg">
                <AlertCircle className="w-4 h-4 text-red-400" />
                <span className="text-red-300 text-sm">{error}</span>
              </div>
            )}

            {/* Success Message */}
            {success && (
              <div className="flex items-center gap-2 p-3 bg-green-500/20 border border-green-500/50 rounded-lg">
                <CheckCircle className="w-4 h-4 text-green-400" />
                <span className="text-green-300 text-sm">{success}</span>
              </div>
            )}

            {/* GitHub Login Button */}
            <Button
              onClick={initiateGitHubLogin}
              disabled={isGitHubLoading}
              className="w-full bg-gray-900 hover:bg-gray-800 text-white border border-gray-700"
            >
              {isGitHubLoading ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Connecting to GitHub...
                </>
              ) : (
                <>
                  <Github className="w-4 h-4 mr-2" />
                  Continue with GitHub
                </>
              )}
            </Button>

            {/* Benefits Section */}
            <div className="space-y-3">
              <h3 className="text-white font-semibold text-center">
                Why connect with GitHub?
              </h3>
              
              <div className="space-y-2">
                <div className="flex items-center gap-3 p-2 bg-white/5 rounded-lg">
                  <Zap className="w-4 h-4 text-yellow-400" />
                  <span className="text-gray-300 text-sm">
                    Instant tier based on your GitHub profile
                  </span>
                </div>
                
                <div className="flex items-center gap-3 p-2 bg-white/5 rounded-lg">
                  <Crown className="w-4 h-4 text-purple-400" />
                  <span className="text-gray-300 text-sm">
                    1000+ followers = Ultra Tier access
                  </span>
                </div>
                
                <div className="flex items-center gap-3 p-2 bg-white/5 rounded-lg">
                  <Shield className="w-4 h-4 text-blue-400" />
                  <span className="text-gray-300 text-sm">
                    100+ followers = Premium Tier access
                  </span>
                </div>
                
                <div className="flex items-center gap-3 p-2 bg-white/5 rounded-lg">
                  <Mail className="w-4 h-4 text-green-400" />
                  <span className="text-gray-300 text-sm">
                    No email verification required
                  </span>
                </div>
              </div>
            </div>

            {/* Tier Information */}
            <div className="border-t border-white/20 pt-4">
              <h4 className="text-white font-medium mb-3">Tier Benefits:</h4>
              <div className="space-y-2">
                <div className={`flex items-center justify-between p-2 rounded-lg border ${getTierColor('free')}`}>
                  <div className="flex items-center gap-2">
                    {getTierIcon('free')}
                    <span className="text-sm font-medium">Free</span>
                  </div>
                  <span className="text-xs">Basic chat access</span>
                </div>
                
                <div className={`flex items-center justify-between p-2 rounded-lg border ${getTierColor('premium')}`}>
                  <div className="flex items-center gap-2">
                    {getTierIcon('premium')}
                    <span className="text-sm font-medium">Premium</span>
                  </div>
                  <span className="text-xs">Extended features</span>
                </div>
                
                <div className={`flex items-center justify-between p-2 rounded-lg border ${getTierColor('ultra')}`}>
                  <div className="flex items-center gap-2">
                    {getTierIcon('ultra')}
                    <span className="text-sm font-medium">Ultra</span>
                  </div>
                  <span className="text-xs">Unlimited access</span>
                </div>
              </div>
            </div>

            {/* Back to Login */}
            <div className="text-center">
              <button
                onClick={() => navigate('/login')}
                className="text-gray-400 hover:text-white text-sm underline"
              >
                Back to email login
              </button>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
