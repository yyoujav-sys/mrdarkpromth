import React from 'react'
import { AlertCircle, X, Info, CheckCircle, AlertTriangle } from 'lucide-react'
import { useUiStore } from '@/store/uiStore'

export const SystemBroadcastBanner: React.FC = () => {
    const { activeBroadcast, dismissBroadcast } = useUiStore()

    if (!activeBroadcast) return null

    const getIcon = () => {
        switch (activeBroadcast.severity) {
            case 'error': return <AlertCircle className="h-5 w-5 text-red-400" />
            case 'warning': return <AlertTriangle className="h-5 w-5 text-yellow-400" />
            case 'success': return <CheckCircle className="h-5 w-5 text-green-400" />
            default: return <Info className="h-5 w-5 text-blue-400" />
        }
    }

    const getColors = () => {
        switch (activeBroadcast.severity) {
            case 'error': return 'bg-red-900/50 border-red-900 text-red-200'
            case 'warning': return 'bg-yellow-900/50 border-yellow-900 text-yellow-200'
            case 'success': return 'bg-green-900/50 border-green-900 text-green-200'
            default: return 'bg-blue-900/50 border-blue-900 text-blue-200'
        }
    }

    return (
        <div className={`
      relative flex items-center gap-3 px-4 py-3 border-b ${getColors()}
    `}>
            <div className="flex-shrink-0">
                {getIcon()}
            </div>
            <div className="flex-1 min-w-0">
                <p className="text-sm font-medium">
                    {activeBroadcast.message}
                </p>
            </div>
            <div className="flex-shrink-0">
                <button
                    onClick={dismissBroadcast}
                    className="p-1 rounded-md hover:bg-black/20 transition-colors"
                >
                    <X className="h-4 w-4" />
                </button>
            </div>
        </div>
    )
}
