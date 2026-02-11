import * as vscode from 'vscode';
import { ApiClient } from './apiClient';
import { AuthManager } from './authManager';

export interface ChatMessage {
    id: string;
    role: 'user' | 'assistant' | 'system';
    content: string;
    timestamp: number;
    ultraModeApplied?: boolean;
}

export class ChatPanelProvider implements vscode.WebviewViewProvider {
    private static readonly viewType = 'mr-darkpromth-sidebar';
    private _view?: vscode.WebviewView;
    private context: vscode.ExtensionContext;
    private apiClient: ApiClient;
    private authManager: AuthManager;
    private chatHistory: ChatMessage[] = [];
    private currentPanel?: vscode.WebviewPanel;
    private ultraMode: boolean = false;

    constructor(context: vscode.ExtensionContext, apiClient: ApiClient, authManager: AuthManager) {
        this.context = context;
        this.apiClient = apiClient;
        this.authManager = authManager;
        this.ultraMode = this.context.globalState.get<boolean>('ultraMode', false);
        this.loadChatHistory();
    }

    public async resolveWebviewView(
        webviewView: vscode.WebviewView,
        context: vscode.WebviewViewResolveContext,
        _token: vscode.CancellationToken
    ) {
        this._view = webviewView;

        webviewView.webview.options = {
            enableScripts: true,
            localResourceRoots: [this.context.extensionUri]
        };

        webviewView.webview.html = this.getWebviewContent(webviewView.webview);

        webviewView.webview.onDidReceiveMessage(async (message) => {
            switch (message.type) {
                case 'sendMessage':
                    await this.handleSendMessage(message.content, message.ultraMode);
                    break;
                case 'toggleUltraMode':
                    this.ultraMode = message.value;
                    await this.context.globalState.update('ultraMode', this.ultraMode);
                    vscode.commands.executeCommand('mr-darkpromth.ultraModeChanged', this.ultraMode);
                    break;
                case 'clearChat':
                    this.clearChat();
                    break;
                case 'getHistory':
                    this.postMessage({ type: 'chatHistory', messages: this.chatHistory });
                    break;
                case 'getUserInfo':
                    const userInfo = this.authManager.getUserInfo();
                    this.postMessage({ type: 'userInfo', userInfo });
                    break;
            }
        });

        this.postMessage({ type: 'chatHistory', messages: this.chatHistory });
    }

    public async show(): Promise<void> {
        if (this.currentPanel) {
            this.currentPanel.reveal();
            return;
        }

        const panel = vscode.window.createWebviewPanel(
            'mr-darkpromth.chat',
            vscode.l10n.t('chatTitle'),
            vscode.ViewColumn.One,
            {
                enableScripts: true,
                localResourceRoots: [this.context.extensionUri],
                retainContextWhenHidden: true
            }
        );

        panel.webview.html = this.getWebviewContent(panel.webview);

        panel.webview.onDidReceiveMessage(async (message) => {
            switch (message.type) {
                case 'sendMessage':
                    await this.handleSendMessage(message.content, message.ultraMode);
                    break;
                case 'toggleUltraMode':
                    this.ultraMode = message.value;
                    await this.context.globalState.update('ultraMode', this.ultraMode);
                    vscode.commands.executeCommand('mr-darkpromth.ultraModeChanged', this.ultraMode);
                    break;
                case 'clearChat':
                    this.clearChat();
                    break;
                case 'getHistory':
                    this.postMessage({ type: 'chatHistory', messages: this.chatHistory });
                    break;
                case 'getUserInfo':
                    const userInfo = this.authManager.getUserInfo();
                    this.postMessage({ type: 'userInfo', userInfo });
                    break;
            }
        });

        panel.onDidDispose(() => {
            this.currentPanel = undefined;
        });

        this.currentPanel = panel;
        this.postMessage({ type: 'chatHistory', messages: this.chatHistory });
    }

