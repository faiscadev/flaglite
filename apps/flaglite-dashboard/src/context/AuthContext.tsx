/* eslint-disable react-refresh/only-export-components */
import { createContext, useContext, useState, useCallback, type ReactNode, useEffect } from 'react';
import type { User, Project, Environment } from '../types';

const STORAGE_KEYS = {
  TOKEN: 'flaglite_token',
  USER: 'flaglite_user',
  PROJECT: 'flaglite_project',
  PROJECT_API_KEY: 'flaglite_project_api_key',
  ENVIRONMENTS: 'flaglite_environments',
  SELECTED_PROJECT: 'flaglite_selected_project',
} as const;

interface AuthState {
  user: User | null;
  currentProject: Project | null;
  environments: Environment[];
  selectedProjectId: string | null;
}

interface AuthContextType extends AuthState {
  isAuthenticated: boolean;
  login: (token: string, user: User, project?: Project, environments?: Environment[]) => void;
  logout: () => void;
  setCurrentProject: (project: Project, environments: Environment[]) => void;
  setSelectedProjectId: (projectId: string) => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<AuthState>(() => {
    const token = localStorage.getItem(STORAGE_KEYS.TOKEN);
    const userStr = localStorage.getItem(STORAGE_KEYS.USER);
    const projectStr = localStorage.getItem(STORAGE_KEYS.PROJECT);
    const envsStr = localStorage.getItem(STORAGE_KEYS.ENVIRONMENTS);
    const selectedProjectId = localStorage.getItem(STORAGE_KEYS.SELECTED_PROJECT);
    
    return {
      user: token && userStr ? JSON.parse(userStr) : null,
      currentProject: projectStr ? JSON.parse(projectStr) : null,
      environments: envsStr ? JSON.parse(envsStr) : [],
      selectedProjectId: selectedProjectId || null,
    };
  });

  const isAuthenticated = !!state.user;

  const login = useCallback((
    token: string,
    user: User,
    project?: Project,
    environments?: Environment[]
  ) => {
    localStorage.setItem(STORAGE_KEYS.TOKEN, token);
    localStorage.setItem(STORAGE_KEYS.USER, JSON.stringify(user));
    
    if (project) {
      localStorage.setItem(STORAGE_KEYS.PROJECT, JSON.stringify(project));
      if (project.api_key) {
        localStorage.setItem(STORAGE_KEYS.PROJECT_API_KEY, project.api_key);
      }
      localStorage.setItem(STORAGE_KEYS.SELECTED_PROJECT, project.id);
    }
    if (environments) {
      localStorage.setItem(STORAGE_KEYS.ENVIRONMENTS, JSON.stringify(environments));
    }
    
    setState({
      user,
      currentProject: project || null,
      environments: environments || [],
      selectedProjectId: project?.id || null,
    });
  }, []);

  const logout = useCallback(() => {
    Object.values(STORAGE_KEYS).forEach((key) => {
      localStorage.removeItem(key);
    });
    
    setState({
      user: null,
      currentProject: null,
      environments: [],
      selectedProjectId: null,
    });
  }, []);

  const setCurrentProject = useCallback((project: Project, environments: Environment[]) => {
    localStorage.setItem(STORAGE_KEYS.PROJECT, JSON.stringify(project));
    if (project.api_key) {
      localStorage.setItem(STORAGE_KEYS.PROJECT_API_KEY, project.api_key);
    }
    localStorage.setItem(STORAGE_KEYS.ENVIRONMENTS, JSON.stringify(environments));
    localStorage.setItem(STORAGE_KEYS.SELECTED_PROJECT, project.id);
    
    setState((prev) => ({
      ...prev,
      currentProject: project,
      environments,
      selectedProjectId: project.id,
    }));
  }, []);

  const setSelectedProjectId = useCallback((projectId: string) => {
    localStorage.setItem(STORAGE_KEYS.SELECTED_PROJECT, projectId);
    setState((prev) => ({
      ...prev,
      selectedProjectId: projectId,
    }));
  }, []);

  // Auto-restore selected project on mount if we have the ID but not the project
  useEffect(() => {
    const restoreProject = async () => {
      if (state.selectedProjectId && !state.currentProject && isAuthenticated) {
        // Project will be loaded by the component that needs it
        // This just ensures the ID is preserved
      }
    };
    restoreProject();
  }, [state.selectedProjectId, state.currentProject, isAuthenticated]);

  return (
    <AuthContext.Provider value={{
      ...state,
      isAuthenticated,
      login,
      logout,
      setCurrentProject,
      setSelectedProjectId,
    }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
