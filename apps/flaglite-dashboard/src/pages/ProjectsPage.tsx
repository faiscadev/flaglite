import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Plus, FolderKanban, RefreshCw, ArrowRight } from 'lucide-react';
import { Layout } from '../components/Layout';
import { Button } from '../components/Button';
import { Modal } from '../components/Modal';
import { Input } from '../components/Input';
import { Alert } from '../components/Alert';
import { projectsApi, getErrorMessage } from '../lib/api';
import { useAuth } from '../context/AuthContext';

export function ProjectsPage() {
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [projectName, setProjectName] = useState('');
  const [error, setError] = useState('');
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { setCurrentProject } = useAuth();

  const { data: projects, isLoading, refetch, isRefetching } = useQuery({
    queryKey: ['projects'],
    queryFn: projectsApi.list,
  });

  const createMutation = useMutation({
    mutationFn: () => projectsApi.create({ name: projectName }),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['projects'] });
      setCurrentProject(data.project, data.environments);
      setIsModalOpen(false);
      setProjectName('');
      navigate(`/projects/${data.project.id}`);
    },
    onError: (err) => {
      setError(getErrorMessage(err));
    },
  });

  const handleCreateProject = (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    createMutation.mutate();
  };

  const handleSelectProject = async (projectId: string) => {
    const project = projects?.find((p) => p.id === projectId);
    if (project) {
      const environments = await projectsApi.getEnvironments(projectId);
      setCurrentProject(project, environments);
      navigate(`/projects/${projectId}`);
    }
  };

  return (
    <Layout>
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-8">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Projects</h1>
          <p className="text-gray-600 mt-1">
            Manage your feature flag projects
          </p>
        </div>

        <div className="flex items-center gap-3">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => refetch()}
            disabled={isRefetching}
          >
            <RefreshCw
              className={`w-4 h-4 ${isRefetching ? 'animate-spin' : ''}`}
            />
          </Button>

          <Button onClick={() => setIsModalOpen(true)}>
            <Plus className="w-4 h-4 mr-2" />
            New Project
          </Button>
        </div>
      </div>

      {/* Projects Grid */}
      {isLoading ? (
        <div className="flex items-center justify-center h-64">
          <RefreshCw className="w-8 h-8 text-green-600 animate-spin" />
        </div>
      ) : !projects || projects.length === 0 ? (
        <div className="bg-white rounded-lg border border-gray-200 p-12 text-center">
          <FolderKanban className="w-12 h-12 text-gray-400 mx-auto mb-4" />
          <p className="text-gray-500 mb-4">No projects yet</p>
          <Button onClick={() => setIsModalOpen(true)}>
            <Plus className="w-4 h-4 mr-2" />
            Create your first project
          </Button>
        </div>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {projects.map((project) => (
            <button
              key={project.id}
              onClick={() => handleSelectProject(project.id)}
              className="bg-white rounded-lg border border-gray-200 p-6 text-left hover:border-green-300 hover:shadow-md transition-all group cursor-pointer"
            >
              <div className="flex items-start justify-between">
                <div className="flex-1 min-w-0">
                  <h3 className="font-semibold text-gray-900 truncate">
                    {project.name}
                  </h3>
                  <p className="text-xs text-gray-500 mt-1">
                    Created {new Date(project.created_at).toLocaleDateString()}
                  </p>
                </div>
                <ArrowRight className="w-5 h-5 text-gray-400 group-hover:text-green-600 transition-colors" />
              </div>
            </button>
          ))}
        </div>
      )}

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

          <p className="text-xs text-gray-500">
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
    </Layout>
  );
}
