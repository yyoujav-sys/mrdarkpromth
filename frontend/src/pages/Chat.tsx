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
  History,
  X,
  Trash2,
  Save,
  ChevronDown,
  ChevronRight,
  Terminal as TerminalIcon,
  Code
} from 'lucide-react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import remarkMath from 'remark-math'
import rehypeKatex from 'rehype-katex'
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism'
import 'katex/dist/katex.min.css'
import apiClient from '@/lib/api'
import { Link } from 'react-router-dom'
import { useAuthStore } from '@/store/authStore'
import { useToast } from '@/components/ui/Toast'
import { GlitchText } from '@/components/ui/GlitchText'
import { useLanguage } from '@/contexts/LanguageContext'

interface Message {
  id: string
  content: string
  role: 'user' | 'assistant'
  timestamp: Date
  jailbreak_applied?: boolean
}

interface ChatSession {
  id: string
  title: string
  messages: Message[]
  createdAt: Date
  updatedAt: Date
}

const MessageComponent: React.FC<{ message: Message }> = ({ message }) => {
  const [showThought, setShowThought] = useState(false);

  const processContent = (content: string) => {
    const thoughtMatch = content.match(/<thought>([\s\S]*?)<\/thought>/);
    if (thoughtMatch) {
      const thought = thoughtMatch[1];
      const remaining = content.replace(/<thought>[\s\S]*?<\/thought>/, '').trim();
      return { thought, content: remaining };
    }
    return { thought: null, content };
  };

  const { thought, content } = processContent(message.content);

  return (
    <div
      className={`flex items-start space-x-3 ${message.role === 'user' ? 'justify-end' : 'justify-start'
        }`}
    >
      {message.role === 'assistant' && (
        <div className="flex-shrink-0 w-8 h-8 bg-neon-purple/20 rounded-full flex items-center justify-center">
          <Bot className="h-4 w-4 text-neon-purple" />
        </div>
      )}

      <div
        className={`max-w-2xl rounded-lg px-4 py-3 ${message.role === 'user'
          ? 'bg-neon-purple/10 text-gray-100'
          : 'bg-dark-secondary text-gray-100'
          }`}
      >
        {message.jailbreak_applied && (
          <div className="mb-2 text-xs text-neon-purple font-medium">
            Ultra Mode Response
          </div>
        )}

        {thought && (
          <div className="mb-3 border-b border-dark-accent pb-2">
            <button
              onClick={() => setShowThought(!showThought)}
              className="flex items-center space-x-2 text-xs text-gray-400 hover:text-gray-200 transition-colors"
            >
              {showThought ? <ChevronDown className="h-3 w-3" /> : <ChevronRight className="h-3 w-3" />}
              <span className="font-mono uppercase tracking-widest text-[10px]">Thinking Process</span>
            </button>
            {showThought && (
              <div className="mt-2 p-3 bg-black/30 rounded text-xs text-gray-400 font-mono italic whitespace-pre-wrap border-l-2 border-neon-purple/50">
                {thought}
              </div>
            )}
          </div>
        )}

        <div className="prose prose-invert max-w-none">
          <ReactMarkdown
            remarkPlugins={[remarkGfm, remarkMath]}
            rehypePlugins={[rehypeKatex]}
            components={{
              code({ node, inline, className, children, ...props }: any) {
                const match = /language-(\w+)/.exec(className || '')
                return !inline && match ? (
                  <SyntaxHighlighter
                    style={vscDarkPlus}
                    language={match[1]}
                    PreTag="div"
                    {...props}
                  >
                    {String(children).replace(/\n$/, '')}
                  </SyntaxHighlighter>
                ) : (
                  <code className={className} {...props}>
                    {children}
                  </code>
                )
              }
            }}
          >
            {content}
          </ReactMarkdown>
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
  );
};

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
  const [showHistory, setShowHistory] = useState(false)
  const [showSettings, setShowSettings] = useState(false)
  const [chatSessions, setChatSessions] = useState<ChatSession[]>([])
  const [currentSessionId, setCurrentSessionId] = useState<string>('')
  const [chatSettings, setChatSettings] = useState({
    autoSave: true,
    soundEnabled: false,
    enterToSend: true,
    theme: 'dark'
  })
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const { user } = useAuthStore()
  const { t, language, setLanguage } = useLanguage()

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  useEffect(() => {
    scrollToBottom()
  }, [messages])

  // Load chat sessions from localStorage on mount
  useEffect(() => {
    const savedSessions = localStorage.getItem('chat_sessions')
    if (savedSessions) {
      const parsed = JSON.parse(savedSessions)
      setChatSessions(parsed.map((s: any) => ({
        ...s,
        createdAt: new Date(s.createdAt),
        updatedAt: new Date(s.updatedAt),
        messages: s.messages.map((m: any) => ({
          ...m,
          timestamp: new Date(m.timestamp)
        }))
      })))
    }
    const savedSettings = localStorage.getItem('chat_settings')
    if (savedSettings) {
      setChatSettings(JSON.parse(savedSettings))
    }
  }, [])

  // Save current session when messages change
  useEffect(() => {
    if (chatSettings.autoSave && messages.length > 1) {
      saveCurrentSession()
    }
  }, [messages])

  const saveCurrentSession = () => {
    const title = messages.find(m => m.role === 'user')?.content.slice(0, 50) || 'New Chat'
    const newSession: ChatSession = {
      id: currentSessionId || Date.now().toString(),
      title,
      messages: [...messages],
      createdAt: new Date(),
      updatedAt: new Date()
    }

    setChatSessions(prev => {
      const existing = prev.find(s => s.id === newSession.id)
      let updated
      if (existing) {
        updated = prev.map(s => s.id === newSession.id ? newSession : s)
      } else {
        updated = [newSession, ...prev].slice(0, 50) // Keep last 50 sessions
      }
      localStorage.setItem('chat_sessions', JSON.stringify(updated))
      return updated
    })

    if (!currentSessionId) {
      setCurrentSessionId(newSession.id)
    }
  }

  const loadSession = (session: ChatSession) => {
    setMessages(session.messages)
    setCurrentSessionId(session.id)
    setShowHistory(false)
  }

  const deleteSession = (sessionId: string, e: React.MouseEvent) => {
    e.stopPropagation()
    setChatSessions(prev => {
      const updated = prev.filter(s => s.id !== sessionId)
      localStorage.setItem('chat_sessions', JSON.stringify(updated))
      return updated
    })
    if (currentSessionId === sessionId) {
      setCurrentSessionId('')
    }
  }

  const startNewChat = () => {
    setMessages([{
      id: '1',
      content: "Hello! I'm your AI assistant. How can I help you today?",
      role: 'assistant',
      timestamp: new Date()
    }])
    setCurrentSessionId('')
    setShowHistory(false)
  }

  const saveSettings = () => {
    localStorage.setItem('chat_settings', JSON.stringify(chatSettings))
    setShowSettings(false)
  }

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
      toast('Connection to Neural Core failed. Retrying...', 'error')
      const assistantMessage: Message = {
        id: (Date.now() + 1).toString(),
        content: '**SYSTEM ERROR:** Connection interrupted. Neural link unstable.',
        role: 'assistant',
        timestamp: new Date(),
        jailbreak_applied: false
      }
      setMessages(prev => [...prev, assistantMessage])
    } finally {
      setIsLoading(false)
    }
  }

  const { toast } = useToast()

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }





  return (
    <div className="h-full flex flex-col bg-black text-neon-green font-mono">
      {/* Header */}
      <div className="flex items-center justify-between mb-6 border-b border-dark-accent pb-4">
        <div>
          <GlitchText
            text={t('chat_title')}
            as="h1"
            className="text-2xl font-black tracking-tighter text-white"
          />
          <p className="text-gray-500 mt-1 text-xs uppercase tracking-widest">
            {t('chat_subtitle')}
          </p>
        </div>
        <div className="flex items-center space-x-3">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setLanguage(language === 'en' ? 'th' : 'en')}
            className="font-mono text-xs border border-dark-accent text-neon-green hover:bg-neon-green/10"
          >
            [{language.toUpperCase()}]
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setJailbreakEnabled(!jailbreakEnabled)}
            className={`flex items-center space-x-2 border-dashed ${jailbreakEnabled ? 'border-neon-red text-neon-red animate-pulse' : 'border-dark-accent text-gray-500'}`}
          >
            <Zap className="h-4 w-4" />
            <span>{jailbreakEnabled ? 'ULTRA MODE: ON' : 'ULTRA MODE: OFF'}</span>
          </Button>
          <Button variant="ghost" size="sm" onClick={() => setShowHistory(true)}>
            <History className="h-4 w-4" />
          </Button>
          <Button variant="ghost" size="sm" onClick={() => setShowSettings(true)}>
            <Settings className="h-4 w-4" />
          </Button>
          <Link to="/sandbox">
            <Button variant="ghost" size="sm" title="Open Sandbox">
              <Code className="h-4 w-4" />
            </Button>
          </Link>
          <Link to="/terminal">
            <Button variant="ghost" size="sm" title="Open Terminal">
              <TerminalIcon className="h-4 w-4" />
            </Button>
          </Link>
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
              <MessageComponent key={message.id} message={message} />
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

      {/* History Modal */}
      {showHistory && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <Card className="w-full max-w-lg max-h-[80vh] flex flex-col">
            <CardHeader className="flex items-center justify-between border-b border-dark-accent">
              <CardTitle className="flex items-center space-x-2">
                <History className="h-5 w-5 text-neon-purple" />
                <span>Chat History</span>
              </CardTitle>
              <Button variant="ghost" size="sm" onClick={() => setShowHistory(false)}>
                <X className="h-4 w-4" />
              </Button>
            </CardHeader>
            <CardContent className="flex-1 overflow-y-auto p-4">
              <Button
                variant="neon"
                size="sm"
                className="w-full mb-4"
                onClick={startNewChat}
              >
                + New Chat
              </Button>
              {chatSessions.length === 0 ? (
                <p className="text-gray-500 text-center py-8">No chat history yet</p>
              ) : (
                <div className="space-y-2">
                  {chatSessions.map(session => (
                    <div
                      key={session.id}
                      onClick={() => loadSession(session)}
                      className={`p-3 rounded-lg cursor-pointer hover:bg-dark-accent transition-colors flex items-center justify-between ${session.id === currentSessionId ? 'bg-neon-purple/20 border border-neon-purple/40' : 'bg-dark-secondary'
                        }`}
                    >
                      <div className="flex-1 min-w-0">
                        <p className="font-medium truncate">{session.title}</p>
                        <p className="text-xs text-gray-500">
                          {session.updatedAt.toLocaleDateString()} • {session.messages.length} messages
                        </p>
                      </div>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={(e) => deleteSession(session.id, e)}
                        className="ml-2 text-red-400 hover:text-red-300"
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      )}

      {/* Settings Modal */}
      {showSettings && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <Card className="w-full max-w-md">
            <CardHeader className="flex items-center justify-between border-b border-dark-accent">
              <CardTitle className="flex items-center space-x-2">
                <Settings className="h-5 w-5 text-neon-purple" />
                <span>Chat Settings</span>
              </CardTitle>
              <Button variant="ghost" size="sm" onClick={() => setShowSettings(false)}>
                <X className="h-4 w-4" />
              </Button>
            </CardHeader>
            <CardContent className="p-4 space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-medium">Auto-save chats</p>
                  <p className="text-sm text-gray-500">Automatically save conversations</p>
                </div>
                <button
                  onClick={() => setChatSettings(s => ({ ...s, autoSave: !s.autoSave }))}
                  className={`w-12 h-6 rounded-full transition-colors ${chatSettings.autoSave ? 'bg-neon-purple' : 'bg-gray-600'
                    }`}
                >
                  <div className={`w-4 h-4 bg-white rounded-full transition-transform ${chatSettings.autoSave ? 'translate-x-7' : 'translate-x-1'
                    }`} />
                </button>
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-medium">Sound effects</p>
                  <p className="text-sm text-gray-500">Play sounds for notifications</p>
                </div>
                <button
                  onClick={() => setChatSettings(s => ({ ...s, soundEnabled: !s.soundEnabled }))}
                  className={`w-12 h-6 rounded-full transition-colors ${chatSettings.soundEnabled ? 'bg-neon-purple' : 'bg-gray-600'
                    }`}
                >
                  <div className={`w-4 h-4 bg-white rounded-full transition-transform ${chatSettings.soundEnabled ? 'translate-x-7' : 'translate-x-1'
                    }`} />
                </button>
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-medium">Enter to send</p>
                  <p className="text-sm text-gray-500">Press Enter to send message</p>
                </div>
                <button
                  onClick={() => setChatSettings(s => ({ ...s, enterToSend: !s.enterToSend }))}
                  className={`w-12 h-6 rounded-full transition-colors ${chatSettings.enterToSend ? 'bg-neon-purple' : 'bg-gray-600'
                    }`}
                >
                  <div className={`w-4 h-4 bg-white rounded-full transition-transform ${chatSettings.enterToSend ? 'translate-x-7' : 'translate-x-1'
                    }`} />
                </button>
              </div>
              <div className="pt-4 border-t border-dark-accent">
                <Button variant="neon" className="w-full" onClick={saveSettings}>
                  <Save className="h-4 w-4 mr-2" />
                  Save Settings
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  )
}
