import React, { useEffect, useMemo, useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Input } from '@/components/ui/Input'
import { 
  Search, 
  Filter, 
  Wrench, 
  Shield, 
  Database,
  Globe,
  Code,
  Cpu,
  Lock,
  Activity
} from 'lucide-react'
import apiClient from '@/lib/api'

interface Tool {
  id: string
  name: string
  description: string
  category: string
  status: 'active' | 'inactive' | 'maintenance'
  usage?: number
  lastUsed?: Date
  icon: React.ElementType
  features: string[]
}

export const Tools: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('')
  const [selectedCategory, setSelectedCategory] = useState('all')
  const [showInactive, setShowInactive] = useState(false)
  const [tools, setTools] = useState<Tool[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)

  const iconMap: Record<string, React.ElementType> = {
    ai: Cpu,
    security: Shield,
    database: Database,
    network: Globe,
    development: Code,
    execution: Code,
    monitoring: Activity,
    file_operations: Wrench,
    encryption: Lock,
    default: Wrench
  }

  useEffect(() => {
    let isActive = true

    const loadTools = async () => {
      setIsLoading(true)
      setErrorMessage(null)

      try {
        const response = await apiClient.listTools()
        const toolList = Array.isArray(response.tools) ? response.tools : response.tools ?? []

        const mappedTools: Tool[] = toolList.map((tool: any, index: number) => {
          const category = tool.category || 'general'
          const icon = iconMap[category] ?? iconMap.default
          return {
            id: `${tool.name}-${index}`,
            name: tool.name,
            description: tool.description,
            category,
            status: tool.enabled ? 'active' : 'inactive',
            usage: undefined,
            lastUsed: undefined,
            icon,
            features: tool.required_permissions?.length
              ? tool.required_permissions
              : ['standard access']
          }
        })

        if (isActive) {
          setTools(mappedTools)
        }
      } catch (error) {
        if (isActive) {
          setErrorMessage('Unable to load tools. Please try again later.')
          setTools([])
        }
      } finally {
        if (isActive) {
          setIsLoading(false)
        }
      }
    }

    loadTools()

    return () => {
      isActive = false
    }
  }, [])

  const categories = useMemo(() => {
    const categoryMap = new Map<string, number>()
    tools.forEach((tool) => {
      categoryMap.set(tool.category, (categoryMap.get(tool.category) || 0) + 1)
    })

    const dynamicCategories = Array.from(categoryMap.entries()).map(([value, count]) => ({
      value,
      label: value.replace(/_/g, ' ').replace(/\b\w/g, (l) => l.toUpperCase()),
      count
    }))

    return [
      { value: 'all', label: 'All Tools', count: tools.length },
      ...dynamicCategories
    ]
  }, [tools])

  const filteredTools = useMemo(() => {
    return tools.filter(tool => {
      const matchesSearch = tool.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                           tool.description.toLowerCase().includes(searchTerm.toLowerCase())
      const matchesCategory = selectedCategory === 'all' || tool.category === selectedCategory
      const matchesStatus = showInactive || tool.status === 'active'
      
      return matchesSearch && matchesCategory && matchesStatus
    })
  }, [tools, searchTerm, selectedCategory, showInactive])

  const getStatusColor = (status: Tool['status']) => {
    switch (status) {
      case 'active': return 'text-green-400 bg-green-400/10'
      case 'inactive': return 'text-gray-400 bg-gray-400/10'
      case 'maintenance': return 'text-yellow-400 bg-yellow-400/10'
      default: return 'text-gray-400 bg-gray-400/10'
    }
  }

  const formatLastUsed = (date?: Date) => {
    if (!date) return 'N/A'
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    const minutes = Math.floor(diff / 60000)
    const hours = Math.floor(minutes / 60)
    const days = Math.floor(hours / 24)

    if (minutes < 1) return 'Just now'
    if (minutes < 60) return `${minutes}m ago`
    if (hours < 24) return `${hours}h ago`
    return `${days}d ago`
  }

  const formatUsage = (usage?: number) => {
    if (usage === undefined) return '—'
    return usage.toLocaleString()
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-gray-100">Tool Explorer</h1>
        <p className="mt-2 text-gray-400">
          Browse and manage available tools and services
        </p>
      </div>

      {errorMessage && (
        <Card>
          <CardContent className="p-4 text-sm text-red-300">
            {errorMessage}
          </CardContent>
        </Card>
      )}

      {/* Search and Filters */}
      <Card>
        <CardContent className="p-6">
          <div className="flex flex-col lg:flex-row gap-4">
            {/* Search */}
            <div className="flex-1">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
                <Input
                  placeholder="Search tools..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="pl-10"
                />
              </div>
            </div>

            {/* Category Filter */}
            <div className="flex gap-2 flex-wrap">
              {categories.map(category => (
                <Button
                  key={category.value}
                  variant={selectedCategory === category.value ? 'primary' : 'ghost'}
                  size="sm"
                  onClick={() => setSelectedCategory(category.value)}
                  className="flex items-center space-x-2"
                >
                  <span>{category.label}</span>
                  <span className="px-2 py-0.5 text-xs bg-dark-accent rounded-full">
                    {category.count}
                  </span>
                </Button>
              ))}
            </div>

            {/* Show Inactive Toggle */}
            <Button
              variant={showInactive ? 'primary' : 'ghost'}
              size="sm"
              onClick={() => setShowInactive(!showInactive)}
            >
              <Filter className="h-4 w-4 mr-2" />
              {showInactive ? 'Hide Inactive' : 'Show Inactive'}
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Tools Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {isLoading ? (
          <Card className="col-span-full">
            <CardContent className="py-12 text-center text-gray-400">
              Loading tools...
            </CardContent>
          </Card>
        ) : (
          filteredTools.map(tool => {
          const Icon = tool.icon
          return (
            <Card key={tool.id} className="hover:shadow-neon-purple/20 transition-all duration-200 hover:scale-[1.02]">
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div className="flex items-center space-x-3">
                    <div className="p-2 bg-neon-purple/10 rounded-lg">
                      <Icon className="h-6 w-6 text-neon-purple" />
                    </div>
                    <div>
                      <CardTitle className="text-lg">{tool.name}</CardTitle>
                      <span className={`inline-block px-2 py-1 text-xs rounded-full mt-1 ${getStatusColor(tool.status)}`}>
                        {tool.status}
                      </span>
                    </div>
                  </div>
                </div>
              </CardHeader>
              
              <CardContent className="space-y-4">
                <p className="text-gray-300 text-sm">{tool.description}</p>
                
                {/* Features */}
                <div className="space-y-2">
                  <h4 className="text-sm font-medium text-gray-200">Features</h4>
                  <div className="flex flex-wrap gap-1">
                    {tool.features.map((feature, index) => (
                      <span
                        key={index}
                        className="px-2 py-1 text-xs bg-dark-accent text-gray-300 rounded"
                      >
                        {feature}
                      </span>
                    ))}
                  </div>
                </div>

                {/* Stats */}
                <div className="flex items-center justify-between text-sm">
                  <div>
                    <span className="text-gray-400">Usage:</span>
                    <span className="ml-2 text-gray-200 font-medium">{formatUsage(tool.usage)}</span>
                  </div>
                  <div>
                    <span className="text-gray-400">Last used:</span>
                    <span className="ml-2 text-gray-200">{formatLastUsed(tool.lastUsed)}</span>
                  </div>
                </div>

                {/* Actions */}
                <div className="flex space-x-2">
                  <Button variant="primary" size="sm" className="flex-1">
                    <Wrench className="h-3 w-3 mr-1" />
                    Configure
                  </Button>
                  <Button variant="ghost" size="sm">
                    View Details
                  </Button>
                </div>
              </CardContent>
            </Card>
          )
        })
        )}
      </div>

      {/* No Results */}
      {filteredTools.length === 0 && (
        <Card>
          <CardContent className="text-center py-12">
            <Wrench className="h-12 w-12 mx-auto mb-4 text-gray-400" />
            <h3 className="text-lg font-medium text-gray-200 mb-2">No tools found</h3>
            <p className="text-gray-400">
              Try adjusting your search or filters to find what you're looking for.
            </p>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
