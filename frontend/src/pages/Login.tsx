import React, { useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Input } from '@/components/ui/Input'
import { useLocation, useNavigate } from 'react-router-dom'
import { useAuthStore } from '@/store/authStore'
import { 
  User, 
  Lock, 
  Eye, 
  EyeOff,
  Zap,
  Shield,
  Bot,
  Github
} from 'lucide-react'

export const Login: React.FC = () => {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [rememberMe, setRememberMe] = useState(false)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const navigate = useNavigate()
  const location = useLocation()
  const { login, isLoading } = useAuthStore()

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault()
    setErrorMessage(null)

    try {
      await login(email, password)
      const redirectTo = (location.state as { from?: { pathname?: string } })?.from?.pathname || '/'
      navigate(redirectTo)
    } catch (error) {
      setErrorMessage('Invalid credentials or server error. Please try again.')
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center px-4" style={{background: 'var(--bg-primary)'}}>
      <div className="w-full max-w-md">
        {/* Logo and Title */}
        <div className="text-center mb-8">
          <div className="flex items-center justify-center space-x-3 mb-4">
            <div className="p-3 rounded-full pulse" style={{background: 'rgba(255, 0, 110, 0.2)'}}>
              <Bot className="h-8 w-8" style={{color: 'var(--accent-primary)'}} />
            </div>
            <h1 className="text-3xl font-bold" style={{background: 'var(--gradient-primary)', WebkitBackgroundClip: 'text', WebkitTextFillColor: 'transparent'}}>MR.DarkPromth</h1>
          </div>
          <p className="text-gray-400">
            Advanced AI platform with ultra-tier capabilities
          </p>
        </div>

        {/* Login Form */}
        <Card variant="glass" className="neon-border">
          <CardHeader>
            <CardTitle className="text-center text-xl">Sign In</CardTitle>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleLogin} className="space-y-6">
              {errorMessage && (
                <div className="rounded-lg border border-red-500/40 bg-red-500/10 px-4 py-2 text-sm text-red-300">
                  {errorMessage}
                </div>
              )}
              {/* Email */}
              <div>
                <Input
                  type="email"
                  placeholder="Email address"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  required
                  icon={<User className="h-4 w-4" />}
                />
              </div>

              {/* Password */}
              <div>
                <div className="relative">
                  <Input
                    type={showPassword ? 'text' : 'password'}
                    placeholder="Password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    required
                    icon={<Lock className="h-4 w-4" />}
                  />
                  <button
                    type="button"
                    onClick={() => setShowPassword(!showPassword)}
                    className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-gray-200"
                  >
                    {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                  </button>
                </div>
              </div>

              {/* Remember Me & Forgot Password */}
              <div className="flex items-center justify-between">
                <label className="flex items-center space-x-2">
                  <input
                    type="checkbox"
                    checked={rememberMe}
                    onChange={(e) => setRememberMe(e.target.checked)}
                    className="rounded border-gray-600 bg-gray-800 text-purple-600 focus:ring-purple-500"
                  />
                  <span className="text-sm text-gray-400">Remember me</span>
                </label>
                <button
                  type="button"
                  className="text-sm text-purple-400 hover:text-purple-300"
                >
                  Forgot password?
                </button>
              </div>

              {/* Submit Button */}
              <Button
                type="submit"
                variant="primary"
                className="w-full"
                disabled={isLoading}
              >
                {isLoading ? (
                  <div className="flex items-center justify-center space-x-2">
                    <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                    <span>Signing in...</span>
                  </div>
                ) : (
                  'Sign In'
                )}
              </Button>
            </form>

            {/* Demo Account */}
            <div className="mt-6 p-4 bg-purple-900/20 border border-purple-600/30 rounded-lg">
              <div className="flex items-center space-x-2 mb-2">
                <Zap className="h-4 w-4 text-purple-400" />
                <span className="text-sm font-medium text-purple-400">Demo Account</span>
              </div>
              <p className="text-xs text-gray-400 mb-2">
                Use any email and password to access the demo
              </p>
              <Button
                variant="neon"
                size="sm"
                className="w-full"
                onClick={() => {
                  setEmail('demo@mrdarkpromth.com')
                  setPassword('demo123')
                }}
              >
                Fill Demo Credentials
              </Button>
            </div>
          </CardContent>
        </Card>

        {/* Security Notice */}
        <div className="mt-6 text-center">
          <div className="flex items-center justify-center space-x-2 text-xs text-gray-500">
            <Shield className="h-3 w-3" />
            <span>Secured with enterprise-grade encryption</span>
          </div>
        </div>

        {/* GitHub Login Button */}
        <div className="mt-4">
          <Button
            className="w-full border-gray-600 hover:bg-gray-800 text-gray-300"
            onClick={() => navigate('/login/github')}
          >
            <Github className="w-4 h-4 mr-2" />
            Continue with GitHub
          </Button>
        </div>

        {/* Sign Up Link */}
        <div className="mt-8 text-center">
          <p className="text-sm text-gray-400">
            Don't have an account?{' '}
            <button 
              onClick={() => navigate('/register')}
              className="text-purple-400 hover:text-purple-300 font-medium"
            >
              Sign up for access
            </button>
          </p>
        </div>
      </div>
    </div>
  )
}
