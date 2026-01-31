import * as vscode from 'vscode';
import { AuthManager } from './authManager';

export class StatusBarManager {
    private authManager: AuthManager;
    private statusBarItem: vscode.StatusBarItem;
    private tierBarItem: vscode.StatusBarItem;
    private updateInterval: NodeJS.Timeout | null = null;

    constructor(authManager: AuthManager) {
        this.authManager = authManager;
        this.statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
        this.tierBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 99);
    }

    initialize(): void {
        this.statusBarItem.command = 'mr-darkpromth.refreshUserInfo';
        this.tierBarItem.command = 'mr-darkpromth.refreshUserInfo';
        
        this.statusBarItem.show();
        this.tierBarItem.show();
        
        this.update();
        
        this.updateInterval = setInterval(() => {
            this.update();
        }, 60000);
    }

    async update(): Promise<void> {
        if (!this.authManager.isAuthenticated()) {
            this.statusBarItem.text = '$(circle-slash) Not Authenticated';
            this.statusBarItem.tooltip = 'Click to authenticate';
            this.statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
            
            this.tierBarItem.hide();
            return;
        }

        const userInfo = this.authManager.getUserInfo();
        if (!userInfo) {
            return;
        }

        const tier = userInfo.tier;
        const tierIcon = tier === 'Ultra' ? '$(star-full)' : tier === 'Premium' ? '$(star)' : '$(circle-outline)';
        
        this.tierBarItem.text = `${tierIcon} ${tier}`;
        this.tierBarItem.tooltip = `User Tier: ${tier}`;
        
        const apiKeyExpiration = this.authManager.getApiKeyExpiration();
        if (apiKeyExpiration) {
            const expirationDate = new Date(apiKeyExpiration);
            const now = new Date();
            const timeDiff = expirationDate.getTime() - now.getTime();
            const daysRemaining = Math.ceil(timeDiff / (1000 * 60 * 60 * 24));
            
            if (daysRemaining <= 0) {
                this.statusBarItem.text = '$(warning) API Key Expired';
                this.statusBarItem.tooltip = 'Your API key has expired. Please refresh.';
                this.statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
            } else if (daysRemaining <= 7) {
                this.statusBarItem.text = `$(warning) Expires in ${daysRemaining} day${daysRemaining > 1 ? 's' : ''}`;
                this.statusBarItem.tooltip = `API Key expires on ${expirationDate.toLocaleDateString()}`;
                this.statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.warningBackground');
            } else {
                this.statusBarItem.text = `$(check) Key Valid`;
                this.statusBarItem.tooltip = `API Key expires on ${expirationDate.toLocaleDateString()}`;
                this.statusBarItem.backgroundColor = undefined;
            }
        } else {
            this.statusBarItem.text = '$(check) API Key Active';
            this.statusBarItem.tooltip = 'API Key is active';
            this.statusBarItem.backgroundColor = undefined;
        }
    }

    dispose(): void {
        if (this.updateInterval) {
            clearInterval(this.updateInterval);
        }
        this.statusBarItem.dispose();
        this.tierBarItem.dispose();
    }
}
