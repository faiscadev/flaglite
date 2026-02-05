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
  login: async (username: string, password: string) => {
    const response = await api.post('/v1/auth/login', { username, password });
    return response.data;
  },
  signup: async (password: string, username?: string, project_name?: string) => {
    const response = await api.post('/v1/auth/signup', { 
      password, 
      username,
      project_name 
    });
    return response.data;
  },
  me: async () => {
    const response = await api.get('/v1/auth/me');
    return response.data;
  },
};

// Projects API
export interface Project {
  id: string;
  name: string;
  description?: string;
  slug: string;
  created_at: string;
  updated_at: string;
}

export const projectsApi = {
  list: async (): Promise<Project[]> => {
    const response = await api.get<Project[]>('/v1/projects');
    return response.data;
  },
  create: async (data: { name: string; description?: string }): Promise<Project> => {
    const response = await api.post<Project>('/v1/projects', data);
    return response.data;
  },
  getEnvironments: async (projectId: string) => {
    const response = await api.get(`/v1/projects/${projectId}/environments`);
    return response.data;
  },
};

// Flags API
export interface Flag {
  id: string;
  name: string;
  key: string;
  description?: string;
  flag_type: string;
  project_id: string;
  enabled: boolean;
  value?: any;
  created_at: string;
  updated_at: string;
}

export const flagsApi = {
  list: async (projectId: string, environment: string = 'production'): Promise<Flag[]> => {
    const response = await api.get<Flag[]>(
      `/v1/projects/${projectId}/flags`,
      { params: { environment } }
    );
    return response.data;
  },
  get: async (projectId: string, key: string, environment: string = 'production'): Promise<Flag> => {
    const response = await api.get<Flag>(
      `/v1/projects/${projectId}/flags/${key}`,
      { params: { environment } }
    );
    return response.data;
  },
  create: async (projectId: string, data: { name: string; key: string; description?: string; enabled?: boolean }): Promise<Flag> => {
    const response = await api.post<Flag>(`/v1/projects/${projectId}/flags`, data);
    return response.data;
  },
  toggle: async (projectId: string, key: string, environment: string): Promise<Flag> => {
    const response = await api.post<Flag>(
      `/v1/projects/${projectId}/flags/${key}/toggle`,
      null,
      { params: { environment } }
    );
    return response.data;
  },
  delete: async (projectId: string, key: string): Promise<void> => {
    await api.delete(`/v1/projects/${projectId}/flags/${key}`);
  },
};

// API Keys API
export interface ApiKey {
  id: string;
  name: string;
  key: string;
  key_prefix?: string;
  environment?: string;
  created_at: string;
}

export const apiKeysApi = {
  list: async () => {
    const response = await api.get<{ keys: ApiKey[] }>('/v1/api-keys');
    return response.data.keys;
  },
};
