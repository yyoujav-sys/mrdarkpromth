import * as vscode from 'vscode';
import { createClient, RedisClientType } from 'redis';

export interface CoordinationEvent {
    event_id: string;
    agent_id: string;
    event_type: 'task_completion' | 'resource_ready' | 'error_event' | 'query_event' | 'response_event' | 'heartbeat' | 'shutdown';
    timestamp: string;
    correlation_id?: string;
    payload: any;
}

export class EventBusCoordinator {
    private client: RedisClientType;
    private isConnected: boolean = false;
    private agentId: string = 'agent10';
    private subscriptions: Map<string, (event: CoordinationEvent) => void> = new Map();
    private heartbeatInterval: NodeJS.Timeout | null = null;

    constructor(private redisUrl: string = 'redis://localhost:6379') {
        this.client = createClient({
            url: this.redisUrl
        });

        this.client.on('error', (err) => {
            console.error('Redis Client Error:', err);
            this.isConnected = false;
        });

        this.client.on('connect', () => {
            console.log('Redis Client Connected');
            this.isConnected = true;
        });
    }

    async connect(): Promise<void> {
        try {
            await this.client.connect();
            await this.setupSubscriptions();
            this.startHeartbeat();
        } catch (error) {
            console.error('Failed to connect to Redis:', error);
            throw error;
        }
    }

    async disconnect(): Promise<void> {
        if (this.heartbeatInterval) {
            clearInterval(this.heartbeatInterval);
        }
        
        await this.publishEvent('shutdown', { agent: this.agentId });
        await this.client.disconnect();
        this.isConnected = false;
    }

    async publishEvent(eventType: string, payload: any, correlationId?: string): Promise<void> {
        if (!this.isConnected) {
            console.warn('Redis not connected, skipping event publish');
            return;
        }

        const event: CoordinationEvent = {
            event_id: this.generateUUID(),
            agent_id: this.agentId,
            event_type: eventType as any,
            timestamp: new Date().toISOString(),
            correlation_id: correlationId,
            payload
        };

        const streamKey = `mr_darkpromth:${eventType === 'query_event' || eventType === 'response_event' ? this.agentId : 'global'}:${eventType}`;
        
        try {
            await this.client.xAdd(streamKey, '*', {
                event_id: event.event_id,
                agent_id: event.agent_id,
                event_type: event.event_type,
                timestamp: event.timestamp,
                correlation_id: event.correlation_id || '',
                payload: JSON.stringify(event.payload)
            });
        } catch (error) {
            console.error('Failed to publish event:', error);
        }
    }

    async queryAgent(targetAgentId: string, query: any, timeoutMs: number = 5000): Promise<any> {
        const correlationId = this.generateUUID();
        
        await this.publishEvent('query_event', query, correlationId);
        
        return new Promise((resolve, reject) => {
            const timeout = setTimeout(() => {
                reject(new Error(`Query timeout after ${timeoutMs}ms`));
            }, timeoutMs);

            const responseHandler = (event: CoordinationEvent) => {
                if (event.correlation_id === correlationId) {
                    clearTimeout(timeout);
                    this.unsubscribe(`${targetAgentId}:response_event`, responseHandler);
                    resolve(event.payload);
                }
            };

            this.subscribe(`${targetAgentId}:response_event`, responseHandler);
        });
    }

    private async setupSubscriptions(): Promise<void> {
        const streams = [
            'mr_darkpromth:global:task_completion',
            'mr_darkpromth:global:error_event',
            'mr_darkpromth:agent2:resource_ready',
            'mr_darkpromth:agent10:query_event'
        ];

        for (const stream of streams) {
            try {
                await this.client.xGroupCreate(stream, 'agent10_group', '0', { MKSTREAM: true });
            } catch (error) {
                // Group might already exist, ignore error
            }
        }

        this.startEventConsumer();
    }

    private async startEventConsumer(): Promise<void> {
        const streams = [
            { key: 'mr_darkpromth:global:task_completion', id: '>' },
            { key: 'mr_darkpromth:global:error_event', id: '>' },
            { key: 'mr_darkpromth:agent2:resource_ready', id: '>' },
            { key: 'mr_darkpromth:agent10:query_event', id: '>' }
        ];

        while (this.isConnected) {
            try {
                const results = await this.client.xReadGroup(
                    'agent10_group',
                    'consumer1',
                    streams,
                    { COUNT: 1, BLOCK: 1000 }
                );

                if (results && results.length > 0) {
                    for (const stream of results) {
                        for (const message of stream.messages) {
                            await this.processMessage(stream.name, message);
                            await this.client.xAck(stream.name, 'agent10_group', message.id);
                        }
                    }
                }
            } catch (error) {
                if (this.isConnected) {
                    console.error('Error in event consumer:', error);
                    await new Promise(resolve => setTimeout(resolve, 1000));
                }
            }
        }
    }

    private async processMessage(streamKey: string, message: any): Promise<void> {
        try {
            const event: CoordinationEvent = {
                event_id: message.message.event_id,
                agent_id: message.message.agent_id,
                event_type: message.message.event_type,
                timestamp: message.message.timestamp,
                correlation_id: message.message.correlation_id || undefined,
                payload: JSON.parse(message.message.payload)
            };

            const subscriptionKey = streamKey.replace('mr_darkpromth:', '').replace('global:', '').replace('agent10:', '');
            const handler = this.subscriptions.get(subscriptionKey);
            
            if (handler) {
                handler(event);
            }

            // Handle specific coordination logic
            switch (event.event_type) {
                case 'resource_ready':
                    if (event.payload.resource === 'api_gateway') {
                        console.log('API Gateway is ready - extension can connect');
                        vscode.window.showInformationMessage('MR.DarkPromth API Gateway is ready!');
                    }
                    break;
                    
                case 'error_event':
                    console.error(`Error from ${event.agent_id}:`, event.payload);
                    vscode.window.showErrorMessage(`Error from ${event.agent_id}: ${event.payload.message}`);
                    break;
                    
                case 'query_event':
                    await this.handleQuery(event);
                    break;
            }
        } catch (error) {
            console.error('Error processing message:', error);
        }
    }

    private async handleQuery(event: CoordinationEvent): Promise<void> {
        // Handle queries from other agents
        let response: any = {};

        switch (event.payload.query) {
            case 'extension_status':
                response = {
                    status: 'active',
                    version: '1.0.0',
                    features: ['chat_interface', 'user_info_display', 'authentication']
                };
                break;
                
            case 'user_authenticated':
                // This would need to be injected from the extension context
                response = { authenticated: false }; // Default response
                break;
                
            default:
                response = { error: 'Unknown query' };
        }

        await this.publishEvent('response_event', response, event.correlation_id);
    }

    subscribe(streamKey: string, handler: (event: CoordinationEvent) => void): void {
        this.subscriptions.set(streamKey, handler);
    }

    unsubscribe(streamKey: string, handler: (event: CoordinationEvent) => void): void {
        this.subscriptions.delete(streamKey);
    }

    private startHeartbeat(): void {
        this.heartbeatInterval = setInterval(async () => {
            await this.publishEvent('heartbeat', {
                status: 'healthy',
                tasks_in_progress: 0,
                tasks_completed: 1, // Extension setup completed
                memory_usage_mb: 50 // Estimated memory usage
            });
        }, 30000); // Every 30 seconds
    }

    private generateUUID(): string {
        return Date.now().toString(36) + Math.random().toString(36).substr(2);
    }

    isRedisConnected(): boolean {
        return this.isConnected;
    }
}
