import React, { useState, useEffect } from 'react'
import { Button } from '@/components/ui/Button'
import { 
  Terminal, 
  Send, 
  Download,
  Copy,
  CheckCircle,
  AlertCircle,
  Loader2,
  RotateCcw
} from 'lucide-react'
import apiClient from '@/lib/api'

interface TerminalSession {
  id: string
  status: 'idle' | 'running' | 'error'
  currentDirectory: string
  user: string
  hostname: string
  environment: Record<string, string>
}

interface TerminalCommand {
  id: string
  command: string
  output: string
  status: 'pending' | 'running' | 'completed' | 'error'
  timestamp: Date
  executionTime?: number
  exitCode?: number
}

export const VSCodeTerminal: React.FC = () => {
  const [session, setSession] = useState<TerminalSession | null>(null)
  const [commands, setCommands] = useState<TerminalCommand[]>([])
  const [input, setInput] = useState('')
  const [isConnected, setIsConnected] = useState(false)
  const [isExecuting, setIsExecuting] = useState(false)
  const [copiedCommand, setCopiedCommand] = useState<string | null>(null)

  useEffect(() => {
    initializeTerminal()
  }, [])

  const initializeTerminal = async () => {
    try {
      const response = await apiClient.getClient().post('/api/terminal/init')
      setSession(response.data.session)
      setIsConnected(true)
    } catch (error) {
      console.error('Failed to initialize terminal:', error)
    }
  }

  const executeCommand = async (command: string) => {
    if (!command.trim() || !session) return

    const commandId = Date.now().toString()
    const newCommand: TerminalCommand = {
      id: commandId,
      command,
      output: '',
      status: 'pending',
      timestamp: new Date()
    }

    setCommands(prev => [...prev, newCommand])
    setIsExecuting(true)
    setInput('')

    try {
      // Update command to running status
      setCommands(prev => 
        prev.map(cmd => 
          cmd.id === commandId 
            ? { ...cmd, status: 'running' }
            : cmd
        )
      )

      const response = await apiClient.getClient().post('/api/terminal/execute', {
        command,
        working_directory: session.currentDirectory
      })

      setCommands(prev => 
        prev.map(cmd => 
          cmd.id === commandId 
            ? {
                ...cmd,
                status: response.data.success ? 'completed' : 'error',
                output: response.data.output,
                executionTime: response.data.execution_time,
                exitCode: response.data.exit_code
              }
            : cmd
        )
      )

      // Update session if directory changed
      if (response.data.new_directory) {
        setSession(prev => prev ? {
          ...prev,
          currentDirectory: response.data.new_directory
        } : null)
      }

    } catch (error) {
      setCommands(prev => 
        prev.map(cmd => 
          cmd.id === commandId 
            ? {
                ...cmd,
                status: 'error',
                output: `Error: ${error}`,
                exitCode: -1
              }
            : cmd
        )
      )
    } finally {
      setIsExecuting(false)
    }
  }

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
    setCopiedCommand(text)
    setTimeout(() => setCopiedCommand(null), 2000)
  }

  const clearTerminal = () => {
    setCommands([])
  }

  const downloadSession = () => {
    const sessionData = {
      session,
      commands: commands.map(cmd => ({
        command: cmd.command,
        output: cmd.output,
        status: cmd.status,
        timestamp: cmd.timestamp,
        executionTime: cmd.executionTime,
        exitCode: cmd.exitCode
      }))
    }
    
    const blob = new Blob([JSON.stringify(sessionData, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `terminal-session-${Date.now()}.json`
    a.click()
    URL.revokeObjectURL(url)
  }

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'pending':
        return <Loader2 className="w-4 h-4 text-gray-400 animate-spin" />
      case 'running':
        return <Loader2 className="w-4 h-4 text-blue-500 animate-spin" />
      case 'completed':
        return <CheckCircle className="w-4 h-4 text-green-500" />
      case 'error':
        return <AlertCircle className="w-4 h-4 text-red-500" />
      default:
        return null
    }
  }

  return (
    <div className="h-full flex flex-col bg-gray-900 text-gray-100">
      {/* Terminal Header */}
      <div className="flex items-center justify-between p-3 bg-gray-800 border-b border-gray-700">
        <div className="flex items-center gap-3">
          <Terminal className="w-5 h-5 text-green-400" />
          <span className="font-mono text-sm">
            {session ? `${session.user}@${session.hostname}:${session.currentDirectory}` : 'Connecting...'}
          </span>
          <div className={`w-2 h-2 rounded-full ${isConnected ? 'bg-green-500' : 'bg-red-500'}`} />
        </div>
        
        <div className="flex items-center gap-2">
          <Button
            onClick={clearTerminal}
            variant="ghost"
            size="sm"
            className="text-gray-400 hover:text-white"
          >
            <RotateCcw className="w-4 h-4" />
          </Button>
          <Button
            onClick={downloadSession}
            variant="ghost"
            size="sm"
            className="text-gray-400 hover:text-white"
          >
            <Download className="w-4 h-4" />
          </Button>
        </div>
      </div>

      {/* Terminal Output */}
      <div className="flex-1 overflow-y-auto p-4 font-mono text-sm">
        {commands.map((cmd) => (
          <div key={cmd.id} className="mb-4">
            {/* Command Input */}
            <div className="flex items-center gap-2 mb-1">
              <span className="text-green-400">$</span>
              <span className="text-white">{cmd.command}</span>
              <div className="ml-auto">
                {getStatusIcon(cmd.status)}
              </div>
              <Button
                onClick={() => copyToClipboard(cmd.command)}
                variant="ghost"
                size="sm"
                className="text-gray-500 hover:text-white ml-2"
              >
                {copiedCommand === cmd.command ? (
                  <CheckCircle className="w-3 h-3 text-green-400" />
                ) : (
                  <Copy className="w-3 h-3" />
                )}
              </Button>
            </div>
            
            {/* Command Output */}
            {cmd.output && (
              <div className={`ml-4 whitespace-pre-wrap ${
                cmd.status === 'error' ? 'text-red-400' : 'text-gray-300'
              }`}>
                {cmd.output}
              </div>
            )}
            
            {/* Execution Info */}
            {cmd.executionTime && (
              <div className="ml-4 text-xs text-gray-500 mt-1">
                Execution time: {cmd.executionTime}ms
                {cmd.exitCode !== undefined && ` • Exit code: ${cmd.exitCode}`}
              </div>
            )}
          </div>
        ))}
        
        {commands.length === 0 && (
          <div className="text-gray-500 text-center py-8">
            Ready to execute commands...
          </div>
        )}
      </div>

      {/* Terminal Input */}
      <div className="border-t border-gray-700 p-3">
        <div className="flex items-center gap-2">
          <span className="text-green-400">$</span>
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyPress={(e) => {
              if (e.key === 'Enter' && !isExecuting) {
                executeCommand(input)
              }
            }}
            placeholder="Enter command..."
            className="flex-1 bg-transparent border-none outline-none text-white placeholder-gray-500 font-mono text-sm"
            disabled={!isConnected || isExecuting}
          />
          <Button
            onClick={() => executeCommand(input)}
            disabled={!isConnected || isExecuting || !input.trim()}
            variant="ghost"
            size="sm"
            className="text-gray-400 hover:text-white"
          >
            <Send className="w-4 h-4" />
          </Button>
        </div>
      </div>
    </div>
  )
}
