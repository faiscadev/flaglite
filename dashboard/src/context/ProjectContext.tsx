import { createContext, useContext, useState, useEffect, useCallback } from 'react';
import type { ReactNode } from 'react';
import { projectsApi, type Project } from '../lib/api';
import { useAuth } from './AuthContext';

interface ProjectContextType {
  projects: Project[];
  currentProject: Project | null;
  loading: boolean;
  error: string | null;
  selectProject: (project: Project) => void;
  refreshProjects: () => Promise<void>;
}

const ProjectContext = createContext<ProjectContextType | undefined>(undefined);

export function ProjectProvider({ children }: { children: ReactNode }) {
  const { isAuthenticated } = useAuth();
  const [projects, setProjects] = useState<Project[]>([]);
  const [currentProject, setCurrentProject] = useState<Project | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refreshProjects = useCallback(async () => {
    if (!isAuthenticated) {
      setProjects([]);
      setCurrentProject(null);
      return;
    }

    setLoading(true);
    setError(null);
    
    try {
      const projectList = await projectsApi.list();
      setProjects(projectList);
      
      // If no current project selected, select the first one
      if (!currentProject && projectList.length > 0) {
        // Try to restore from localStorage
        const savedProjectId = localStorage.getItem('currentProjectId');
        const savedProject = projectList.find(p => p.id === savedProjectId);
        setCurrentProject(savedProject || projectList[0]);
      } else if (currentProject) {
        // Verify current project still exists
        const stillExists = projectList.find(p => p.id === currentProject.id);
        if (!stillExists && projectList.length > 0) {
          setCurrentProject(projectList[0]);
        }
      }
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to load projects');
    } finally {
      setLoading(false);
    }
  }, [isAuthenticated, currentProject]);

  const selectProject = useCallback((project: Project) => {
    setCurrentProject(project);
    localStorage.setItem('currentProjectId', project.id);
  }, []);

  useEffect(() => {
    refreshProjects();
  }, [isAuthenticated]);

  return (
    <ProjectContext.Provider
      value={{
        projects,
        currentProject,
        loading,
        error,
        selectProject,
        refreshProjects,
      }}
    >
      {children}
    </ProjectContext.Provider>
  );
}

export function useProject() {
  const context = useContext(ProjectContext);
  if (context === undefined) {
    throw new Error('useProject must be used within a ProjectProvider');
  }
  return context;
}
