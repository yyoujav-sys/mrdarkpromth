import * as vscode from 'vscode';
import { ApiClient } from './apiClient';
import { AuthManager } from './authManager';

export interface TerminalSession {
    sessionId: string;
    userId: string;
    tier: string;
    wsUrl: string;
    connected: boolean;
}

export interface TerminalOutput {
    type: 'stdout' | 'stderr' | 'status' | 'resource';
    data: string;
    timestamp: number;
}

export class UltraTerminalProvider {
    private context: vscode.ExtensionContext;
    private apiClient: ApiClient;
    private authManager: AuthManager;
    private websocket: any = null;
    private session: TerminalSession | null = null;
    private outputChannel: vscode.OutputChannel;
    private _onDidReceiveOutput: vscode.EventEmitter<TerminalOutput>;
    public readonly onDidReceiveOutput: vscode.Event<TerminalOutput>;

    constructor(context: vscode.ExtensionContext, apiClient: ApiClient, authManager: AuthManager) {
        this.context = context;
        this.apiClient = apiClient;
        this.authManager = authManager;
        this.outputChannel = vscode.window.createOutputChannel('MR.DarkPromth Ultra Terminal');
        this._onDidReceiveOutput = new vscode.EventEmitter<TerminalOutput>();
        this.onDidReceiveOutput = this._onDidReceiveOutput.event;
    }

    public async initializeSession(): Promise<boolean> {
        const token = await this.context.secrets.get('authToken');
        if (!token) {
            vscode.window.showErrorMessage(vscode.l10n.t('authRequired'));
            return false;
        }

        try {
            // Get session info from API
            const response = await this.apiClient.post('/api/terminal/ultra/session/init', {}, {
                headers: { 'Authorization': `Bearer ${token}` }
            });

            this.session = {
                sessionId: response.data.session_id,
                userId: response.data.user_id,
                tier: response.data.tier,
                wsUrl: response.data.ws_url,
                connected: false
            };

            await this.connectWebSocket();
            return true;
        } catch (error) {
            vscode.window.showErrorMessage(vscode.l10n.t('failedInit', String(error)));
            return false;
        }
    }

    private async connectWebSocket(): Promise<void> {
        if (!this.session) return;

        const wsUrl = this.session.wsUrl.replace('http://', 'ws://').replace('https://', 'wss://');

        try {
            // Note: WebSocket is not globally available in node.js environment of VS Code extension
            // In a real extension, you would use a library like 'ws' or vscode.Webview context
            // For now, keeping the logic but adding localization
            const token = await this.context.secrets.get('authToken');
            this.websocket = new (require('ws'))(`${wsUrl}?token=${token}`);

            if (this.websocket) {
                this.websocket.on('open', () => {
                    this.session!.connected = true;
                    vscode.window.showInformationMessage(vscode.l10n.t('guardianActive'));
                    this.outputChannel.appendLine(`[Guardian] ${vscode.l10n.t('sessionEstablished')}`);
                    this.outputChannel.appendLine(`[Guardian] ${vscode.l10n.t('resourceLimits')}`);
                    this.outputChannel.appendLine(`[Guardian] ${vscode.l10n.t('networkIsolated')}`);
                    this.outputChannel.appendLine(`[Guardian] ${vscode.l10n.t('behavioralAnalysis')}`);
                });

                this.websocket.on('message', (data: any) => {
                    const message = JSON.parse(data.toString());
                    this.handleWebSocketMessage(message);
                });

                this.websocket.on('error', (error: any) => {
                    vscode.window.showErrorMessage(vscode.l10n.t('wsError'));
                    console.error('WebSocket error:', error);
                });

                this.websocket.on('close', () => {
                    this.session!.connected = false;
                    vscode.window.showWarningMessage(vscode.l10n.t('disconnected'));
                });
            }
        } catch (error) {
            vscode.window.showErrorMessage(vscode.l10n.t('wsFailed', String(error)));
        }
    }

    private handleWebSocketMessage(message: any): void {
        switch (message.type) {
            case 'output':
                this.outputChannel.append(message.data);
                this._onDidReceiveOutput.fire({
                    type: 'stdout',
                    data: message.data,
                    timestamp: Date.now()
                });
                break;
            case 'error':
                this.outputChannel.appendLine(`Error: ${message.data}`);
                this._onDidReceiveOutput.fire({
                    type: 'stderr',
                    data: message.data,
                    timestamp: Date.now()
                });
                break;
            case 'status':
                this.outputChannel.appendLine(`[Status] ${message.status}`);
                if (message.exit_code !== undefined) {
                    this.outputChannel.appendLine(`[Exit Code] ${message.exit_code}`);
                }
                break;
            case 'resource_update':
                this._onDidReceiveOutput.fire({
                    type: 'resource',
                    data: `Memory: ${message.memory_mb}MB | CPU: ${message.cpu_percent.toFixed(1)}%`,
                    timestamp: Date.now()
                });
                break;
        }
    }

    public async executeCommand(command: string, args: string[] = []): Promise<void> {
        if (!this.session?.connected || !this.websocket) {
            vscode.window.showErrorMessage(vscode.l10n.t('terminalNotConnected'));
            return;
        }

        const message = {
            type: 'execute',
            command,
            args,
            timestamp: Date.now()
        };

        this.websocket.send(JSON.stringify(message));
        this.outputChannel.appendLine(`$ ${command} ${args.join(' ')}`);
    }

    public async disconnect(): Promise<void> {
        if (this.websocket) {
            this.websocket.close();
            this.websocket = null;
        }
        this.session = null;
    }

    public showTerminal(): void {
        this.outputChannel.show(true);
    }

    public isConnected(): boolean {
        return this.session?.connected ?? false;
    }
}

// TerminalSession and other interfaces remain same
