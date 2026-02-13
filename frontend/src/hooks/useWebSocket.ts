import { useEffect, useRef, useState } from 'react'
import { useAuthStore } from '../store/authStore'
import { useUiStore } from '../store/uiStore'
import { useChatStore } from '../store/chatStore'

// Event types from backend
type WsEventType =
    | 'session_sync'
    | 'chat_notification'
    | 'system_broadcast'
    | 'user_status_changed'
    | 'heartbeat'
    | 'terminal_update'
    | 'pong'

interface WsEvent {
    type: WsEventType
    [key: string]: any
}

export const useWebSocket = () => {
    const { token, isAuthenticated } = useAuthStore()
    const { addBroadcast, setOnlineStatus } = useUiStore()
    const { handleNotification } = useChatStore()
    const socketRef = useRef<WebSocket | null>(null)
    const reconnectTimeoutRef = useRef<NodeJS.Timeout | undefined>(undefined)
    const [isConnected, setIsConnected] = useState(false)
    const [reconnectAttempts, setReconnectAttempts] = useState(0)

    const connect = () => {
        if (!token || !isAuthenticated) return

        if (socketRef.current) {
            socketRef.current.close()
        }

        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
        const host = window.location.host
        const wsUrl = `${protocol}//${host}/api/ws/events`

        console.log('Connecting to WebSocket...', wsUrl)

        // Use the Sec-WebSocket-Protocol trick for authentication
        const ws = new WebSocket(wsUrl, ['access_token', token])

        ws.onopen = () => {
            console.log('WebSocket connected')
            setIsConnected(true)
            setReconnectAttempts(0)
        }

        ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data) as WsEvent
                handleEvent(data)
            } catch (e) {
                console.error('Failed to parse WS message', e)
            }
        }

        ws.onclose = () => {
            console.log('WebSocket disconnected')
            setIsConnected(false)
            scheduleReconnect()
        }

        ws.onerror = (error) => {
            console.error('WebSocket error', error)
            ws.close()
        }

        socketRef.current = ws
    }

    const scheduleReconnect = () => {
        if (reconnectTimeoutRef.current) clearTimeout(reconnectTimeoutRef.current)

        const delay = Math.min(1000 * Math.pow(2, reconnectAttempts), 30000)
        reconnectTimeoutRef.current = setTimeout(() => {
            setReconnectAttempts(prev => prev + 1)
            connect()
        }, delay)
    }

    const handleEvent = (event: WsEvent) => {
        switch (event.type) {
            case 'system_broadcast':
                addBroadcast({
                    type: event.event_type || 'system',
                    message: event.payload?.message || 'System Update',
                    severity: event.payload?.severity || 'info'
                })
                break

            case 'chat_notification':
                handleNotification({
                    session_id: event.session_id,
                    message_preview: event.message_preview,
                    role: event.role
                })
                break

            case 'user_status_changed':
                setOnlineStatus(event.user_id, event.is_online)
                console.log(`User ${event.username} status: ${event.is_online ? 'online' : 'offline'}`)
                break

            case 'heartbeat':
                // console.debug('WS Heartbeat received', event.server_time)
                break

            case 'session_sync':
                console.log('WS Session sync received', event.active_sessions?.length || 0, 'sessions')
                break

            case 'terminal_update':
                console.log('WS Terminal update received', event.session_id, event.status)
                break

            default:
                console.log('Unhandled WS event:', event)
        }
    }

    useEffect(() => {
        if (isAuthenticated && token) {
            connect()
        } else {
            socketRef.current?.close()
        }

        return () => {
            socketRef.current?.close()
            if (reconnectTimeoutRef.current) clearTimeout(reconnectTimeoutRef.current)
        }
    }, [isAuthenticated, token])

    return { isConnected }
}