    private async handleSendMessage(content: string, ultraMode?: boolean): Promise<void> {
        if (!this.authManager.isAuthenticated()) {
            vscode.window.showErrorMessage(vscode.l10n.t('authRequired'));
            return;
        }

        const userMessage: ChatMessage = {
            id: this.generateId(),
            role: 'user',
            content,
            timestamp: Date.now()
        };

        this.chatHistory.push(userMessage);
        this.postMessage({ type: 'message', message: userMessage });
        await this.saveChatHistory();

        try {
            // Use persistent conversation ID if available, otherwise generate once
            let conversationId = this.context.globalState.get<string>('currentConversationId');
            if (!conversationId) {
                conversationId = this.generateId();
                await this.context.globalState.update('currentConversationId', conversationId);
            }

            const response = await this.apiClient.post('/api/chat', {
                message: content,
                conversation_id: conversationId,
                jailbreak_prompt: ultraMode ? 'enabled' : undefined,
                ultra_mode: ultraMode
            });

            const assistantMessage: ChatMessage = {
                id: this.generateId(),
                role: 'assistant',
                content: response.data.response || response.data.ai_response || 'No response received',
                timestamp: Date.now(),
                ultraModeApplied: response.data.jailbreak_applied || (ultraMode ? true : false)
            };

            this.chatHistory.push(assistantMessage);
            this.postMessage({ type: 'message', message: assistantMessage });
            await this.saveChatHistory();
        } catch (error) {
            const errorMessage: ChatMessage = {
                id: this.generateId(),
                role: 'system',
                content: `Error: ${error}`,
                timestamp: Date.now()
            };

            this.chatHistory.push(errorMessage);
            this.postMessage({ type: 'message', message: errorMessage });
        }
    }

    public clearChat(): void {
        this.chatHistory = [];
        this.postMessage({ type: 'clearChat' });
        this.saveChatHistory();
    }

    private postMessage(message: any): void {
        // Automatically inject ultraMode into certain messages if needed, 
        // or add a separate init message.
        if (message.type === 'chatHistory' || message.type === 'userInfo') {
            message.ultraMode = this.ultraMode;
        }

        if (this._view) {
            this._view.webview.postMessage(message);
        }
        if (this.currentPanel) {
            this.currentPanel.webview.postMessage(message);
        }
    }

    private getWebviewContent(webview: vscode.Webview): string {
        const config = vscode.workspace.getConfiguration('mr-darkpromth');
        const theme = config.get<string>('theme', 'auto');

        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MR.DarkPromth Chat</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background-color: var(--vscode-editor-background);
            color: var(--vscode-editor-foreground);
            height: 100vh;
            display: flex;
            flex-direction: column;
        }
        
        .container {
            display: flex;
            flex-direction: column;
            height: 100%;
        }
        
