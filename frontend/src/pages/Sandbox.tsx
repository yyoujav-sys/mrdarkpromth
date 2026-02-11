import React, { useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Input } from '@/components/ui/Input'
import {
  Play,
  RefreshCw,
  Terminal,
  AlertTriangle,
  FileCode,
  FolderOpen,
  Copy,
  Check
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
  const [copiedId, setCopiedId] = useState<string | null>(null)
  const [virtualFiles] = useState([
    { name: 'main.py', size: '1.2kb', type: 'file' },
    { name: 'utils.js', size: '2.5kb', type: 'file' },
    { name: 'data/', size: '-', type: 'directory' },
    { name: 'data/config.json', size: '512b', type: 'file' }
  ])

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

  const copyToClipboard = (text: string, id: string) => {
    navigator.clipboard.writeText(text)
    setCopiedId(id)
    setTimeout(() => setCopiedId(null), 2000)
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-gray-100 italic tracking-tighter">AI_SANDBOX <span className="text-xs font-mono text-neon-purple px-2 py-0.5 bg-neon-purple/10 rounded ml-2">v2.1</span></h1>
        <p className="mt-2 text-gray-400">
          Neural-isolated execution environment for autonomous agents.
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        {/* Virtual File System Sidebar */}
        <div className="lg:col-span-1 space-y-6">
          <Card className="h-full">
            <CardHeader className="pb-3 border-b border-dark-accent">
              <CardTitle className="text-xs uppercase tracking-widest flex items-center gap-2">
                <FolderOpen className="h-4 w-4 text-neon-blue" />
                Virtual_FS
              </CardTitle>
            </CardHeader>
            <CardContent className="pt-4 px-2">
              <div className="space-y-1">
                {virtualFiles.map((file, idx) => (
                  <div key={idx} className="flex items-center justify-between px-3 py-2 rounded hover:bg-white/5 cursor-pointer group transition-colors">
                    <div className="flex items-center gap-2">
                      <FileCode className={`h-4 w-4 ${file.type === 'directory' ? 'text-neon-blue' : 'text-gray-500 group-hover:text-neon-purple'}`} />
                      <span className={`text-xs font-mono ${file.type === 'directory' ? 'text-neon-blue' : 'text-gray-300'}`}>{file.name}</span>
                    </div>
                    <span className="text-[10px] text-gray-600 font-mono italic">{file.size}</span>
                  </div>
                ))}
              </div>
              <div className="mt-6 pt-4 border-t border-dark-accent p-3">
                <div className="text-[10px] text-gray-500 uppercase tracking-widest mb-2 font-bold">Quota Usage</div>
                <div className="w-full bg-black/40 h-1.5 rounded-full overflow-hidden">
                  <div className="bg-neon-purple h-full w-[12%]" />
                </div>
                <div className="flex justify-between mt-1">
                  <span className="text-[9px] text-gray-600 italic">12.5 MB / 512 MB</span>
                  <span className="text-[9px] text-neon-purple font-bold">2.4%</span>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Main Interface */}
        <div className="lg:col-span-3 space-y-6">
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
                  className="flex-1 font-mono text-sm bg-black/50 border-dark-accent"
                />
                <select
                  value={language}
                  onChange={(e) => setLanguage(e.target.value)}
                  disabled={isRunning}
                  className="px-3 py-2 bg-dark-secondary border border-dark-accent rounded-lg text-gray-100 text-sm focus:border-neon-purple focus:outline-none cursor-pointer"
                >
                  <option value="bash">Bash</option>
                  <option value="python">Python</option>
                  <option value="javascript">Node.js</option>
                  <option value="rust">Rust</option>
                </select>
                <Button
                  onClick={executeCommand}
                  disabled={isRunning || !command.trim()}
                  variant="primary"
                  className="flex items-center space-x-2 px-6"
                >
                  {isRunning ? (
                    <>
                      <RefreshCw className="h-4 w-4 animate-spin" />
                      <span>...</span>
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
                  <RefreshCw className="h-4 w-4" />
                </Button>
              </div>

              {/* Quick Commands */}
              <div className="flex flex-wrap gap-2 pt-2">
                {[
                  { label: 'ls -R', cmd: 'ls -R', color: 'blue' },
                  { label: 'top', cmd: 'top -n 1', color: 'purple' },
                  { label: 'whoami', cmd: 'whoami', color: 'green' },
                  { label: 'netstat', cmd: 'netstat -tuln', color: 'red' }
                ].map((tag) => (
                  <Button
                    key={tag.label}
                    size="sm"
                    variant="secondary"
                    className="h-7 text-[10px] uppercase tracking-widest font-bold px-3 hover:border-white/20"
                    onClick={() => {
                      setLanguage('bash')
                      setCommand(tag.cmd)
                    }}
                    disabled={isRunning}
                  >
                    {tag.label}
                  </Button>
                ))}
              </div>
            </CardContent>
          </Card>

          {/* Execution Results */}
          <Card className="flex-1 bg-black/20">
            <CardHeader className="py-4 border-b border-dark-accent/50 flex flex-row items-center justify-between">
              <CardTitle className="text-sm uppercase tracking-widest opacity-60">Execution History</CardTitle>
              {results.length > 0 && <span className="text-[10px] font-mono text-neon-purple bg-neon-purple/10 px-2 py-0.5 rounded">{results.length} LOGS</span>}
            </CardHeader>
            <CardContent className="p-0">
              <div className="max-h-[600px] overflow-y-auto">
                {results.length === 0 ? (
                  <div className="text-center py-16 text-gray-500">
                    <Terminal className="h-12 w-12 mx-auto mb-4 opacity-20" />
                    <p className="text-sm font-mono italic">Awaiting neural input...</p>
                  </div>
                ) : (
                  <div className="divide-y divide-dark-accent/30">
                    {results.map((result) => (
                      <div
                        key={result.id}
                        className={`p-5 font-mono text-sm group relative ${result.status === 'error'
                          ? 'bg-red-500/[0.02]'
                          : result.status === 'running'
                            ? 'bg-yellow-500/[0.02]'
                            : 'bg-green-500/[0.02]'
                          }`}
                      >
                        <div className="flex items-center justify-between mb-3">
                          <div className="flex items-center space-x-3">
                            <span className="text-neon-purple font-bold">#</span>
                            <span className="text-gray-100 font-bold">{result.command}</span>
                            <span className="px-2 py-0.5 bg-white/5 rounded text-[10px] text-gray-500 uppercase tracking-widest">{result.language}</span>
                          </div>
                          <div className="flex items-center space-x-4">
                            {result.executionTime && (
                              <span className="text-[10px] text-gray-600 font-bold italic">
                                {result.executionTime}ms
                              </span>
                            )}
                            <div className="flex gap-2">
                              <button
                                onClick={() => copyToClipboard(result.output, result.id)}
                                className="p-1.5 rounded hover:bg-white/10 text-gray-500 transition-colors"
                              >
                                {copiedId === result.id ? <Check className="h-3 w-3 text-green-400" /> : <Copy className="h-3 w-3" />}
                              </button>
                              <span
                                className={`px-2 py-0.5 text-[10px] font-bold rounded uppercase tracking-widest ${result.status === 'error'
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
                        </div>
                        <div className="relative">
                          <pre className="text-gray-400 whitespace-pre-wrap text-xs leading-relaxed bg-black/40 p-4 rounded border border-white/5 max-h-96 overflow-y-auto">
                            {result.output || (
                              result.status === 'running' && (
                                <div className="flex items-center space-x-2 text-neon-blue">
                                  <RefreshCw className="h-3 w-3 animate-spin" />
                                  <span className="italic">CALCULATING_TRAJECTORY...</span>
                                </div>
                              )
                            )}
                          </pre>
                        </div>
                        {result.securityViolations && result.securityViolations.length > 0 && (
                          <div className="mt-3 p-2 bg-red-500/10 border border-red-500/30 rounded text-[10px] text-red-300 flex items-center gap-2">
                            <AlertTriangle className="h-3 w-3" />
                            SECURITY_ALERT: {result.securityViolations.join(', ')}
                          </div>
                        )}
                        <div className="mt-3 text-[10px] text-gray-600 italic">
                          TS: {result.timestamp.toISOString()}
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}

