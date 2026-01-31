import { useChatStore } from '@/store/chatStore'
import { useAuthStore } from '@/store/authStore'

export class WebSocketClient {
  private ws: WebSocket | null = null
  private url: string
  private reconnectAttempts = 0
  private maxReconnectAttempts = 5
  private reconnectDelay = 1000
  private isConnecting = false

  constructor(url: string) {
    this.url = url
  }

  connect() {
    if (this.isConnecting || (this.ws && this.ws.readyState === WebSocket.OPEN)) {
      return
    }

    this.isConnecting = true

    try {
      const token = localStorage.getItem('token')
      const wsUrl = `${this.url}?token=${token}`
      
      this.ws = new WebSocket(wsUrl)

      this.ws.onopen = () => {
        console.log('WebSocket connected')
        this.isConnecting = false
        this.reconnectAttempts = 0
      }

      this.ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data)
          this.handleMessage(data)
        } catch (error) {
          console.error('Error parsing WebSocket message:', error)
        }
      }

      this.ws.onclose = (event) => {
        console.log('WebSocket disconnected:', event.code, event.reason)
        this.isConnecting = false
        this.handleReconnect()
      }

      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error)
        this.isConnecting = false
      }

    } catch (error) {
      console.error('Failed to connect to WebSocket:', error)
      this.isConnecting = false
      this.handleReconnect()
    }
  }

  private handleMessage(data: any) {
    const chatStore = useChatStore.getState()

    switch (data.type) {
      case 'message':
        chatStore.addMessage({
          content: data.content,
          role: data.role,
          jailbreak_applied: data.jailbreak_applied
        })
        chatStore.setLoading(false)
        break

      case 'typing':
        chatStore.setLoading(data.isTyping)
        break

      case 'error':
        console.error('WebSocket error message:', data.message)
        chatStore.setLoading(false)
        break

      default:
        console.log('Unknown WebSocket message type:', data.type)
    }
  }

  private handleReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++
      console.log(`Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`)
      
      setTimeout(() => {
        this.connect()
      }, this.reconnectDelay * this.reconnectAttempts)
    } else {
      console.error('Max reconnection attempts reached')
    }
  }

  sendMessage(message: string, jailbreakEnabled: boolean = false) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      const data = {
        type: 'message',
        content: message,
        jailbreak_enabled: jailbreakEnabled,
        timestamp: new Date().toISOString()
      }
      
      this.ws.send(JSON.stringify(data))
    } else {
      console.error('WebSocket is not connected')
      // Fallback to HTTP API
      this.sendMessageViaAPI(message, jailbreakEnabled)
    }
  }

  private async sendMessageViaAPI(message: string, jailbreakEnabled: boolean) {
    try {
      const { default: apiClient } = await import('./api')
      const chatStore = useChatStore.getState()
      
      chatStore.setLoading(true)
      
      const authState = useAuthStore.getState()
      const tier = authState.user?.tier ?? 'Free'

      const response = await apiClient.sendMessage({
        message,
        user_tier: tier,
        jailbreak_prompt: jailbreakEnabled ? 'enabled' : undefined
      })
      
      chatStore.addMessage({
        content: response.message,
        role: 'assistant',
        jailbreak_applied: response.jailbreak_applied
      })
      
      chatStore.setLoading(false)
    } catch (error) {
      console.error('Failed to send message via API:', error)
      const chatStore = useChatStore.getState()
      chatStore.setLoading(false)
    }
  }

  disconnect() {
    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
  }

  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN
  }
}

// Singleton instance
let wsClient: WebSocketClient | null = null

export const getWebSocketClient = () => {
  if (!wsClient) {
    const wsUrl = import.meta.env.VITE_WS_URL || 'ws://localhost:8080/ws'
    wsClient = new WebSocketClient(wsUrl)
  }
  return wsClient
}

export const useWebSocket = () => {
  const client = getWebSocketClient()
  
  return {
    connect: () => client.connect(),
    disconnect: () => client.disconnect(),
    sendMessage: (message: string, jailbreakEnabled?: boolean) => 
      client.sendMessage(message, jailbreakEnabled),
    isConnected: () => client.isConnected()
  }
}
