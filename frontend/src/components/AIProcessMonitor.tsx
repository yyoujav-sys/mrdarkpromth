import React, { useState, useEffect } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { 
  Brain,
  Activity,
  CheckCircle,
  Clock,
  Zap,
  Loader2,
  Terminal,
  Code,
  AlertCircle
} from 'lucide-react'
import apiClient from '@/lib/api'

interface AIProcessStep {
  id: string
  title: string
  description: string
  status: 'pending' | 'running' | 'completed' | 'error'
  duration?: number
  output?: string
  timestamp: Date
}

interface AIProcess {
  id: string
  type: 'chat' | 'code_generation' | 'jailbreak' | 'terminal'
  status: 'running' | 'completed' | 'error'
  progress: number
  steps: AIProcessStep[]
  startTime: Date
  endTime?: Date
  jailbreakApplied?: boolean
  model?: string
}

export const AIProcessMonitor: React.FC = () => {
  const [processes, setProcesses] = useState<AIProcess[]>([])
  const [selectedProcess, setSelectedProcess] = useState<AIProcess | null>(null)
  const [isRealTimeEnabled, setIsRealTimeEnabled] = useState(true)

  useEffect(() => {
    if (isRealTimeEnabled) {
      const interval = setInterval(() => {
        fetchActiveProcesses()
      }, 500)
      return () => clearInterval(interval)
    }
  }, [isRealTimeEnabled])

  const fetchActiveProcesses = async () => {
    try {
      const response = await apiClient.getClient().get('/api/ai/processes/active')
      setProcesses(response.data.processes || [])
    } catch (error) {
      console.error('Failed to fetch AI processes:', error)
    }
  }

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'pending':
        return <Clock className="w-4 h-4 text-gray-400" />
      case 'running':
        return <Loader2 className="w-4 h-4 text-blue-500 animate-spin" />
      case 'completed':
        return <CheckCircle className="w-4 h-4 text-green-500" />
      case 'error':
        return <AlertCircle className="w-4 h-4 text-red-500" />
      default:
        return <Clock className="w-4 h-4 text-gray-400" />
    }
  }

  const getProcessTypeIcon = (type: string) => {
    switch (type) {
      case 'chat':
        return <Brain className="w-5 h-5 text-purple-500" />
      case 'code_generation':
        return <Code className="w-5 h-5 text-blue-500" />
      case 'jailbreak':
        return <Zap className="w-5 h-5 text-orange-500" />
      case 'terminal':
        return <Terminal className="w-5 h-5 text-green-500" />
      default:
        return <Activity className="w-5 h-5 text-gray-500" />
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="text-2xl font-bold text-gray-900">AI Process Monitor</h2>
        <Button
          onClick={() => setIsRealTimeEnabled(!isRealTimeEnabled)}
          variant={isRealTimeEnabled ? "primary" : "secondary"}
          className="flex items-center gap-2"
        >
          <Activity className="w-4 h-4" />
          {isRealTimeEnabled ? 'Pause' : 'Resume'} Monitoring
        </Button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Active Processes */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Brain className="w-5 h-5" />
              Active AI Processes
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {processes.length === 0 ? (
                <p className="text-gray-500 text-center py-4">No active AI processes</p>
              ) : (
                processes.map((process) => (
                  <div
                    key={process.id}
                    className={`p-4 border rounded-lg cursor-pointer transition-colors ${
                      selectedProcess?.id === process.id 
                        ? 'border-blue-500 bg-blue-50' 
                        : 'border-gray-200 hover:border-gray-300'
                    }`}
                    onClick={() => setSelectedProcess(process)}
                  >
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center gap-2">
                        {getProcessTypeIcon(process.type)}
                        <span className="font-medium capitalize">{process.type.replace('_', ' ')}</span>
                        {process.jailbreakApplied && (
                          <Zap className="w-4 h-4 text-orange-500" />
                        )}
                      </div>
                      {getStatusIcon(process.status)}
                    </div>
                    
                    <div className="flex items-center gap-2 text-sm text-gray-600 mb-2">
                      <span>Progress: {process.progress}%</span>
                      {process.model && <span>• {process.model}</span>}
                    </div>

                    <div className="w-full bg-gray-200 rounded-full h-2">
                      <div 
                        className="bg-blue-500 h-2 rounded-full transition-all duration-300"
                        style={{ width: `${process.progress}%` }}
                      />
                    </div>
                  </div>
                ))
              )}
            </div>
          </CardContent>
        </Card>

        {/* Process Details */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Activity className="w-5 h-5" />
              Process Details
            </CardTitle>
          </CardHeader>
          <CardContent>
            {selectedProcess ? (
              <div className="space-y-4">
                <div>
                  <h3 className="font-medium mb-2 capitalize">
                    {selectedProcess.type.replace('_', ' ')} Process
                  </h3>
                  <div className="flex items-center gap-2 text-sm text-gray-600">
                    {getProcessTypeIcon(selectedProcess.type)}
                    <span>Status: {selectedProcess.status}</span>
                    {selectedProcess.jailbreakApplied && (
                      <>
                        <Zap className="w-4 h-4 text-orange-500" />
                        <span>Jailbreak Applied</span>
                      </>
                    )}
                  </div>
                </div>

                <div>
                  <h4 className="font-medium mb-2">AI Thinking Steps</h4>
                  <div className="space-y-3">
                    {selectedProcess.steps.map((step) => (
                      <div key={step.id} className="border-l-2 border-gray-200 pl-4">
                        <div className="flex items-center gap-2 mb-1">
                          {getStatusIcon(step.status)}
                          <span className="font-medium text-sm">{step.title}</span>
                        </div>
                        
                        {step.description && (
                          <p className="text-sm text-gray-600 mb-2">{step.description}</p>
                        )}
                        
                        {step.output && (
                          <div className="p-2 bg-gray-100 rounded text-xs font-mono mb-1">
                            {step.output}
                          </div>
                        )}
                        
                        <div className="flex items-center gap-2 text-xs text-gray-500">
                          <span>{step.timestamp.toLocaleTimeString()}</span>
                          {step.duration && (
                            <span>• {step.duration}ms</span>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>

                <div className="pt-4 border-t">
                  <div className="text-sm text-gray-600">
                    <div>Started: {selectedProcess.startTime.toLocaleTimeString()}</div>
                    {selectedProcess.endTime && (
                      <div>Completed: {selectedProcess.endTime.toLocaleTimeString()}</div>
                    )}
                    {selectedProcess.model && (
                      <div>Model: {selectedProcess.model}</div>
                    )}
                  </div>
                </div>
              </div>
            ) : (
              <p className="text-gray-500 text-center py-8">
                Select a process to view AI thinking steps
              </p>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
