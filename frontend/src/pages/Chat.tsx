import React, { useState, useRef, useEffect } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Input } from '@/components/ui/Input'
import { 
  Send, 
  Bot, 
  User, 
  Zap,
  Settings,
  History
} from 'lucide-react'
import ReactMarkdown from 'react-markdown'
import apiClient from '@/lib/api'
import { useAuthStore } from '@/store/authStore'

interface Message {
  id: string
  content: string
  role: 'user' | 'assistant'
  timestamp: Date
  jailbreak_applied?: boolean
}

interface ThinkingStep {
  id: string
  title: string
  description: string
  status: 'pending' | 'running' | 'completed'
  duration?: number
}

export const Chat: React.FC = () => {
  const [messages, setMessages] = useState<Message[]>([
    {
      id: '1',
      content: 'Hello! I\'m your AI assistant. How can I help you today?',
      role: 'assistant',
      timestamp: new Date()
    }
  ])
  const [input, setInput] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [jailbreakEnabled, setJailbreakEnabled] = useState(false)
  const [_showThinking, _setShowThinking] = useState(false)
  const [_thinkingSteps, _setThinkingSteps] = useState<ThinkingStep[]>([])
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const { user } = useAuthStore()

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  useEffect(() => {
    scrollToBottom()
  }, [messages])

  const handleSend = async () => {
    if (!input.trim()) return

    const messageContent = input

    const userMessage: Message = {
      id: Date.now().toString(),
      content: input,
      role: 'user',
      timestamp: new Date()
    }

    setMessages(prev => [...prev, userMessage])
    setInput('')
    setIsLoading(true)

    try {
      const tier = user?.tier ?? 'Free'
      const response = await apiClient.sendMessage({
        message: messageContent,
        user_tier: tier,
        jailbreak_prompt: jailbreakEnabled ? 'enabled' : undefined
      })

      const assistantMessage: Message = {
        id: response.id ?? (Date.now() + 1).toString(),
        content: response.response ?? 'No response received.',
        role: 'assistant',
        timestamp: new Date(response.timestamp ?? Date.now()),
        jailbreak_applied: response.jailbreak_applied
      }

      setMessages(prev => [...prev, assistantMessage])
    } catch (error) {
      const assistantMessage: Message = {
        id: (Date.now() + 1).toString(),
        content: 'Unable to reach the AI service. Please try again.',
        role: 'assistant',
        timestamp: new Date(),
        jailbreak_applied: false
      }
      setMessages(prev => [...prev, assistantMessage])
    } finally {
      setIsLoading(false)
    }
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-gray-100">AI Chat Interface</h1>
          <p className="text-gray-400 mt-1">Advanced AI conversation with jailbreak capabilities</p>
        </div>
        <div className="flex items-center space-x-3">
          <Button
            variant={jailbreakEnabled ? 'neon' : 'ghost'}
            size="sm"
            onClick={() => setJailbreakEnabled(!jailbreakEnabled)}
            className="flex items-center space-x-2"
          >
            <Zap className="h-4 w-4" />
            <span>{jailbreakEnabled ? 'Ultra Mode' : 'Standard'}</span>
          </Button>
          <Button variant="ghost" size="sm">
            <History className="h-4 w-4" />
          </Button>
          <Button variant="ghost" size="sm">
            <Settings className="h-4 w-4" />
          </Button>
        </div>
      </div>

      {/* Chat Container */}
      <Card className="flex-1 flex flex-col">
        <CardHeader className="border-b border-dark-accent">
          <CardTitle className="flex items-center space-x-2">
            <Bot className="h-5 w-5 text-neon-purple" />
            <span>AI Assistant</span>
            {jailbreakEnabled && (
              <span className="px-2 py-1 text-xs bg-neon-purple/20 text-neon-purple rounded-full">
                Ultra Mode Active
              </span>
            )}
          </CardTitle>
        </CardHeader>
        
        <CardContent className="flex-1 flex flex-col p-0">
          {/* Messages */}
          <div className="flex-1 overflow-y-auto p-4 space-y-4">
            {messages.map((message) => (
              <div
                key={message.id}
                className={`flex items-start space-x-3 ${
                  message.role === 'user' ? 'justify-end' : 'justify-start'
                }`}
              >
                {message.role === 'assistant' && (
                  <div className="flex-shrink-0 w-8 h-8 bg-neon-purple/20 rounded-full flex items-center justify-center">
                    <Bot className="h-4 w-4 text-neon-purple" />
                  </div>
                )}
                
                <div
                  className={`max-w-2xl rounded-lg px-4 py-3 ${
                    message.role === 'user'
                      ? 'bg-neon-purple/10 text-gray-100'
                      : 'bg-dark-secondary text-gray-100'
                  }`}
                >
                  {message.jailbreak_applied && (
                    <div className="mb-2 text-xs text-neon-purple font-medium">
                      Ultra Mode Response
                    </div>
                  )}
                  <div className="prose prose-invert max-w-none">
                    <ReactMarkdown>{message.content}</ReactMarkdown>
                  </div>
                  <div className="mt-2 text-xs text-gray-500">
                    {message.timestamp.toLocaleTimeString()}
                  </div>
                </div>

                {message.role === 'user' && (
                  <div className="flex-shrink-0 w-8 h-8 bg-neon-blue/20 rounded-full flex items-center justify-center">
                    <User className="h-4 w-4 text-neon-blue" />
                  </div>
                )}
              </div>
            ))}
            
            {isLoading && (
              <div className="flex items-start space-x-3">
                <div className="flex-shrink-0 w-8 h-8 bg-neon-purple/20 rounded-full flex items-center justify-center">
                  <Bot className="h-4 w-4 text-neon-purple" />
                </div>
                <div className="bg-dark-secondary rounded-lg px-4 py-3">
                  <div className="flex space-x-1">
                    <div className="w-2 h-2 bg-neon-purple rounded-full animate-bounce"></div>
                    <div className="w-2 h-2 bg-neon-purple rounded-full animate-bounce" style={{ animationDelay: '0.1s' }}></div>
                    <div className="w-2 h-2 bg-neon-purple rounded-full animate-bounce" style={{ animationDelay: '0.2s' }}></div>
                  </div>
                </div>
              </div>
            )}
            <div ref={messagesEndRef} />
          </div>

          {/* Input */}
          <div className="border-t border-dark-accent p-4">
            <div className="flex space-x-3">
              <Input
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyPress={handleKeyPress}
                placeholder="Type your message... (Shift+Enter for new line)"
                disabled={isLoading}
                className="flex-1"
              />
              <Button
                onClick={handleSend}
                disabled={isLoading || !input.trim()}
                variant="primary"
                className="px-4"
              >
                <Send className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
