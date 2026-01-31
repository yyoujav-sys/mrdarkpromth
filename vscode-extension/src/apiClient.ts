import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';

export class ApiClient {
    private client: AxiosInstance;
    private apiEndpoint: string;

    constructor(apiEndpoint: string) {
        this.apiEndpoint = apiEndpoint;
        this.client = axios.create({
            baseURL: apiEndpoint,
            timeout: 30000,
            headers: {
                'Content-Type': 'application/json'
            }
        });

        this.client.interceptors.request.use(
            (config) => {
                return config;
            },
            (error) => {
                return Promise.reject(error);
            }
        );

        this.client.interceptors.response.use(
            (response) => response,
            (error) => {
                if (error.response) {
                    console.error(`API Error: ${error.response.status} - ${error.response.data?.message || 'Unknown error'}`);
                } else if (error.request) {
                    console.error('API Error: No response received from server');
                } else {
                    console.error(`API Error: ${error.message}`);
                }
                return Promise.reject(error);
            }
        );
    }

    updateEndpoint(newEndpoint: string): void {
        this.apiEndpoint = newEndpoint;
        this.client.defaults.baseURL = newEndpoint;
    }

    async get<T = any>(url: string, config?: AxiosRequestConfig): Promise<AxiosResponse<T>> {
        return this.client.get<T>(url, config);
    }

    async post<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<AxiosResponse<T>> {
        return this.client.post<T>(url, data, config);
    }

    async put<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<AxiosResponse<T>> {
        return this.client.put<T>(url, data, config);
    }

    async delete<T = any>(url: string, config?: AxiosRequestConfig): Promise<AxiosResponse<T>> {
        return this.client.delete<T>(url, config);
    }

    getEndpoint(): string {
        return this.apiEndpoint;
    }
}
