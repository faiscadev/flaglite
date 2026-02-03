import axios from 'axios';

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

export const api = axios.create({
  baseURL: API_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Add auth token to requests
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Handle 401 responses
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('token');
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);

// Auth API
export const authApi = {
  login: async (email: string, password: string) => {
    const response = await api.post('/v1/auth/login', { email, password });
    return response.data;
  },
  signup: async (email: string, password: string) => {
    const response = await api.post('/v1/auth/signup', { email, password });
    return response.data;
  },
};

// Flags API
export interface Flag {
  id: string;
  name: string;
  key: string;
  description?: string;
  enabled: boolean;
  rollout_percentage: number;
  environments?: Record<string, { enabled: boolean; rollout_percentage: number }>;
  created_at: string;
  updated_at: string;
}

export const flagsApi = {
  list: async () => {
    const response = await api.get<{ flags: Flag[] }>('/v1/flags');
    return response.data.flags;
  },
  get: async (key: string) => {
    const response = await api.get<Flag>(`/v1/flags/${key}`);
    return response.data;
  },
  create: async (data: { name: string; key: string; description?: string }) => {
    const response = await api.post<Flag>('/v1/flags', data);
    return response.data;
  },
  toggle: async (key: string, environment?: string) => {
    const url = environment 
      ? `/v1/flags/${key}/environments/${environment}/toggle`
      : `/v1/flags/${key}/toggle`;
    const response = await api.post<Flag>(url);
    return response.data;
  },
  update: async (key: string, environment: string, data: { enabled?: boolean; rollout_percentage?: number }) => {
    const response = await api.patch<Flag>(`/v1/flags/${key}/environments/${environment}`, data);
    return response.data;
  },
};

// API Keys API
export interface ApiKey {
  id: string;
  name: string;
  key: string;
  environment?: string;
  created_at: string;
}

export const apiKeysApi = {
  list: async () => {
    const response = await api.get<{ keys: ApiKey[] }>('/v1/api-keys');
    return response.data.keys;
  },
};
