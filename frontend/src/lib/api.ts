import axios from 'axios'
import type { AxiosInstance, AxiosRequestConfig } from 'axios'

class ApiClient {
  private client: AxiosInstance

  constructor() {
    this.client = axios.create({
      baseURL: import.meta.env.VITE_API_BASE_URL || 'https://bt-shop-dark.online',
      timeout: 15000,
      headers: {
        'Content-Type': 'application/json',
      },
    })

    // Request interceptor to add auth token
    this.client.interceptors.request.use(
      (config) => {
        const token = localStorage.getItem('token')
        if (token) {
          config.headers.Authorization = `Bearer ${token}`
        }
        return config
      },
      (error) => {
        return Promise.reject(error)
      }
    )

    // Response interceptor for error handling
    this.client.interceptors.response.use(
      (response) => response,
      (error) => {
        const status = error.response?.status
        
        if (status === 401) {
          // Token expired or invalid
          localStorage.removeItem('token')
          if (window.location.pathname !== '/login') {
            window.location.href = '/login'
          }
        } else if (status === 403) {
          console.error('Access forbidden: You do not have permission for this action.')
        } else if (status === 429) {
          console.error('Too many requests: Please try again later.')
        } else if (status >= 500) {
          console.error('Server error: Something went wrong on our end.')
        }
        
        return Promise.reject(error)
      }
    )
  }

  // Auth endpoints
  async login(email: string, password: string) {
    const response = await this.client.post('/api/auth/login', {
      email,
      password,
    })
    return response.data
  }

  async register(userData: {
    username: string
    email: string
    password: string
  }) {
    const response = await this.client.post('/api/auth/register', userData)
    return response.data
  }

  async logout() {
    const response = await this.client.post('/api/auth/logout')
    return response.data
  }

  // GitHub Auth
  async getGitHubAuthUrl() {
    const response = await this.client.get('/api/auth/github/url')
    return response.data
  }

  async githubCallback(code: string) {
    const response = await this.client.post('/api/auth/github/callback', { code })
    return response.data
  }

  // User endpoints
  async getUserProfile() {
    const response = await this.client.get('/api/auth/me')
    return response.data
  }

  async updateUserProfile(userData: Partial<{
    username: string
    email: string
  }>) {
    const response = await this.client.put('/api/auth/me', userData)
    return response.data
  }

  async getUserStats() {
    const response = await this.client.get('/api/users/me/stats')
    return response.data
  }

  async getUserPreferences() {
    const response = await this.client.get('/api/users/me/preferences')
    return response.data
  }

  async updateUserPreferences(preferences: {
    emailNotifications: boolean
    twoFactorAuth: boolean
    dataSharing: boolean
  }) {
    const response = await this.client.put('/api/users/me/preferences', preferences)
    return response.data
  }

  async uploadAvatar(formData: FormData) {
    const response = await this.client.post('/api/users/avatar', formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
    })
    return response.data
  }

  // Chat endpoints
  async sendMessage(data: {
    message: string
    user_tier?: string
    jailbreak_prompt?: string
    model?: string
  }) {
    const response = await this.client.post('/api/chat', data)
    return response.data
  }

  // Jailbreak endpoints
  async getJailbreakPrompts() {
    const response = await this.client.get('/api/jailbreak/prompts')
    return response.data
  }

  async createJailbreakPrompt(data: {
    title: string
    content: string
    category: string
    tags: string[]
  }) {
    const response = await this.client.post('/api/jailbreak/prompts', data)
    return response.data
  }

  async searchJailbreakPrompts(query: string) {
    const response = await this.client.post('/api/jailbreak/prompts/search', {
      query,
    })
    return response.data
  }

  // Admin endpoints
  async getUsers() {
    const response = await this.client.get('/api/admin/users')
    return response.data
  }

  async getSystemMetrics() {
    const response = await this.client.get('/api/admin/metrics')
    return response.data
  }

  async updateUserStatus(userId: string, status: string) {
    const response = await this.client.put(`/api/admin/users/${userId}/status`, {
      status,
    })
    return response.data
  }

  // Email verification endpoints
  async verifyEmail(token: string, email: string) {
    const response = await this.client.post('/api/auth/verify-email', {
      token,
      email,
    })
    return response.data
  }

  async resendVerificationEmail(email: string) {
    const response = await this.client.post('/api/auth/resend-verification', {
      email,
    })
    return response.data
  }

  // Password reset endpoints
  async requestPasswordReset(email: string) {
    const response = await this.client.post('/api/auth/request-password-reset', {
      email,
    })
    return response.data
  }

  async resetPassword(token: string, password: string) {
    const response = await this.client.post('/api/auth/reset-password', {
      token,
      password,
    })
    return response.data
  }

  async changePassword(currentPassword: string, newPassword: string) {
    const response = await this.client.post('/api/auth/change-password', {
      current_password: currentPassword,
      new_password: newPassword,
    })
    return response.data
  }

  // Billing endpoints
  async generateQRCode(planId: string, amount: number) {
    const response = await this.client.post('/api/billing/generate-qr', {
      plan_id: planId,
      amount,
    })
    return response.data
  }

  async verifyPaymentSlip(formData: FormData) {
    const response = await this.client.post('/api/billing/verify-slip', formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
    })
    return response.data
  }

  async processCreditCardPayment(data: {
    plan_id: string
    amount: number
    card_token: string
  }) {
    const response = await this.client.post('/api/billing/credit-card-payment', data)
    return response.data
  }

  async getSubscriptionStatus() {
    const response = await this.client.get('/api/billing/subscription')
    return response.data
  }

  async getPaymentHistory() {
    const response = await this.client.get('/api/billing/history')
    return response.data
  }

  // Tooling endpoints
  async listTools() {
    const response = await this.client.get('/api/tools')
    return response.data
  }

  async executeTool(payload: {
    tool_name: string
    input: Record<string, any>
    timeout_ms?: number
  }) {
    const response = await this.client.post('/api/tools/execute', payload)
    return response.data
  }

  // Terminal endpoints
  async executeTerminal(command: string) {
    const response = await this.client.post('/api/terminal/execute', { command })
    return response.data
  }

  async getApiKeyStatus() {
    const response = await this.client.get('/api/status/keys')
    return response.data
  }

  // Sandbox endpoints
  async executeSandbox(payload: {
    code: string
    language: string
    timeout_seconds?: number
  }) {
    const response = await this.client.post('/api/sandbox/execute', payload)
    return response.data
  }

  // Health check
  async healthCheck() {
    const response = await this.client.get('/health')
    return response.data
  }

  // Generic request method
  async request<T = any>(config: AxiosRequestConfig): Promise<T> {
    const response = await this.client.request<T>(config)
    return response.data
  }

  // Get axios client instance for advanced usage
  getClient(): AxiosInstance {
    return this.client
  }
}

export const apiClient = new ApiClient()
export default apiClient