        .header {
            padding: 16px;
            background-color: var(--vscode-editor-inactiveSelectionBackground);
            border-bottom: 1px solid var(--vscode-panel-border);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        
        .header h1 {
            font-size: 18px;
            font-weight: 600;
        }
        
        .header-actions {
            display: flex;
            gap: 8px;
        }
        
        button {
            padding: 6px 12px;
            background-color: var(--vscode-button-background);
            color: var(--vscode-button-foreground);
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 13px;
        }
        
        button:hover {
            background-color: var(--vscode-button-hoverBackground);
        }
        
        .chat-container {
            flex: 1;
            overflow-y: auto;
            padding: 16px;
        }
        
        .message {
            margin-bottom: 16px;
            padding: 12px;
            border-radius: 8px;
            max-width: 80%;
        }
        
        .message.user {
            background-color: var(--vscode-button-background);
            margin-left: auto;
        }
        
        .message.assistant {
            background-color: var(--vscode-editor-inactiveSelectionBackground);
        }
        
        .message.system {
            background-color: var(--vscode-editorError-foreground);
            color: var(--vscode-editor-background);
        }
        
        .message-role {
            font-size: 11px;
            font-weight: 600;
            margin-bottom: 4px;
            opacity: 0.7;
        }
        
        .message-content {
            white-space: pre-wrap;
            word-wrap: break-word;
        }
        
        .input-container {
            padding: 16px;
            background-color: var(--vscode-editor-inactiveSelectionBackground);
            border-top: 1px solid var(--vscode-panel-border);
        }
        
        .input-wrapper {
            display: flex;
            gap: 8px;
        }
        
        textarea {
            flex: 1;
            min-height: 60px;
            max-height: 200px;
            padding: 8px;
            background-color: var(--vscode-input-background);
            color: var(--vscode-input-foreground);
            border: 1px solid var(--vscode-input-border);
            border-radius: 4px;
            resize: vertical;
            font-family: inherit;
        }
        
        textarea:focus {
            outline: none;
            border-color: var(--vscode-focusBorder);
        }
        
        .jailbreak-badge {
            display: inline-block;
            padding: 2px 6px;
            background-color: #ff6b6b;
            color: white;
            border-radius: 3px;
            font-size: 10px;
            margin-left: 8px;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>${vscode.l10n.t('chatTitle')}</h1>
            <div class="header-actions">
                <button id="toggleMode" class="btn-toggle">Mode: Normal</button>
                <button id="clearChat">${vscode.l10n.t('clear')}</button>
            </div>
        </div>
        <div class="chat-container" id="chatContainer"></div>
        <div class="status-bar-sim" id="statusBarSim" style="font-size: 10px; opacity: 0.6; padding: 4px 16px; border-top: 1px solid var(--vscode-panel-border); font-family: monospace;">
            ULTRA_MODE: <span id="ultraStatus">OFF</span>
        </div>
        <div class="input-container">
            <div class="input-wrapper">
                <textarea id="messageInput" placeholder="${vscode.l10n.t('typeMessage')}"></textarea>
                <button id="sendButton">${vscode.l10n.t('send')}</button>
            </div>
        </div>
    </div>
    
    <script>
        const vscode = acquireVsCodeApi();
        const chatContainer = document.getElementById('chatContainer');
        const messageInput = document.getElementById('messageInput');
        const sendButton = document.getElementById('sendButton');
        const clearButton = document.getElementById('clearChat');
        const toggleButton = document.getElementById('toggleMode');
        const ultraStatus = document.getElementById('ultraStatus');
        
        let ultraMode = false;
        
        vscode.postMessage({ type: 'getHistory' });
        vscode.postMessage({ type: 'getUserInfo' });

        function updateUltraUI() {
            toggleButton.textContent = ultraMode ? 'Mode: Mr.DarkPromth' : 'Mode: Normal';
            toggleButton.style.backgroundColor = ultraMode ? '#ff6b6b' : 'var(--vscode-button-background)';
            ultraStatus.textContent = ultraMode ? 'ON' : 'OFF';
            ultraStatus.style.color = ultraMode ? '#ff6b6b' : 'inherit';
        }

        toggleButton.addEventListener('click', () => {
            ultraMode = !ultraMode;
            updateUltraUI();
            vscode.postMessage({ type: 'toggleUltraMode', value: ultraMode });
        });
        
        function renderMessage(message) {
            const messageDiv = document.createElement('div');
            messageDiv.className = 'message ' + message.role;
            
            const roleDiv = document.createElement('div');
            roleDiv.className = 'message-role';
            roleDiv.textContent = message.role.charAt(0).toUpperCase() + message.role.slice(1);
            
            if (message.ultraModeApplied) {
                const badge = document.createElement('span');
                badge.className = 'jailbreak-badge';
                badge.textContent = 'Ultra Mode';
                roleDiv.appendChild(badge);
            }
            
            const contentDiv = document.createElement('div');
            contentDiv.className = 'message-content';
            contentDiv.textContent = message.content;
            
            messageDiv.appendChild(roleDiv);
            messageDiv.appendChild(contentDiv);
            chatContainer.appendChild(messageDiv);
            
            chatContainer.scrollTop = chatContainer.scrollHeight;
        }
        
        window.addEventListener('message', (event) => {
            const message = event.data;
            
            if (message.ultraMode !== undefined) {
                ultraMode = message.ultraMode;
                updateUltraUI();
            }

            switch (message.type) {
                case 'chatHistory':
                    chatContainer.innerHTML = '';
                    message.messages.forEach(renderMessage);
                    break;
                    
                case 'message':
                    renderMessage(message.message);
                    break;
                    
                case 'clearChat':
                    chatContainer.innerHTML = '';
                    break;
                    
                case 'userInfo':
                    if (message.userInfo) {
                        document.querySelector('.header h1').textContent = 
                            'MR.DarkPromth Chat (' + message.userInfo.tier + ')';
                    }
                    break;
            }
        });
        
        function sendMessage() {
            const content = messageInput.value.trim();
            if (!content) return;
            
            vscode.postMessage({
                type: 'sendMessage',
                content: content,
                ultraMode: ultraMode
            });
            
            messageInput.value = '';
        }
        
        sendButton.addEventListener('click', sendMessage);
        messageInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                sendMessage();
            }
        });
        
        clearButton.addEventListener('click', () => {
            vscode.postMessage({ type: 'clearChat' });
        });
    </script>
</body>
</html>`;
    }

    private generateId(): string {
        return Date.now().toString(36) + Math.random().toString(36).substr(2);
    }

    private async saveChatHistory(): Promise<void> {
        const config = vscode.workspace.getConfiguration('mr-darkpromth');
        const maxHistoryLength = config.get<number>('maxHistoryLength', 50);

        if (this.chatHistory.length > maxHistoryLength) {
            this.chatHistory = this.chatHistory.slice(-maxHistoryLength);
        }

        await this.context.globalState.update('chatHistory', this.chatHistory);
    }

    private loadChatHistory(): void {
        this.chatHistory = this.context.globalState.get<ChatMessage[]>('chatHistory', []);
    }
}
