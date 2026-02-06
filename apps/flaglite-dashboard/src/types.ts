// API Types - based on crates/flaglite-core/src/types.rs

export interface User {
  id: string;
  username: string;
  email?: string;
  created_at: string;
}

export interface Project {
  id: string;
  name: string;
  api_key: string;
  created_at: string;
}

export interface Environment {
  id: string;
  name: string;
  api_key: string;
  project_id: string;
  created_at: string;
}

export interface FlagEnvironmentValue {
  enabled: boolean;
  rollout: number;
}

export interface Flag {
  key: string;
  name: string;
  description?: string;
  environments: Record<string, FlagEnvironmentValue>;
}

// Request/Response types
export interface SignupRequest {
  username?: string;
  password: string;
  project_name?: string;
}

export interface SignupResponse {
  user: User;
  api_key: ApiKeyCreated;
  token: string;
  project?: Project;
  environments?: Environment[];
}

export interface ApiKeyCreated {
  id: string;
  key: string;
  key_prefix: string;
  name?: string;
  created_at: string;
}

export interface LoginRequest {
  username: string;
  password: string;
}

export interface AuthResponse {
  token: string;
  user: User;
  project?: Project;
  environments?: Environment[];
}

export interface CreateProjectRequest {
  name: string;
}

export interface CreateProjectResponse {
  project: Project;
  environments: Environment[];
}

export interface CreateFlagRequest {
  key: string;
  name: string;
  description?: string;
}

export interface FlagToggleResponse {
  key: string;
  environment: string;
  enabled: boolean;
}
