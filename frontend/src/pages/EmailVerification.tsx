import React, { useState, useEffect } from 'react'
import { useNavigate, useSearchParams } from 'react-router-dom'
import { Mail, CheckCircle, AlertCircle, Loader } from 'lucide-react'
import { apiClient } from '../lib/api'

export const EmailVerification: React.FC = () => {
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()
  const [status, setStatus] = useState<'loading' | 'success' | 'error' | 'idle'>('idle')
  const [message, setMessage] = useState('')
  const [email, setEmail] = useState('')
  const [resendCooldown, setResendCooldown] = useState(0)

  const token = searchParams.get('token')
  const userEmail = searchParams.get('email')

  useEffect(() => {
    if (token && userEmail) {
      verifyEmail(token, userEmail)
    } else if (userEmail) {
      setEmail(userEmail)
    }
  }, [token, userEmail])

  useEffect(() => {
    if (resendCooldown > 0) {
      const timer = setTimeout(() => setResendCooldown(resendCooldown - 1), 1000)
      return () => clearTimeout(timer)
    }
  }, [resendCooldown])

  const verifyEmail = async (verificationToken: string, emailAddress: string) => {
    try {
      setStatus('loading')
      await apiClient.request({
        method: 'POST',
        url: '/api/auth/verify-email',
        data: {
          token: verificationToken,
          email: emailAddress,
        },
      })

      setStatus('success')
      setMessage('Email verified successfully! Redirecting to login...')
      setTimeout(() => navigate('/login'), 2000)
    } catch (error: any) {
      setStatus('error')
      setMessage(error.response?.data?.message || 'Email verification failed. Please try again.')
    }
  }

  const handleResendVerification = async () => {
    if (!email) {
      setMessage('Please enter your email address')
      return
    }

    try {
      setStatus('loading')
      await apiClient.request({
        method: 'POST',
        url: '/api/auth/resend-verification',
        data: { email },
      })

      setStatus('success')
      setMessage('Verification email sent! Check your inbox.')
      setResendCooldown(60)
      setTimeout(() => setStatus('idle'), 3000)
    } catch (error: any) {
      setStatus('error')
      setMessage(error.response?.data?.message || 'Failed to resend verification email')
    }
  }

  const handleSkip = () => {
    // Allow users to skip email verification and continue
    navigate('/login')
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-dark-primary to-dark-secondary flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        <div className="bg-dark-secondary rounded-lg shadow-xl p-8 border border-dark-accent">
          {/* Header */}
          <div className="text-center mb-8">
            <div className="flex justify-center mb-4">
              {status === 'loading' && (
                <Loader className="h-12 w-12 text-neon-purple animate-spin" />
              )}
              {status === 'success' && (
                <CheckCircle className="h-12 w-12 text-green-500" />
              )}
              {status === 'error' && (
                <AlertCircle className="h-12 w-12 text-red-500" />
              )}
              {status === 'idle' && (
                <Mail className="h-12 w-12 text-neon-purple" />
              )}
            </div>
            <h1 className="text-2xl font-bold text-white mb-2">Email Verification</h1>
            <p className="text-gray-400">
              {token ? 'Verifying your email...' : 'Verify your email to access all features'}
            </p>
          </div>

          {/* Status Message */}
          {message && (
            <div
              className={`mb-6 p-4 rounded-lg ${
                status === 'success'
                  ? 'bg-green-500/10 border border-green-500/30 text-green-400'
                  : status === 'error'
                  ? 'bg-red-500/10 border border-red-500/30 text-red-400'
                  : 'bg-blue-500/10 border border-blue-500/30 text-blue-400'
              }`}
            >
              {message}
            </div>
          )}

          {/* Email Input (if not verifying) */}
          {!token && (
            <div className="mb-6">
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Email Address
              </label>
              <input
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder="your@email.com"
                className="w-full px-4 py-2 bg-dark-primary border border-dark-accent rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-neon-purple transition-colors"
              />
            </div>
          )}

          {/* Action Buttons */}
          <div className="space-y-3">
            {!token && status !== 'success' && (
              <>
                <button
                  onClick={handleResendVerification}
                  disabled={resendCooldown > 0 || status === 'loading'}
                  className="w-full px-4 py-2 bg-neon-purple hover:bg-neon-purple/80 disabled:bg-gray-600 text-white font-medium rounded-lg transition-colors disabled:cursor-not-allowed"
                >
                  {resendCooldown > 0
                    ? `Resend in ${resendCooldown}s`
                    : 'Send Verification Email'}
                </button>
                <button
                  onClick={handleSkip}
                  className="w-full px-4 py-2 bg-dark-accent hover:bg-dark-accent/80 text-gray-300 font-medium rounded-lg transition-colors"
                >
                  Skip for Now
                </button>
              </>
            )}

            {status === 'success' && (
              <button
                onClick={() => navigate('/login')}
                className="w-full px-4 py-2 bg-green-500 hover:bg-green-600 text-white font-medium rounded-lg transition-colors"
              >
                Go to Login
              </button>
            )}
          </div>

          {/* Info Text */}
          <div className="mt-6 p-4 bg-dark-primary rounded-lg border border-dark-accent">
            <p className="text-xs text-gray-400 text-center">
              ℹ️ Email verification is optional. You can continue using the platform without verifying your email.
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}

export default EmailVerification
