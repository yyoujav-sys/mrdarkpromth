import * as vscode from 'vscode';
import { ChatPanelProvider } from './chatPanelProvider';
import { AuthManager } from './authManager';
import { ApiClient } from './apiClient';
import { StatusBarManager } from './statusBarManager';
import { EventBusCoordinator } from './eventBusCoordinator';
import { UltraTerminalProvider } from './ultraTerminal';

let chatPanelProvider: ChatPanelProvider;
let authManager: AuthManager;
let apiClient: ApiClient;
let statusBarManager: StatusBarManager;
let eventBusCoordinator: EventBusCoordinator;
let ultraTerminalProvider: UltraTerminalProvider;
let healthCheckInterval: NodeJS.Timeout | null = null;

export function activate(context: vscode.ExtensionContext) {
    console.log('MR.DarkPromth extension is now active!');

    const config = vscode.workspace.getConfiguration('mr-darkpromth');
    const apiEndpoint = config.get<string>('apiEndpoint', 'https://mrdarkpromth.online/api');

    // Initialize event bus coordinator for agent coordination
    eventBusCoordinator = new EventBusCoordinator();

    apiClient = new ApiClient(apiEndpoint);
    authManager = new AuthManager(context, apiClient);
    statusBarManager = new StatusBarManager(authManager);
    chatPanelProvider = new ChatPanelProvider(context, apiClient, authManager);
    ultraTerminalProvider = new UltraTerminalProvider(context, apiClient, authManager);

    context.subscriptions.push(
        vscode.commands.registerCommand('mr-darkpromth.openChat', async () => {
            if (!authManager.isAuthenticated()) {
                const result = await vscode.window.showWarningMessage(
                    vscode.l10n.t('authRequired'),
                    vscode.l10n.t('connect')
                );
                if (result === vscode.l10n.t('connect')) {
                    await vscode.commands.executeCommand('mr-darkpromth.authenticate');
                }
                return;
            }
            await chatPanelProvider.show();
        }),

        vscode.commands.registerCommand('mr-darkpromth.authenticate', async () => {
            const email = await vscode.window.showInputBox({
                prompt: vscode.l10n.t('enterEmail'),
                placeHolder: 'user@example.com'
            });

            if (!email) {
                return;
            }

            const password = await vscode.window.showInputBox({
                prompt: vscode.l10n.t('enterPassword'),
                password: true
            });

            if (!password) {
                return;
            }

            try {
                await authManager.authenticate(email, password);
                vscode.window.showInformationMessage(vscode.l10n.t('authSuccess'));
                await statusBarManager.update();
                vscode.commands.executeCommand('setContext', 'mr-darkpromth.authenticated', true);
            } catch (error) {
                vscode.window.showErrorMessage(vscode.l10n.t('authFailed', String(error)));
            }
        }),

        vscode.commands.registerCommand('mr-darkpromth.clearChat', async () => {
            const result = await vscode.window.showWarningMessage(
                vscode.l10n.t('clearChatConfirm'),
                vscode.l10n.t('clear'),
                vscode.l10n.t('cancel')
            );
            if (result === vscode.l10n.t('clear')) {
                chatPanelProvider.clearChat();
                vscode.window.showInformationMessage(vscode.l10n.t('chatCleared'));
            }
        }),

        vscode.commands.registerCommand('mr-darkpromth.refreshUserInfo', async () => {
            try {
                await authManager.refreshUserInfo();
                await statusBarManager.update();
                vscode.window.showInformationMessage(vscode.l10n.t('refreshSuccess'));
            } catch (error) {
                vscode.window.showErrorMessage(vscode.l10n.t('refreshFailed', String(error)));
            }
        }),

        vscode.commands.registerCommand('mr-darkpromth.openUltraTerminal', async () => {
            if (!authManager.isAuthenticated()) {
                vscode.window.showErrorMessage(vscode.l10n.t('authRequired'));
                return;
            }
            const success = await ultraTerminalProvider.initializeSession();
            if (success) {
                ultraTerminalProvider.showTerminal();
            }
        }),

        vscode.window.registerWebviewViewProvider('mr-darkpromth-sidebar', chatPanelProvider),

        vscode.commands.registerCommand('mr-darkpromth.healthCheck', async () => {
            try {
                await apiClient.healthCheck();
                statusBarManager.updateHealthStatus(true);
            } catch (error) {
                statusBarManager.updateHealthStatus(false);
            }
        })
    );

    statusBarManager.initialize();

    // Initial health check
    vscode.commands.executeCommand('mr-darkpromth.healthCheck');

    // Periodic health check
    healthCheckInterval = setInterval(() => {
        vscode.commands.executeCommand('mr-darkpromth.healthCheck');
    }, 60000); // every 60 seconds


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
            const newEndpoint = config.get<string>('apiEndpoint', 'https://mrdarkpromth.online');
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
    if (healthCheckInterval) {
        clearInterval(healthCheckInterval);
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
