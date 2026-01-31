import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import apiClient from '@/lib/api'

interface User {
  id: string
  username: string
  email: string
  tier: 'Free' | 'Premium' | 'Ultra'
  avatar?: string
}

interface AuthState {
  user: User | null
  token: string | null
  isAuthenticated: boolean
  isLoading: boolean
  login: (email: string, password: string) => Promise<void>
  logout: () => void
  updateUser: (user: Partial<User>) => void
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      token: null,
      isAuthenticated: false,
      isLoading: false,

      login: async (email: string, password: string) => {
        set({ isLoading: true })

        try {
          const response = await apiClient.login(email, password)
          const rawTier = response.user?.tier ?? 'Free'
          const normalizedTier = rawTier.toLowerCase()
          const tier: User['tier'] = normalizedTier === 'ultra'
            ? 'Ultra'
            : normalizedTier === 'premium'
            ? 'Premium'
            : 'Free'

          const user: User = {
            id: response.user.id,
            username: response.user.username,
            email: response.user.email,
            tier
          }

          set({
            user,
            token: response.token,
            isAuthenticated: true,
            isLoading: false
          })

          localStorage.setItem('token', response.token)
        } catch (error) {
          set({ isLoading: false })
          throw error
        }
      },

      logout: () => {
        apiClient.logout().catch(() => undefined)
        set({
          user: null,
          token: null,
          isAuthenticated: false
        })
        localStorage.removeItem('token')
      },

      updateUser: (userData: Partial<User>) => {
        const currentUser = get().user
        if (currentUser) {
          set({
            user: { ...currentUser, ...userData }
          })
        }
      }
    }),
    {
      name: 'auth-storage',
      partialize: (state) => ({
        user: state.user,
        token: state.token,
        isAuthenticated: state.isAuthenticated
      })
    }
  )
)
