import React, { useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Input } from '@/components/ui/Input'
import { 
  Play, 
  RefreshCw, 
  Terminal,
  Code,
  Zap,
  AlertTriangle
} from 'lucide-react'
import apiClient from '@/lib/api'

interface ExecutionResult {
  id: string
  command: string
  output: string
  status: 'success' | 'error' | 'running'
  timestamp: Date
  executionTime?: number
  exitCode?: number
  language?: string
  securityViolations?: string[]
}

export const Sandbox: React.FC = () => {
  const [command, setCommand] = useState('')
  const [language, setLanguage] = useState('bash')
  const [isRunning, setIsRunning] = useState(false)
  const [results, setResults] = useState<ExecutionResult[]>([])
  const [errorMessage, setErrorMessage] = useState<string | null>(null)

  const executeCommand = async () => {
    if (!command.trim()) return

    const newResult: ExecutionResult = {
      id: Date.now().toString(),
      command,
      output: '',
      status: 'running',
      timestamp: new Date(),
      language
    }

    setResults(prev => [newResult, ...prev])
    setIsRunning(true)
    setCommand('')
    setErrorMessage(null)

    try {
      const response = await apiClient.executeSandbox({
        code: command,
        language
      })

      const output = [response.stdout, response.stderr]
        .filter(Boolean)
        .join('\n')
        .trim()
        || 'No output returned.'

      setResults(prev => prev.map(result =>
        result.id === newResult.id
          ? {
              ...result,
              output,
              status: response.exit_code === 0 ? 'success' : 'error',
              executionTime: response.execution_time_ms,
              exitCode: response.exit_code,
              securityViolations: response.security_violations
            }
          : result
      ))
    } catch (error) {
      setResults(prev => prev.map(result =>
        result.id === newResult.id
          ? {
              ...result,
              output: 'Sandbox execution failed. Please try again.',
              status: 'error'
            }
          : result
      ))
      setErrorMessage('Unable to execute in sandbox. Please verify your tier and try again.')
    } finally {
      setIsRunning(false)
    }
  }

  const clearResults = () => {
    setResults([])
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      executeCommand()
    }
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-gray-100">AI Sandbox</h1>
        <p className="mt-2 text-gray-400">
          Execute and test AI commands in a secure environment
        </p>
      </div>

      {/* Control Panel */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Terminal className="h-5 w-5 text-neon-purple" />
            <span>Command Terminal</span>
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {errorMessage && (
            <div className="rounded-lg border border-red-500/40 bg-red-500/10 px-4 py-2 text-sm text-red-300">
              {errorMessage}
            </div>
          )}
          <div className="flex space-x-3">
            <Input
              value={command}
              onChange={(e) => setCommand(e.target.value)}
              onKeyPress={handleKeyPress}
              placeholder="Enter command to execute..."
              disabled={isRunning}
              className="flex-1 font-mono"
            />
            <select
              value={language}
              onChange={(e) => setLanguage(e.target.value)}
              disabled={isRunning}
              className="px-3 py-2 bg-dark-secondary border border-dark-secondary rounded-lg text-gray-100 text-sm focus:border-neon-purple focus:outline-none"
            >
              <option value="bash">Bash</option>
              <option value="python">Python</option>
              <option value="javascript">JavaScript</option>
              <option value="rust">Rust</option>
            </select>
            <Button
              onClick={executeCommand}
              disabled={isRunning || !command.trim()}
              variant="primary"
              className="flex items-center space-x-2"
            >
              {isRunning ? (
                <>
                  <RefreshCw className="h-4 w-4 animate-spin" />
                  <span>Running...</span>
                </>
              ) : (
                <>
                  <Play className="h-4 w-4" />
                  <span>Execute</span>
                </>
              )}
            </Button>
            <Button
              onClick={clearResults}
              variant="ghost"
              disabled={isRunning}
            >
              Clear
            </Button>
          </div>

          {/* Quick Commands */}
          <div className="flex flex-wrap gap-2">
            <Button
              size="sm"
              variant="secondary"
              onClick={() => {
                setLanguage('bash')
                setCommand('echo "Available commands: help, status, scan, analyze"')
              }}
              disabled={isRunning}
            >
              <Code className="h-3 w-3 mr-1" />
              help
            </Button>
            <Button
              size="sm"
              variant="secondary"
              onClick={() => {
                setLanguage('bash')
                setCommand('echo "System Status: Online"')
              }}
              disabled={isRunning}
            >
              <Zap className="h-3 w-3 mr-1" />
              status
            </Button>
            <Button
              size="sm"
              variant="secondary"
              onClick={() => {
                setLanguage('bash')
                setCommand('echo "Security scan queued"')
              }}
              disabled={isRunning}
            >
              <AlertTriangle className="h-3 w-3 mr-1" />
              scan
            </Button>
            <Button
              size="sm"
              variant="secondary"
              onClick={() => {
                setLanguage('bash')
                setCommand('echo "Deep analysis started"')
              }}
              disabled={isRunning}
            >
              analyze
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Execution Results */}
      <Card>
        <CardHeader>
          <CardTitle>Execution History</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {results.length === 0 ? (
              <div className="text-center py-8 text-gray-400">
                <Terminal className="h-12 w-12 mx-auto mb-4 opacity-50" />
                <p>No commands executed yet</p>
                <p className="text-sm mt-2">Try running a command above to see results</p>
              </div>
            ) : (
              results.map((result) => (
                <div
                  key={result.id}
                  className={`border rounded-lg p-4 font-mono text-sm ${
                    result.status === 'error'
                      ? 'border-red-500/30 bg-red-500/5'
                      : result.status === 'running'
                      ? 'border-yellow-500/30 bg-yellow-500/5'
                      : 'border-green-500/30 bg-green-500/5'
                  }`}
                >
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center space-x-2">
                      <span className="text-gray-400">$</span>
                      <span className="text-gray-100">{result.command}</span>
                      {result.language && (
                        <span className="text-xs text-gray-500">[{result.language}]</span>
                      )}
                    </div>
                    <div className="flex items-center space-x-2">
                      {result.executionTime && (
                        <span className="text-xs text-gray-500">
                          {result.executionTime}ms
                        </span>
                      )}
                      {result.exitCode !== undefined && (
                        <span className="text-xs text-gray-500">exit {result.exitCode}</span>
                      )}
                      <span
                        className={`px-2 py-1 text-xs rounded ${
                          result.status === 'error'
                            ? 'bg-red-500/20 text-red-400'
                            : result.status === 'running'
                            ? 'bg-yellow-500/20 text-yellow-400'
                            : 'bg-green-500/20 text-green-400'
                        }`}
                      >
                        {result.status}
                      </span>
                    </div>
                  </div>
                  <pre className="text-gray-300 whitespace-pre-wrap">
                    {result.output || (
                      result.status === 'running' && (
                        <div className="flex items-center space-x-2">
                          <RefreshCw className="h-3 w-3 animate-spin" />
                          <span>Executing...</span>
                        </div>
                      )
                    )}
                  </pre>
                  {result.securityViolations && result.securityViolations.length > 0 && (
                    <div className="mt-2 text-xs text-red-300">
                      Security violations: {result.securityViolations.join(', ')}
                    </div>
                  )}
                  <div className="mt-2 text-xs text-gray-500">
                    {result.timestamp.toLocaleString()}
                  </div>
                </div>
              ))
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
