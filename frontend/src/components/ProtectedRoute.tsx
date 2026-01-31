import React from 'react'
import { Navigate, useLocation } from 'react-router-dom'
import { useAuthStore } from '@/store/authStore'

interface ProtectedRouteProps {
  children: React.ReactNode
  requiredTier?: 'Free' | 'Premium' | 'Ultra'
}

export const ProtectedRoute: React.FC<ProtectedRouteProps> = ({ 
  children, 
  requiredTier 
}) => {
  const { isAuthenticated, user } = useAuthStore()
  const location = useLocation()

  // Check if user is authenticated
  if (!isAuthenticated) {
    return <Navigate to="/login" state={{ from: location }} replace />
  }

  // Check tier requirements
  if (requiredTier && user) {
    const tierLevels = { 'Free': 0, 'Premium': 1, 'Ultra': 2 }
    const userLevel = tierLevels[user.tier]
    const requiredLevel = tierLevels[requiredTier]

    if (userLevel < requiredLevel) {
      return (
        <div className="min-h-screen bg-dark-primary flex items-center justify-center">
          <div className="text-center">
            <div className="mb-6">
              <div className="w-16 h-16 bg-neon-purple/20 rounded-full flex items-center justify-center mx-auto">
                <svg className="w-8 h-8 text-neon-purple" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
              </div>
            </div>
            <h1 className="text-2xl font-bold text-gray-100 mb-2">
              Access Restricted
            </h1>
            <p className="text-gray-400 mb-6">
              This feature requires {requiredTier} tier access. Your current tier is {user.tier}.
            </p>
            <button
              onClick={() => window.history.back()}
              className="px-6 py-2 bg-neon-purple text-white rounded-lg hover:bg-neon-purple/80 transition-colors"
            >
              Go Back
            </button>
          </div>
        </div>
      )
    }
  }

  return <>{children}</>
}
