import { create } from 'zustand'

export interface SystemBroadcast {
    id: string
    type: string
    message: string
    severity: 'info' | 'warning' | 'error' | 'success'
    timestamp: Date
}

interface UiState {
    broadcasts: SystemBroadcast[]
    activeBroadcast: SystemBroadcast | null
    onlineUsers: Set<string>

    addBroadcast: (broadcast: Omit<SystemBroadcast, 'id' | 'timestamp'>) => void
    dismissBroadcast: () => void
    clearBroadcasts: () => void
    setOnlineStatus: (userId: string, isOnline: boolean) => void
}

export const useUiStore = create<UiState>((set) => ({
    broadcasts: [],
    activeBroadcast: null,
    onlineUsers: new Set(),

    addBroadcast: (broadcast) => {
        const newBroadcast: SystemBroadcast = {
            ...broadcast,
            id: Date.now().toString(),
            timestamp: new Date()
        }

        set((state) => ({
            broadcasts: [newBroadcast, ...state.broadcasts],
            activeBroadcast: newBroadcast
        }))
    },

    dismissBroadcast: () => {
        set({ activeBroadcast: null })
    },

    clearBroadcasts: () => {
        set({ broadcasts: [], activeBroadcast: null })
    },

    setOnlineStatus: (userId: string, isOnline: boolean) => {
        set((state) => {
            const newOnlineUsers = new Set(state.onlineUsers)
            if (isOnline) {
                newOnlineUsers.add(userId)
            } else {
                newOnlineUsers.delete(userId)
            }
            return { onlineUsers: newOnlineUsers }
        })
    }
}))
