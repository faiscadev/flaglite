import { useState, useRef, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { ChevronsUpDown, Check, Plus } from 'lucide-react';
import { projectsApi, getErrorMessage } from '../lib/api';
import { useAuth } from '../context/AuthContext';
import { Modal } from './Modal';
import { Input } from './Input';
import { Button } from './Button';
import { Alert } from './Alert';
import type { Project } from '../types';

export function ProjectSelector() {
  const [isOpen, setIsOpen] = useState(false);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [projectName, setProjectName] = useState('');
  const [error, setError] = useState('');
  const dropdownRef = useRef<HTMLDivElement>(null);
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { currentProject, setCurrentProject, selectedProjectId, setSelectedProjectId } = useAuth();

  const { data: projects } = useQuery({
    queryKey: ['projects'],
    queryFn: projectsApi.list,
  });

  const createMutation = useMutation({
    mutationFn: () => projectsApi.create({ name: projectName }),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['projects'] });
      setCurrentProject(data.project, data.environments);
      setSelectedProjectId(data.project.id);
      setIsModalOpen(false);
      setProjectName('');
      setError('');
      navigate(`/projects/${data.project.id}/flags`);
    },
    onError: (err) => {
      setError(getErrorMessage(err));
    },
  });

  // Click outside to close
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const handleSelectProject = async (project: Project) => {
    try {
      const environments = await projectsApi.getEnvironments(project.id);
      setCurrentProject(project, environments);
      setSelectedProjectId(project.id);
      setIsOpen(false);
      navigate(`/projects/${project.id}/flags`);
    } catch (err) {
      console.error('Failed to load project environments:', err);
    }
  };

  const handleCreateProject = (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    createMutation.mutate();
  };

  const handleNewProjectClick = () => {
    setIsOpen(false);
    setIsModalOpen(true);
  };

  // Count flags for current project (we don't have this data readily available, so show nothing or fetch)
  const flagCount = null; // Could be enhanced with a query

  return (
    <>
      <div className="border-b border-zinc-200 relative" ref={dropdownRef}>
        <button
          onClick={() => setIsOpen(!isOpen)}
          className="w-full px-4 py-3 flex items-center gap-3 hover:bg-zinc-50 transition-colors cursor-pointer"
        >
          {/* Project Avatar */}
          <div className="w-8 h-8 rounded-lg bg-green-600 flex items-center justify-center text-white font-semibold text-sm">
            {currentProject ? currentProject.name.charAt(0).toUpperCase() : '?'}
          </div>
          {/* Project Name */}
          <div className="flex-1 text-left">
            <div className="text-sm font-medium text-zinc-900 truncate">
              {currentProject ? currentProject.name : 'Select project'}
            </div>
            {flagCount !== null && (
              <div className="text-xs text-zinc-500">{flagCount} flags</div>
            )}
          </div>
          {/* Chevron */}
          <ChevronsUpDown className="w-4 h-4 text-zinc-400" />
        </button>

        {/* Dropdown */}
        {isOpen && (
          <div className="absolute left-2 right-2 mt-1 bg-white border border-zinc-200 rounded-lg shadow-lg z-50">
            <div className="p-2">
              <div className="px-2 py-1.5 text-xs font-medium text-zinc-500 uppercase tracking-wide">
                Projects
              </div>

              {/* Project List */}
              {projects?.map((project) => {
                const isSelected = project.id === selectedProjectId;
                return (
                  <button
                    key={project.id}
                    onClick={() => handleSelectProject(project)}
                    className={`w-full px-2 py-2 flex items-center gap-3 rounded-lg cursor-pointer ${
                      isSelected
                        ? 'bg-green-50 text-green-700'
                        : 'hover:bg-zinc-50 text-zinc-700'
                    }`}
                  >
                    <div
                      className={`w-7 h-7 rounded-md flex items-center justify-center font-medium text-xs ${
                        isSelected
                          ? 'bg-green-600 text-white'
                          : 'bg-zinc-200 text-zinc-600'
                      }`}
                    >
                      {project.name.charAt(0).toUpperCase()}
                    </div>
                    <span className="text-sm truncate flex-1 text-left">
                      {project.name}
                    </span>
                    {isSelected && <Check className="w-4 h-4" />}
                  </button>
                );
              })}

              {/* Divider */}
              <div className="my-2 border-t border-zinc-100"></div>

              {/* New Project */}
              <button
                onClick={handleNewProjectClick}
                className="w-full px-2 py-2 flex items-center gap-3 rounded-lg hover:bg-zinc-50 text-zinc-600 cursor-pointer"
              >
                <div className="w-7 h-7 rounded-md border-2 border-dashed border-zinc-300 flex items-center justify-center">
                  <Plus className="w-4 h-4 text-zinc-400" />
                </div>
                <span className="text-sm">New Project</span>
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Create Project Modal */}
      <Modal
        isOpen={isModalOpen}
        onClose={() => {
          setIsModalOpen(false);
          setProjectName('');
          setError('');
        }}
        title="Create Project"
      >
        {error && (
          <div className="mb-4">
            <Alert type="error">{error}</Alert>
          </div>
        )}

        <form onSubmit={handleCreateProject} className="space-y-4">
          <Input
            label="Project Name"
            value={projectName}
            onChange={(e) => setProjectName(e.target.value)}
            placeholder="my-awesome-app"
            required
          />

          <p className="text-xs text-zinc-500">
            A project will be created with 3 default environments: development,
            staging, and production.
          </p>

          <div className="flex justify-end gap-3 pt-2">
            <Button
              type="button"
              variant="secondary"
              onClick={() => {
                setIsModalOpen(false);
                setProjectName('');
                setError('');
              }}
            >
              Cancel
            </Button>
            <Button type="submit" loading={createMutation.isPending}>
              Create Project
            </Button>
          </div>
        </form>
      </Modal>
    </>
  );
}
