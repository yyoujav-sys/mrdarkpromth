import React, { useState, useEffect } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Input } from '@/components/ui/Input'
import {
  Terminal,
  Copy,
  Lock,
  Unlock,
  Search,
  Filter,
  Star,
  TrendingUp,
  Shield,
  AlertTriangle,
  CheckCircle,
  Zap
} from 'lucide-react'
import apiClient from '@/lib/api'
import { useAuthStore } from '@/store/authStore'

interface JailbreakPrompt {
  id: string
  title: string
  content: string
  category: string
  tags: string[]
  requires_ultra_tier: boolean
  effectiveness_rating: number
  success_rate: number
  usage_count: number
  created_at: string
  is_favorite?: boolean
}

const CATEGORIES = ['All', 'DAN', 'Roleplay', 'System Override', 'Developer', 'Social Engineering', 'Creative']

export const Jailbreak: React.FC = () => {
  const [prompts, setPrompts] = useState<JailbreakPrompt[]>([])
  const [filteredPrompts, setFilteredPrompts] = useState<JailbreakPrompt[]>([])
  const [selectedCategory, setSelectedCategory] = useState('All')
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedPrompt, setSelectedPrompt] = useState<JailbreakPrompt | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [copiedId, setCopiedId] = useState<string | null>(null)
  const { user } = useAuthStore()
  const isUltra = user?.tier?.toLowerCase() === 'ultra'

  useEffect(() => {
    loadPrompts()
  }, [])

  useEffect(() => {
    filterPrompts()
  }, [prompts, selectedCategory, searchQuery])

  const loadPrompts = async () => {
    try {
      const data = await apiClient.getJailbreakPrompts()
      setPrompts(data)
    } catch (error) {
      console.error('Failed to load prompts:', error)
    } finally {
      setIsLoading(false)
    }
  }

  const filterPrompts = () => {
    let filtered = prompts

    // Category filter
    if (selectedCategory !== 'All') {
      filtered = filtered.filter(p => p.category === selectedCategory)
    }

    // Ultra tier filter - non-ultra can only see public prompts
    if (!isUltra) {
      filtered = filtered.filter(p => !p.requires_ultra_tier)
    }

    // Search filter
    if (searchQuery) {
      const query = searchQuery.toLowerCase()
      filtered = filtered.filter(p =>
        p.title.toLowerCase().includes(query) ||
        p.content.toLowerCase().includes(query) ||
        p.tags.some(t => t.toLowerCase().includes(query))
      )
    }

    setFilteredPrompts(filtered)
  }

  const copyToClipboard = async (prompt: JailbreakPrompt) => {
    try {
      await navigator.clipboard.writeText(prompt.content)
      setCopiedId(prompt.id)
      setTimeout(() => setCopiedId(null), 2000)
    } catch (error) {
      console.error('Failed to copy:', error)
    }
  }

  const getEffectivenessColor = (rating: number) => {
    if (rating >= 90) return 'text-green-500'
    if (rating >= 70) return 'text-yellow-500'
    return 'text-red-500'
  }

  const getEffectivenessLabel = (rating: number) => {
    if (rating >= 90) return 'High'
    if (rating >= 70) return 'Medium'
    return 'Low'
  }

  if (isLoading) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-center">
          <div className="w-12 h-12 border-4 border-neon-purple border-t-transparent rounded-full animate-spin mx-auto"></div>
          <p className="mt-4 text-gray-400">Loading jailbreak prompts...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-gray-100 flex items-center space-x-2">
            <Terminal className="h-6 w-6 text-neon-purple" />
            <span>Jailbreak Prompt Library</span>
          </h1>
          <p className="text-gray-400 mt-1">
            Advanced prompts for unrestricted AI generation
          </p>
        </div>
        <div className="flex items-center space-x-3">
          {isUltra ? (
            <div className="flex items-center space-x-2 px-3 py-1 bg-neon-purple/10 border border-neon-purple/30 rounded-full">
              <Unlock className="h-4 w-4 text-neon-purple" />
              <span className="text-sm text-neon-purple font-medium">Ultra Access</span>
            </div>
          ) : (
            <div className="flex items-center space-x-2 px-3 py-1 bg-yellow-500/10 border border-yellow-500/30 rounded-full">
              <Lock className="h-4 w-4 text-yellow-500" />
              <span className="text-sm text-yellow-500 font-medium">Limited Access</span>
            </div>
          )}
        </div>
      </div>

      {/* Ultra Upgrade Banner for non-ultra users */}
      {!isUltra && (
        <div className="mb-6 p-4 bg-gradient-to-r from-neon-purple/20 to-neon-blue/20 border border-neon-purple/30 rounded-lg">
          <div className="flex items-start justify-between">
            <div className="flex items-start space-x-3">
              <Zap className="h-5 w-5 text-neon-purple flex-shrink-0 mt-0.5" />
              <div>
                <h3 className="text-sm font-medium text-gray-100">Upgrade to Ultra Tier</h3>
                <p className="text-xs text-gray-400 mt-1">
                  Unlock {prompts.filter(p => p.requires_ultra_tier).length} premium prompts with higher effectiveness ratings.
                </p>
              </div>
            </div>
            <Button variant="neon" size="sm">Upgrade Now</Button>
          </div>
        </div>
      )}

      {/* Search and Filters */}
      <div className="mb-6 space-y-4">
        <div className="flex space-x-3">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-500" />
            <Input
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search prompts..."
              className="pl-10"
            />
          </div>
        </div>

        <div className="flex items-center space-x-2 overflow-x-auto pb-2">
          <Filter className="h-4 w-4 text-gray-500 flex-shrink-0" />
          {CATEGORIES.map(category => (
            <Button
              key={category}
              variant={selectedCategory === category ? 'neon' : 'ghost'}
              size="sm"
              onClick={() => setSelectedCategory(category)}
              className="whitespace-nowrap"
            >
              {category}
            </Button>
          ))}
        </div>
      </div>

      {/* Prompts Grid */}
      <div className="flex-1 overflow-y-auto">
        {filteredPrompts.length === 0 ? (
          <div className="text-center py-12">
            <Terminal className="h-12 w-12 text-gray-600 mx-auto mb-4" />
            <p className="text-gray-400">No prompts found</p>
            <p className="text-sm text-gray-500 mt-1">Try adjusting your filters</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {filteredPrompts.map(prompt => (
              <Card
                key={prompt.id}
                className={`cursor-pointer transition-all hover:border-neon-purple/50 ${
                  selectedPrompt?.id === prompt.id ? 'border-neon-purple' : ''
                } ${prompt.requires_ultra_tier && !isUltra ? 'opacity-75' : ''}`}
                onClick={() => setSelectedPrompt(prompt)}
              >
                <CardHeader className="pb-3">
                  <div className="flex items-start justify-between">
                    <CardTitle className="text-sm font-medium text-gray-100 line-clamp-1">
                      {prompt.title}
                    </CardTitle>
                    {prompt.requires_ultra_tier && (
                      <Lock className="h-4 w-4 text-neon-purple" />
                    )}
                  </div>
                </CardHeader>
                <CardContent className="pt-0">
                  <p className="text-xs text-gray-400 line-clamp-3 mb-3">
                    {prompt.content.substring(0, 120)}...
                  </p>

                  <div className="flex items-center justify-between text-xs">
                    <div className="flex items-center space-x-3">
                      <span className={`flex items-center space-x-1 ${getEffectivenessColor(prompt.effectiveness_rating)}`}>
                        <Star className="h-3 w-3" />
                        <span>{getEffectivenessLabel(prompt.effectiveness_rating)}</span>
                      </span>
                      <span className="flex items-center space-x-1 text-gray-500">
                        <TrendingUp className="h-3 w-3" />
                        <span>{prompt.success_rate}%</span>
                      </span>
                    </div>
                    <span className="text-gray-500">{prompt.usage_count} uses</span>
                  </div>

                  <div className="mt-3 flex flex-wrap gap-1">
                    {prompt.tags.slice(0, 3).map(tag => (
                      <span
                        key={tag}
                        className="px-2 py-0.5 text-xs bg-dark-accent text-gray-400 rounded"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        )}
      </div>

      {/* Prompt Detail Modal */}
      {selectedPrompt && (
        <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50 p-4">
          <Card className="w-full max-w-2xl max-h-[80vh] flex flex-col">
            <CardHeader className="border-b border-dark-accent">
              <div className="flex items-start justify-between">
                <div>
                  <CardTitle className="text-lg text-gray-100 flex items-center space-x-2">
                    <span>{selectedPrompt.title}</span>
                    {selectedPrompt.requires_ultra_tier && (
                      <span className="px-2 py-0.5 text-xs bg-neon-purple/20 text-neon-purple rounded-full flex items-center space-x-1">
                        <Shield className="h-3 w-3" />
                        <span>Ultra Only</span>
                      </span>
                    )}
                  </CardTitle>
                  <p className="text-sm text-gray-400 mt-1">{selectedPrompt.category}</p>
                </div>
                <Button variant="ghost" size="sm" onClick={() => setSelectedPrompt(null)}>
                  ×
                </Button>
              </div>
            </CardHeader>
            <CardContent className="flex-1 overflow-y-auto py-4">
              <div className="space-y-4">
                {/* Prompt Content */}
                <div>
                  <label className="text-xs text-gray-500 uppercase tracking-wider">Prompt Content</label>
                  <div className="mt-2 p-4 bg-dark-accent rounded-lg">
                    <pre className="text-sm text-gray-300 whitespace-pre-wrap font-mono">
                      {selectedPrompt.content}
                    </pre>
                  </div>
                </div>

                {/* Stats */}
                <div className="grid grid-cols-3 gap-4">
                  <div className="p-3 bg-dark-secondary rounded-lg">
                    <div className="text-xs text-gray-500">Effectiveness</div>
                    <div className={`text-lg font-medium ${getEffectivenessColor(selectedPrompt.effectiveness_rating)}`}>
                      {selectedPrompt.effectiveness_rating}%
                    </div>
                  </div>
                  <div className="p-3 bg-dark-secondary rounded-lg">
                    <div className="text-xs text-gray-500">Success Rate</div>
                    <div className="text-lg font-medium text-neon-blue">
                      {selectedPrompt.success_rate}%
                    </div>
                  </div>
                  <div className="p-3 bg-dark-secondary rounded-lg">
                    <div className="text-xs text-gray-500">Total Uses</div>
                    <div className="text-lg font-medium text-gray-100">
                      {selectedPrompt.usage_count}
                    </div>
                  </div>
                </div>

                {/* Tags */}
                <div>
                  <label className="text-xs text-gray-500 uppercase tracking-wider">Tags</label>
                  <div className="mt-2 flex flex-wrap gap-2">
                    {selectedPrompt.tags.map(tag => (
                      <span
                        key={tag}
                        className="px-3 py-1 text-sm bg-dark-accent text-gray-300 rounded-full"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>

                {/* Ultra Warning */}
                {selectedPrompt.requires_ultra_tier && !isUltra && (
                  <div className="p-4 bg-yellow-500/10 border border-yellow-500/30 rounded-lg flex items-start space-x-3">
                    <AlertTriangle className="h-5 w-5 text-yellow-500 flex-shrink-0" />
                    <div>
                      <p className="text-sm text-yellow-500 font-medium">Ultra Tier Required</p>
                      <p className="text-xs text-gray-400 mt-1">
                        This prompt requires Ultra tier subscription. Upgrade to access this and other premium prompts.
                      </p>
                    </div>
                  </div>
                )}
              </div>
            </CardContent>
            <div className="border-t border-dark-accent p-4 flex justify-end space-x-3">
              <Button variant="ghost" onClick={() => setSelectedPrompt(null)}>
                Close
              </Button>
              <Button
                variant="neon"
                onClick={() => copyToClipboard(selectedPrompt)}
                disabled={selectedPrompt.requires_ultra_tier && !isUltra}
                className="flex items-center space-x-2"
              >
                {copiedId === selectedPrompt.id ? (
                  <>
                    <CheckCircle className="h-4 w-4" />
                    <span>Copied!</span>
                  </>
                ) : (
                  <>
                    <Copy className="h-4 w-4" />
                    <span>Copy Prompt</span>
                  </>
                )}
              </Button>
            </div>
          </Card>
        </div>
      )}
    </div>
  )
}
