import * as vscode from 'vscode';
import { ChatPanelProvider } from './chatPanelProvider';
import { AuthManager } from './authManager';
import { ApiClient } from './apiClient';
import { StatusBarManager } from './statusBarManager';
import { EventBusCoordinator } from './eventBusCoordinator';

let chatPanelProvider: ChatPanelProvider;
let authManager: AuthManager;
let apiClient: ApiClient;
let statusBarManager: StatusBarManager;
let eventBusCoordinator: EventBusCoordinator;

export function activate(context: vscode.ExtensionContext) {
    console.log('MR.DarkPromth extension is now active!');

    const config = vscode.workspace.getConfiguration('mr-darkpromth');
    const apiEndpoint = config.get<string>('apiEndpoint', 'http://127.0.0.1:8080');

    // Initialize event bus coordinator for agent coordination
    eventBusCoordinator = new EventBusCoordinator();
    
    apiClient = new ApiClient(apiEndpoint);
    authManager = new AuthManager(context, apiClient);
    statusBarManager = new StatusBarManager(authManager);
    chatPanelProvider = new ChatPanelProvider(context, apiClient, authManager);

    context.subscriptions.push(
        vscode.commands.registerCommand('mr-darkpromth.openChat', async () => {
            if (!authManager.isAuthenticated()) {
                const result = await vscode.window.showWarningMessage(
                    'You need to authenticate first',
                    'Authenticate'
                );
                if (result === 'Authenticate') {
                    await vscode.commands.executeCommand('mr-darkpromth.authenticate');
                }
                return;
            }
            await chatPanelProvider.show();
        }),

        vscode.commands.registerCommand('mr-darkpromth.authenticate', async () => {
            const email = await vscode.window.showInputBox({
                prompt: 'Enter your email',
                placeHolder: 'user@example.com'
            });

            if (!email) {
                return;
            }

            const password = await vscode.window.showInputBox({
                prompt: 'Enter your password',
                password: true
            });

            if (!password) {
                return;
            }

            try {
                await authManager.authenticate(email, password);
                vscode.window.showInformationMessage('Authentication successful!');
                await statusBarManager.update();
                vscode.commands.executeCommand('setContext', 'mr-darkpromth.authenticated', true);
            } catch (error) {
                vscode.window.showErrorMessage(`Authentication failed: ${error}`);
            }
        }),

        vscode.commands.registerCommand('mr-darkpromth.clearChat', async () => {
            const result = await vscode.window.showWarningMessage(
                'Are you sure you want to clear the chat history?',
                'Clear',
                'Cancel'
            );
            if (result === 'Clear') {
                chatPanelProvider.clearChat();
                vscode.window.showInformationMessage('Chat history cleared');
            }
        }),

        vscode.commands.registerCommand('mr-darkpromth.refreshUserInfo', async () => {
            try {
                await authManager.refreshUserInfo();
                await statusBarManager.update();
                vscode.window.showInformationMessage('User information refreshed');
            } catch (error) {
                vscode.window.showErrorMessage(`Failed to refresh user info: ${error}`);
            }
        }),

        vscode.window.registerWebviewViewProvider('mr-darkpromth-sidebar', chatPanelProvider)
    );

    statusBarManager.initialize();

    // Initialize event bus coordination
    eventBusCoordinator.connect().then(() => {
        console.log('Event bus coordinator connected');
        eventBusCoordinator.publishEvent('task_completion', {
            task: 'vscode_extension_initialized',
            status: 'completed',
            artifacts: ['vscode-extension/'],
            metadata: { version: '1.0.0' }
        });
    }).catch(error => {
        console.warn('Failed to connect to Redis event bus:', error);
    });

    vscode.workspace.onDidChangeConfiguration(async (e) => {
        if (e.affectsConfiguration('mr-darkpromth.apiEndpoint')) {
            const newEndpoint = config.get<string>('apiEndpoint', 'http://127.0.0.1:8080');
            apiClient.updateEndpoint(newEndpoint);
        }
    });

    if (config.get<boolean>('autoUpdate', true)) {
        checkForUpdates(context);
    }
}

export function deactivate() {
    console.log('MR.DarkPromth extension is now deactivated');
    
    if (eventBusCoordinator) {
        eventBusCoordinator.disconnect().catch(error => {
            console.error('Error disconnecting event bus:', error);
        });
    }
}

async function checkForUpdates(context: vscode.ExtensionContext) {
    const currentVersion = context.extension.packageJSON.version;
    try {
        const response = await fetch('https://marketplace.visualstudio.com/_apis/public/gallery/extensionquery');
        // Update check implementation would go here
        console.log('Checking for updates...');
    } catch (error) {
        console.error('Failed to check for updates:', error);
    }
}
