import * as vscode from 'vscode';
import * as jwt from 'jsonwebtoken';
import { ApiClient } from './apiClient';

export interface UserInfo {
    id: string;
    email: string;
    username: string;
    tier: 'Free' | 'Premium' | 'Ultra';
    apiKey: string;
    apiKeyExpiration?: string;
}

export class AuthManager {
    private context: vscode.ExtensionContext;
    private apiClient: ApiClient;
    private userInfo: UserInfo | null = null;

    constructor(context: vscode.ExtensionContext, apiClient: ApiClient) {
        this.context = context;
        this.apiClient = apiClient;
        this.loadStoredCredentials();
    }

    async authenticate(email: string, password: string): Promise<void> {
        try {
            const response = await this.apiClient.post('/api/auth/login', {
                email,
                password
            });

            if (response.data && response.data.token) {
                this.userInfo = {
                    id: response.data.user_id,
                    email: response.data.email,
                    username: response.data.username,
                    tier: response.data.tier || 'Free',
                    apiKey: response.data.api_key,
                    apiKeyExpiration: response.data.api_key_expiration
                };

                await this.storeCredentials(response.data.token, this.userInfo);
                vscode.commands.executeCommand('setContext', 'mr-darkpromth.authenticated', true);
            } else {
                throw new Error('Invalid response from server');
            }
        } catch (error) {
            throw new Error(`Authentication failed: ${error}`);
        }
    }

    async logout(): Promise<void> {
        await this.context.secrets.delete('authToken');
        await this.context.globalState.update('userInfo', undefined);
        this.userInfo = null;
        vscode.commands.executeCommand('setContext', 'mr-darkpromth.authenticated', false);
    }

    isAuthenticated(): boolean {
        return this.userInfo !== null;
    }

    getUserInfo(): UserInfo | null {
        return this.userInfo;
    }

    getApiKey(): string | null {
        return this.userInfo?.apiKey || null;
    }

    getUserTier(): string {
        return this.userInfo?.tier || 'Free';
    }

    async refreshUserInfo(): Promise<void> {
        if (!this.isAuthenticated()) {
            return;
        }

        try {
            const response = await this.apiClient.get('/api/users/me');
            if (response.data) {
                this.userInfo = {
                    id: response.data.id,
                    email: response.data.email,
                    username: response.data.username,
                    tier: response.data.tier || 'Free',
                    apiKey: response.data.api_key,
                    apiKeyExpiration: response.data.api_key_expiration
                };
                await this.context.globalState.update('userInfo', this.userInfo);
            }
        } catch (error) {
            console.error('Failed to refresh user info:', error);
        }
    }

    getApiKeyExpiration(): string | null {
        return this.userInfo?.apiKeyExpiration || null;
    }

    isApiKeyExpired(): boolean {
        if (!this.userInfo?.apiKeyExpiration) {
            return false;
        }

        const expirationDate = new Date(this.userInfo.apiKeyExpiration);
        return expirationDate < new Date();
    }

    private async storeCredentials(token: string, userInfo: UserInfo): Promise<void> {
        await this.context.secrets.store('authToken', token);
        await this.context.globalState.update('userInfo', userInfo);
    }

    private async loadStoredCredentials(): Promise<void> {
        const token = await this.context.secrets.get('authToken');
        const userInfo = this.context.globalState.get<UserInfo>('userInfo');

        if (token && userInfo) {
            try {
                const decoded = jwt.decode(token) as any;
                if (decoded && decoded.exp * 1000 > Date.now()) {
                    this.userInfo = userInfo;
                    vscode.commands.executeCommand('setContext', 'mr-darkpromth.authenticated', true);
                } else {
                    await this.logout();
                }
            } catch (error) {
                console.error('Failed to decode token:', error);
                await this.logout();
            }
        }
    }
}
