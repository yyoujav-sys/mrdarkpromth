import React, { createContext, useContext, useState, useCallback } from 'react'
import { AlertCircle, CheckCircle, Info, X } from 'lucide-react'

type ToastType = 'success' | 'error' | 'info' | 'warning'

interface Toast {
    id: string
    type: ToastType
    message: string
}

interface ToastContextType {
    toast: (message: string, type?: ToastType) => void
}

const ToastContext = createContext<ToastContextType | undefined>(undefined)

export const ToastProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    const [toasts, setToasts] = useState<Toast[]>([])

    const toast = useCallback((message: string, type: ToastType = 'info') => {
        const id = Math.random().toString(36).substr(2, 9)
        setToasts((prev) => [...prev, { id, type, message }])

        setTimeout(() => {
            setToasts((prev) => prev.filter((t) => t.id !== id))
        }, 5000)
    }, [])

    const removeToast = (id: string) => {
        setToasts((prev) => prev.filter((t) => t.id !== id))
    }

    return (
        <ToastContext.Provider value={{ toast }}>
            {children}
            <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 pointer-events-none">
                {toasts.map((t) => (
                    <div
                        key={t.id}
                        className={`
              pointer-events-auto min-w-[300px] max-w-md p-4 rounded-none border-l-4 shadow-lg backdrop-blur-md animate-in slide-in-from-right-full
              ${t.type === 'success' ? 'bg-black/90 border-neon-green text-neon-green' : ''}
              ${t.type === 'error' ? 'bg-black/90 border-neon-red text-neon-red' : ''}
              ${t.type === 'warning' ? 'bg-black/90 border-yellow-500 text-yellow-500' : ''}
              ${t.type === 'info' ? 'bg-black/90 border-neon-blue text-neon-blue' : ''}
            `}
                    >
                        <div className="flex items-start justify-between gap-3">
                            <div className="mt-0.5">
                                {t.type === 'success' && <CheckCircle className="w-5 h-5" />}
                                {t.type === 'error' && <AlertCircle className="w-5 h-5" />}
                                {t.type === 'warning' && <AlertCircle className="w-5 h-5" />}
                                {t.type === 'info' && <Info className="w-5 h-5" />}
                            </div>
                            <div className="flex-1 font-mono text-sm leading-tight">
                                <p className="font-bold mb-1 uppercase tracking-wider">{t.type}</p>
                                <p>{t.message}</p>
                            </div>
                            <button
                                onClick={() => removeToast(t.id)}
                                className="hover:bg-white/10 rounded p-1 transition-colors"
                                aria-label="Close notification"
                            >
                                <X className="w-4 h-4" />
                            </button>
                        </div>
                    </div>
                ))}
            </div>
        </ToastContext.Provider>
    )
}

export const useToast = () => {
    const context = useContext(ToastContext)
    if (context === undefined) {
        throw new Error('useToast must be used within a ToastProvider')
    }
    return context
}
