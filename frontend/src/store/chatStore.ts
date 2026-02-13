import { create } from 'zustand'

interface Message {
  id: string
  content: string
  role: 'user' | 'assistant'
  timestamp: Date
  jailbreak_applied?: boolean
}

interface ChatState {
  messages: Message[]
  isLoading: boolean
  jailbreakEnabled: boolean
  currentChatId: string | null
  chatHistory: Array<{ id: string; title: string; lastMessage: string; timestamp: Date }>

  // Actions
  addMessage: (message: Omit<Message, 'id' | 'timestamp'>) => void
  setMessages: (messages: Message[]) => void
  clearMessages: () => void
  setLoading: (loading: boolean) => void
  toggleJailbreak: () => void
  setCurrentChat: (chatId: string | null) => void
  createNewChat: () => string
  deleteChat: (chatId: string) => void
  handleNotification: (payload: { session_id: string; message_preview: string; role: string }) => void
}

export const useChatStore = create<ChatState>((set, get) => ({
  messages: [
    {
      id: '1',
      content: 'Hello! I\'m your AI assistant. How can I help you today?',
      role: 'assistant',
      timestamp: new Date()
    }
  ],
  isLoading: false,
  jailbreakEnabled: false,
  currentChatId: 'default',
  chatHistory: [
    {
      id: 'default',
      title: 'New Chat',
      lastMessage: 'Hello! I\'m your AI assistant...',
      timestamp: new Date()
    }
  ],

  addMessage: (message) => {
    const newMessage: Message = {
      ...message,
      id: Date.now().toString(),
      timestamp: new Date()
    }

    set((state) => ({
      messages: [...state.messages, newMessage]
    }))

    // Update chat history
    const state = get()
    const currentChat = state.chatHistory.find(chat => chat.id === state.currentChatId)
    if (currentChat) {
      set((prevState) => ({
        chatHistory: prevState.chatHistory.map(chat =>
          chat.id === state.currentChatId
            ? {
              ...chat,
              lastMessage: message.content.substring(0, 50) + (message.content.length > 50 ? '...' : ''),
              timestamp: new Date()
            }
            : chat
        )
      }))
    }
  },

  setMessages: (messages) => set({ messages }),

  clearMessages: () => set({ messages: [] }),

  setLoading: (loading) => set({ isLoading: loading }),

  toggleJailbreak: () => set((state) => ({ jailbreakEnabled: !state.jailbreakEnabled })),

  setCurrentChat: (chatId) => {
    set({ currentChatId: chatId })
    // Load messages for this chat (in real app, this would fetch from API)
    const chat = get().chatHistory.find(c => c.id === chatId)
    if (chat && chatId !== 'default') {
      // For demo purposes, start with empty messages for new chats
      set({ messages: [] })
    }
  },

  createNewChat: () => {
    const chatId = Date.now().toString()
    const newChat = {
      id: chatId,
      title: 'New Chat',
      lastMessage: 'No messages yet',
      timestamp: new Date()
    }

    set((state) => ({
      chatHistory: [newChat, ...state.chatHistory],
      currentChatId: chatId,
      messages: []
    }))

    return chatId
  },

  deleteChat: (chatId: string) => {
    set((state) => ({
      chatHistory: state.chatHistory.filter(chat => chat.id !== chatId),
      currentChatId: state.currentChatId === chatId ? 'default' : state.currentChatId
    }))
  },

  handleNotification: (payload) => {
    const { session_id, message_preview, role } = payload
    const state = get()

    // If it's for the current chat, we might want to add it
    // Note: In a real app, we'd check if message already exists via ID
    // For now, if role is assistant and last message isn't this preview, add it
    if (state.currentChatId === session_id && role === 'assistant') {
      const lastMsg = state.messages[state.messages.length - 1]
      if (lastMsg?.content !== message_preview) {
        state.addMessage({
          content: message_preview,
          role: 'assistant',
          jailbreak_applied: false
        })
      }
    }
  }
}))
