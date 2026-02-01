import React, { useState, useEffect } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { 
  Brain, 
  Zap, 
  CheckCircle, 
  Clock, 
  Terminal, 
  Code, 
  AlertCircle,
  Activity,
  Loader2,
  Play,
  Pause
} from 'lucide-react'
import apiClient from '@/lib/api'

interface TaskStep {
  id: string
  title: string
  status: 'pending' | 'running' | 'completed' | 'error'
  description?: string
  duration?: number
  output?: string
}

interface Task {
  id: string
  title: string
  type: 'chat' | 'terminal' | 'code_generation' | 'jailbreak'
  status: 'pending' | 'running' | 'completed' | 'error'
  progress: number
  steps: TaskStep[]
  startTime: Date
  endTime?: Date
  userTier: string
  jailbreakApplied?: boolean
}

export const TaskMonitor: React.FC = () => {
  const [tasks, setTasks] = useState<Task[]>([])
  const [selectedTask, setSelectedTask] = useState<Task | null>(null)
  const [isRealTimeEnabled, setIsRealTimeEnabled] = useState(true)

  useEffect(() => {
    if (isRealTimeEnabled) {
      const interval = setInterval(() => {
        fetchTasks()
      }, 1000)
      return () => clearInterval(interval)
    }
  }, [isRealTimeEnabled])

  const fetchTasks = async () => {
    try {
      const response = await apiClient.getClient().get('/api/tasks/active')
      setTasks(response.data.tasks || [])
    } catch (error) {
      console.error('Failed to fetch tasks:', error)
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

  const getTaskIcon = (type: string) => {
    switch (type) {
      case 'chat':
        return <Brain className="w-5 h-5 text-purple-500" />
      case 'terminal':
        return <Terminal className="w-5 h-5 text-green-500" />
      case 'code_generation':
        return <Code className="w-5 h-5 text-blue-500" />
      case 'jailbreak':
        return <Zap className="w-5 h-5 text-orange-500" />
      default:
        return <Activity className="w-5 h-5 text-gray-500" />
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="text-2xl font-bold text-gray-900">Task Monitor</h2>
        <Button
          onClick={() => setIsRealTimeEnabled(!isRealTimeEnabled)}
          variant={isRealTimeEnabled ? "primary" : "secondary"}
          className="flex items-center gap-2"
        >
          {isRealTimeEnabled ? <Pause className="w-4 h-4" /> : <Play className="w-4 h-4" />}
          {isRealTimeEnabled ? 'Pause' : 'Resume'} Real-time
        </Button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Active Tasks List */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Activity className="w-5 h-5" />
              Active Tasks
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {tasks.length === 0 ? (
                <p className="text-gray-500 text-center py-4">No active tasks</p>
              ) : (
                tasks.map((task) => (
                  <div
                    key={task.id}
                    className={`p-4 border rounded-lg cursor-pointer transition-colors ${
                      selectedTask?.id === task.id 
                        ? 'border-blue-500 bg-blue-50' 
                        : 'border-gray-200 hover:border-gray-300'
                    }`}
                    onClick={() => setSelectedTask(task)}
                  >
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center gap-2">
                        {getTaskIcon(task.type)}
                        <span className="font-medium">{task.title}</span>
                        {task.jailbreakApplied && (
                          <Zap className="w-4 h-4 text-orange-500" />
                        )}
                      </div>
                      {getStatusIcon(task.status)}
                    </div>
                    
                    <div className="flex items-center gap-2 text-sm text-gray-600 mb-2">
                      <span>Tier: {task.userTier}</span>
                      <span>•</span>
                      <span>{task.progress}%</span>
                    </div>

                    <div className="w-full bg-gray-200 rounded-full h-2">
                      <div 
                        className="bg-blue-500 h-2 rounded-full transition-all duration-300"
                        style={{ width: `${task.progress}%` }}
                      />
                    </div>
                  </div>
                ))
              )}
            </div>
          </CardContent>
        </Card>

        {/* Task Details */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Brain className="w-5 h-5" />
              Task Details
            </CardTitle>
          </CardHeader>
          <CardContent>
            {selectedTask ? (
              <div className="space-y-4">
                <div>
                  <h3 className="font-medium mb-2">{selectedTask.title}</h3>
                  <div className="flex items-center gap-2 text-sm text-gray-600">
                    {getTaskIcon(selectedTask.type)}
                    <span>{selectedTask.type}</span>
                    {selectedTask.jailbreakApplied && (
                      <>
                        <Zap className="w-4 h-4 text-orange-500" />
                        <span>Jailbreak Applied</span>
                      </>
                    )}
                  </div>
                </div>

                <div>
                  <h4 className="font-medium mb-2">Execution Steps</h4>
                  <div className="space-y-2">
                    {selectedTask.steps.map((step) => (
                      <div key={step.id} className="flex items-start gap-3">
                        <div className="mt-1">
                          {getStatusIcon(step.status)}
                        </div>
                        <div className="flex-1">
                          <div className="font-medium text-sm">{step.title}</div>
                          {step.description && (
                            <div className="text-sm text-gray-600">{step.description}</div>
                          )}
                          {step.output && (
                            <div className="mt-1 p-2 bg-gray-100 rounded text-xs font-mono">
                              {step.output}
                            </div>
                          )}
                          {step.duration && (
                            <div className="text-xs text-gray-500">
                              Duration: {step.duration}ms
                            </div>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>

                <div className="pt-4 border-t">
                  <div className="text-sm text-gray-600">
                    <div>Started: {selectedTask.startTime.toLocaleTimeString()}</div>
                    {selectedTask.endTime && (
                      <div>Completed: {selectedTask.endTime.toLocaleTimeString()}</div>
                    )}
                  </div>
                </div>
              </div>
            ) : (
              <p className="text-gray-500 text-center py-8">
                Select a task to view details
              </p>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
