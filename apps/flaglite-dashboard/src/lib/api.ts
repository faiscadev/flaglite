import axios, { AxiosError } from 'axios';
import type {
  AuthResponse,
  CreateFlagRequest,
  CreateProjectRequest,
  CreateProjectResponse,
  Environment,
  Flag,
  FlagToggleResponse,
  LoginRequest,
  Project,
  SignupRequest,
  SignupResponse,
  User,
} from '../types';

const API_URL = import.meta.env.VITE_API_URL || 'https://api.flaglite.dev';

export const api = axios.create({
  baseURL: API_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Add auth token to requests
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('flaglite_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Handle 401 responses
api.interceptors.response.use(
  (response) => response,
  (error: AxiosError) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('flaglite_token');
      localStorage.removeItem('flaglite_user');
      localStorage.removeItem('flaglite_project');
      localStorage.removeItem('flaglite_project_api_key');
      localStorage.removeItem('flaglite_environments');
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);

// Helper to get current project ID
const getCurrentProjectId = () => {
  const projectStr = localStorage.getItem('flaglite_project');
  if (!projectStr) return null;
  try {
    const project = JSON.parse(projectStr);
    return project.id;
  } catch {
    return null;
  }
};

// Auth API
export const authApi = {
  signup: async (data: SignupRequest): Promise<SignupResponse> => {
    const response = await api.post<SignupResponse>('/v1/auth/signup', data);
    return response.data;
  },

  login: async (data: LoginRequest): Promise<AuthResponse> => {
    const response = await api.post<AuthResponse>('/v1/auth/login', data);
    return response.data;
  },

  me: async (): Promise<User> => {
    const response = await api.get<User>('/v1/auth/me');
    return response.data;
  },
};

// Projects API
export const projectsApi = {
  list: async (): Promise<Project[]> => {
    const response = await api.get<Project[]>('/v1/projects');
    return response.data;
  },

  create: async (data: CreateProjectRequest): Promise<CreateProjectResponse> => {
    // API returns just the project, so we need to fetch environments separately
    const response = await api.post<Project>('/v1/projects', data);
    const project = response.data;
    const environments = await projectsApi.getEnvironments(project.id);
    return { project, environments };
  },

  getEnvironments: async (projectId: string): Promise<Environment[]> => {
    const response = await api.get<Environment[]>(`/v1/projects/${projectId}/environments`);
    return response.data;
  },
};

// Flags API (uses JWT auth and project routes)
export const flagsApi = {
  list: async (projectId?: string): Promise<Flag[]> => {
    const pid = projectId || getCurrentProjectId();
    if (!pid) throw new Error('No project selected');
    
    const response = await api.get<Flag[]>(`/v1/projects/${pid}/flags`);
    return response.data;
  },

  get: async (key: string, projectId?: string): Promise<Flag> => {
    const pid = projectId || getCurrentProjectId();
    if (!pid) throw new Error('No project selected');
    
    const response = await api.get<Flag>(`/v1/projects/${pid}/flags/${key}`);
    return response.data;
  },

  create: async (data: CreateFlagRequest, projectId?: string): Promise<Flag> => {
    const pid = projectId || getCurrentProjectId();
    if (!pid) throw new Error('No project selected');
    
    const response = await api.post<Flag>(`/v1/projects/${pid}/flags`, data);
    return response.data;
  },

  toggle: async (key: string, environment: string, projectId?: string): Promise<FlagToggleResponse> => {
    const pid = projectId || getCurrentProjectId();
    if (!pid) throw new Error('No project selected');
    
    const response = await api.post<FlagToggleResponse>(
      `/v1/projects/${pid}/flags/${key}/toggle`,
      null,
      { params: { environment } }
    );
    return response.data;
  },

  delete: async (key: string, projectId?: string): Promise<void> => {
    const pid = projectId || getCurrentProjectId();
    if (!pid) throw new Error('No project selected');
    
    await api.delete(`/v1/projects/${pid}/flags/${key}`);
  },
};

// Error handling helper
export const getErrorMessage = (error: unknown): string => {
  if (error instanceof AxiosError) {
    return error.response?.data?.error || error.message;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return 'An unexpected error occurred';
};
