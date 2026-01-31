import axios from 'axios'
import type { AxiosInstance, AxiosRequestConfig } from 'axios'

class ApiClient {
  private client: AxiosInstance

  constructor() {
    this.client = axios.create({
      baseURL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080',
      timeout: 10000,
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
        if (error.response?.status === 401) {
          // Token expired or invalid
          localStorage.removeItem('token')
          window.location.href = '/login'
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

  // User endpoints
  async getUserProfile() {
    const response = await this.client.get('/api/users/me')
    return response.data
  }

  async updateUserProfile(userData: Partial<{
    username: string
    email: string
  }>) {
    const response = await this.client.put('/api/users/me', userData)
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
}

export const apiClient = new ApiClient()
export default apiClient
