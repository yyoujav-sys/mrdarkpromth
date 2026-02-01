import React, { useState, useRef, useEffect } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import {
  Terminal,
  Trash2,
  AlertTriangle,
  Clock,
  History,
  Download,
  Shield
} from 'lucide-react'
import apiClient from '@/lib/api'

interface TerminalCommand {
  id: string
  command: string
  output: string
  status: 'success' | 'error' | 'running'
  timestamp: Date
  executionTime?: number
}

export const TerminalPage: React.FC = () => {
  const [commands, setCommands] = useState<TerminalCommand[]>([])
  const [input, setInput] = useState('')
  const [history, setHistory] = useState<string[]>([])
  const [historyIndex, setHistoryIndex] = useState(-1)
  const [isExecuting, setIsExecuting] = useState(false)
  const [sessionInfo, setSessionInfo] = useState({
    currentDirectory: '/home/ultra',
    user: 'ultra',
    hostname: 'mr-darkpromth'
  })
  const terminalRef = useRef<HTMLDivElement>(null)
  const inputRef = useRef<HTMLInputElement>(null)

  const scrollToBottom = () => {
    terminalRef.current?.scrollIntoView({ behavior: 'smooth', block: 'end' })
  }

  useEffect(() => {
    scrollToBottom()
  }, [commands])

  // Focus input on mount
  useEffect(() => {
    inputRef.current?.focus()
  }, [])

  const executeCommand = async (command: string) => {
    if (!command.trim()) return

    const commandId = Date.now().toString()
    const newCommand: TerminalCommand = {
      id: commandId,
      command,
      output: '',
      status: 'running',
      timestamp: new Date()
    }

    setCommands(prev => [...prev, newCommand])
    setHistory(prev => [...prev, command])
    setHistoryIndex(-1)
    setInput('')
    setIsExecuting(true)

    const startTime = performance.now()

    try {
      const response = await apiClient.getClient().post('/api/terminal/execute', {
        command,
        working_directory: sessionInfo.currentDirectory
      })

      const executionTime = performance.now() - startTime

      setCommands(prev =>
        prev.map(cmd =>
          cmd.id === commandId
            ? {
                ...cmd,
                output: response.data.output || 'Command executed successfully (no output)',
                status: response.data.status || 'success',
                executionTime: Math.round(executionTime)
              }
            : cmd
        )
      )

      // Update current directory if changed
      if (response.data.current_directory) {
        setSessionInfo(prev => ({
          ...prev,
          currentDirectory: response.data.current_directory
        }))
      }
    } catch (error: any) {
      const executionTime = performance.now() - startTime
      const errorMessage = error.response?.data?.message || error.message || 'Command execution failed'

      setCommands(prev =>
        prev.map(cmd =>
          cmd.id === commandId
            ? {
                ...cmd,
                output: `Error: ${errorMessage}`,
                status: 'error',
                executionTime: Math.round(executionTime)
              }
            : cmd
        )
      )
    } finally {
      setIsExecuting(false)
      inputRef.current?.focus()
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      executeCommand(input)
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      if (history.length > 0) {
        const newIndex = historyIndex === -1 ? history.length - 1 : Math.max(0, historyIndex - 1)
        setHistoryIndex(newIndex)
        setInput(history[newIndex])
      }
    } else if (e.key === 'ArrowDown') {
      e.preventDefault()
      if (historyIndex !== -1) {
        const newIndex = historyIndex + 1
        if (newIndex >= history.length) {
          setHistoryIndex(-1)
          setInput('')
        } else {
          setHistoryIndex(newIndex)
          setInput(history[newIndex])
        }
      }
    } else if (e.key === 'l' && e.ctrlKey) {
      e.preventDefault()
      clearTerminal()
    }
  }

  const clearTerminal = () => {
    setCommands([])
    inputRef.current?.focus()
  }

  const downloadSession = () => {
    const sessionText = commands
      .map(
        cmd =>
          `[${cmd.timestamp.toISOString()}] $ ${cmd.command}\n${cmd.output}\n---\n`
      )
      .join('\n')

    const blob = new Blob([sessionText], { type: 'text/plain' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `terminal-session-${new Date().toISOString().split('T')[0]}.txt`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }

  const getPrompt = () => {
    return `${sessionInfo.user}@${sessionInfo.hostname}:${sessionInfo.currentDirectory}$`
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-gray-100 flex items-center space-x-2">
            <Terminal className="h-6 w-6 text-neon-purple" />
            <span>Terminal</span>
          </h1>
          <p className="text-gray-400 mt-1">
            Ultra Tier exclusive - Full shell access with sandboxed execution
          </p>
        </div>
        <div className="flex items-center space-x-3">
          <div className="flex items-center space-x-2 px-3 py-1 bg-neon-purple/10 border border-neon-purple/30 rounded-full">
            <Shield className="h-4 w-4 text-neon-purple" />
            <span className="text-sm text-neon-purple font-medium">Ultra Mode</span>
          </div>
          <Button variant="ghost" size="sm" onClick={downloadSession}>
            <Download className="h-4 w-4" />
          </Button>
          <Button variant="ghost" size="sm" onClick={clearTerminal}>
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      </div>

      {/* Warning Banner */}
      <div className="mb-4 p-4 bg-yellow-500/10 border border-yellow-500/30 rounded-lg flex items-start space-x-3">
        <AlertTriangle className="h-5 w-5 text-yellow-500 flex-shrink-0 mt-0.5" />
        <div>
          <h3 className="text-sm font-medium text-yellow-500">Warning: Full System Access</h3>
          <p className="text-xs text-gray-400 mt-1">
            This terminal provides real shell access. All commands are logged for security.
            Malicious activities will result in immediate account termination.
          </p>
        </div>
      </div>

      {/* Terminal Container */}
      <Card className="flex-1 flex flex-col bg-dark-primary border-dark-accent">
        <CardHeader className="border-b border-dark-accent py-3">
          <CardTitle className="flex items-center justify-between text-sm">
            <div className="flex items-center space-x-2">
              <Terminal className="h-4 w-4 text-neon-purple" />
              <span className="font-mono">bash</span>
            </div>
            <div className="flex items-center space-x-4 text-xs text-gray-500">
              <span className="flex items-center space-x-1">
                <Clock className="h-3 w-3" />
                <span>Session: {Math.floor(commands.length / 2)}m</span>
              </span>
              <span className="flex items-center space-x-1">
                <History className="h-3 w-3" />
                <span>History: {history.length}</span>
              </span>
            </div>
          </CardTitle>
        </CardHeader>

        <CardContent className="flex-1 flex flex-col p-0 overflow-hidden">
          {/* Terminal Output */}
          <div
            ref={terminalRef}
            className="flex-1 overflow-y-auto p-4 font-mono text-sm bg-dark-primary"
          >
            {/* Welcome Message */}
            {commands.length === 0 && (
              <div className="text-gray-500 mb-4">
                <p>Welcome to MR.DarkPromth Ultra Terminal v1.0</p>
                <p>Type 'help' for available commands or start typing...</p>
                <p className="mt-2 text-xs">Keyboard shortcuts: Ctrl+L (clear), ↑↓ (history)</p>
              </div>
            )}

            {/* Command History */}
            {commands.map(cmd => (
              <div key={cmd.id} className="mb-4">
                {/* Command Line */}
                <div className="flex items-center space-x-2 text-gray-300">
                  <span className="text-neon-green">{getPrompt()}</span>
                  <span>{cmd.command}</span>
                </div>

                {/* Output */}
                <div
                  className={`mt-2 pl-4 border-l-2 ${
                    cmd.status === 'error'
                      ? 'border-red-500 text-red-400'
                      : cmd.status === 'running'
                        ? 'border-yellow-500 text-gray-400'
                        : 'border-gray-600 text-gray-300'
                  }`}
                >
                  {cmd.status === 'running' ? (
                    <div className="flex items-center space-x-2">
                      <div className="w-2 h-2 bg-yellow-500 rounded-full animate-pulse"></div>
                      <span>Executing...</span>
                    </div>
                  ) : (
                    <pre className="whitespace-pre-wrap break-all text-xs">
                      {cmd.output}
                    </pre>
                  )}

                  {/* Execution Info */}
                  {cmd.status !== 'running' && cmd.executionTime && (
                    <div className="mt-2 text-xs text-gray-500">
                      Executed in {cmd.executionTime}ms
                    </div>
                  )}
                </div>
              </div>
            ))}

            {/* Current Input Line */}
            <div className="flex items-center space-x-2 text-gray-300">
              <span className="text-neon-green">{getPrompt()}</span>
              <input
                ref={inputRef}
                type="text"
                value={input}
                onChange={e => setInput(e.target.value)}
                onKeyDown={handleKeyDown}
                disabled={isExecuting}
                className="flex-1 bg-transparent border-none outline-none text-gray-100 font-mono"
                placeholder={isExecuting ? 'Executing...' : 'Type command...'}
                autoFocus
              />
            </div>
          </div>

          {/* Quick Commands */}
          <div className="border-t border-dark-accent p-3 bg-dark-secondary">
            <div className="flex items-center space-x-2 overflow-x-auto">
              <span className="text-xs text-gray-500 flex-shrink-0">Quick:</span>
              {['ls -la', 'pwd', 'whoami', 'help', 'clear'].map(cmd => (
                <Button
                  key={cmd}
                  variant="ghost"
                  size="sm"
                  onClick={() => executeCommand(cmd)}
                  disabled={isExecuting}
                  className="text-xs px-2 py-1 h-auto"
                >
                  {cmd}
                </Button>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Session Info */}
      <div className="mt-4 grid grid-cols-3 gap-4">
        <div className="p-3 bg-dark-secondary rounded-lg border border-dark-accent">
          <div className="text-xs text-gray-500">Working Directory</div>
          <div className="text-sm font-mono text-neon-blue truncate">
            {sessionInfo.currentDirectory}
          </div>
        </div>
        <div className="p-3 bg-dark-secondary rounded-lg border border-dark-accent">
          <div className="text-xs text-gray-500">User</div>
          <div className="text-sm font-mono text-neon-purple">{sessionInfo.user}</div>
        </div>
        <div className="p-3 bg-dark-secondary rounded-lg border border-dark-accent">
          <div className="text-xs text-gray-500">Hostname</div>
          <div className="text-sm font-mono text-neon-green">{sessionInfo.hostname}</div>
        </div>
      </div>
    </div>
  )
}
