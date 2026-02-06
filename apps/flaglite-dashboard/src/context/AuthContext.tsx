/* eslint-disable react-refresh/only-export-components */
import { createContext, useContext, useState, useCallback, type ReactNode } from 'react';
import type { User, Project, Environment } from '../types';

interface AuthState {
  user: User | null;
  currentProject: Project | null;
  environments: Environment[];
}

interface AuthContextType extends AuthState {
  isAuthenticated: boolean;
  login: (token: string, user: User, project?: Project, environments?: Environment[]) => void;
  logout: () => void;
  setCurrentProject: (project: Project, environments: Environment[]) => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<AuthState>(() => {
    const token = localStorage.getItem('flaglite_token');
    const userStr = localStorage.getItem('flaglite_user');
    const projectStr = localStorage.getItem('flaglite_project');
    const envsStr = localStorage.getItem('flaglite_environments');
    
    return {
      user: token && userStr ? JSON.parse(userStr) : null,
      currentProject: projectStr ? JSON.parse(projectStr) : null,
      environments: envsStr ? JSON.parse(envsStr) : [],
    };
  });

  const isAuthenticated = !!state.user;

  const login = useCallback((
    token: string,
    user: User,
    project?: Project,
    environments?: Environment[]
  ) => {
    localStorage.setItem('flaglite_token', token);
    localStorage.setItem('flaglite_user', JSON.stringify(user));
    
    if (project) {
      localStorage.setItem('flaglite_project', JSON.stringify(project));
      localStorage.setItem('flaglite_project_api_key', project.api_key);
    }
    if (environments) {
      localStorage.setItem('flaglite_environments', JSON.stringify(environments));
    }
    
    setState({
      user,
      currentProject: project || null,
      environments: environments || [],
    });
  }, []);

  const logout = useCallback(() => {
    localStorage.removeItem('flaglite_token');
    localStorage.removeItem('flaglite_user');
    localStorage.removeItem('flaglite_project');
    localStorage.removeItem('flaglite_project_api_key');
    localStorage.removeItem('flaglite_environments');
    
    setState({
      user: null,
      currentProject: null,
      environments: [],
    });
  }, []);

  const setCurrentProject = useCallback((project: Project, environments: Environment[]) => {
    localStorage.setItem('flaglite_project', JSON.stringify(project));
    localStorage.setItem('flaglite_project_api_key', project.api_key);
    localStorage.setItem('flaglite_environments', JSON.stringify(environments));
    
    setState((prev) => ({
      ...prev,
      currentProject: project,
      environments,
    }));
  }, []);

  return (
    <AuthContext.Provider value={{
      ...state,
      isAuthenticated,
      login,
      logout,
      setCurrentProject,
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
