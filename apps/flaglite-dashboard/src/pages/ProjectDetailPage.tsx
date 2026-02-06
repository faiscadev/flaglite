import { useState, useEffect } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Plus, RefreshCw, Trash2, ChevronLeft, Flag as FlagIcon } from 'lucide-react';
import { Layout } from '../components/Layout';
import { Button } from '../components/Button';
import { Toggle } from '../components/Toggle';
import { Modal } from '../components/Modal';
import { Input } from '../components/Input';
import { Alert } from '../components/Alert';
import { flagsApi, projectsApi, getErrorMessage } from '../lib/api';
import { useAuth } from '../context/AuthContext';
import type { Flag } from '../types';

export function ProjectDetailPage() {
  const { projectId } = useParams<{ projectId: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { currentProject, environments, setCurrentProject } = useAuth();
  
  const [selectedEnv, setSelectedEnv] = useState('development');
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [deleteFlag, setDeleteFlag] = useState<string | null>(null);
  const [togglingFlags, setTogglingFlags] = useState<Set<string>>(new Set());
  
  // Create flag form state
  const [flagKey, setFlagKey] = useState('');
  const [flagName, setFlagName] = useState('');
  const [flagDescription, setFlagDescription] = useState('');
  const [error, setError] = useState('');

  // Load project if not in context
  useEffect(() => {
    const loadProject = async () => {
      if (!currentProject && projectId) {
        const projects = await projectsApi.list();
        const project = projects.find((p) => p.id === projectId);
        if (project) {
          const envs = await projectsApi.getEnvironments(projectId);
          setCurrentProject(project, envs);
        } else {
          navigate('/projects');
        }
      }
    };
    loadProject();
  }, [currentProject, projectId, navigate, setCurrentProject]);

  const { data: flags, isLoading, refetch, isRefetching } = useQuery({
    queryKey: ['flags', projectId],
    queryFn: () => flagsApi.list(projectId),
    enabled: !!currentProject && !!projectId,
  });

  const createMutation = useMutation({
    mutationFn: () =>
      flagsApi.create({
        key: flagKey,
        name: flagName,
        description: flagDescription || undefined,
      }, projectId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['flags', projectId] });
      setIsCreateModalOpen(false);
      resetCreateForm();
    },
    onError: (err) => {
      setError(getErrorMessage(err));
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (key: string) => flagsApi.delete(key, projectId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['flags', projectId] });
      setDeleteFlag(null);
    },
    onError: (err) => {
      setError(getErrorMessage(err));
    },
  });

  const resetCreateForm = () => {
    setFlagKey('');
    setFlagName('');
    setFlagDescription('');
    setError('');
  };

  const handleToggle = async (flag: Flag) => {
    const key = flag.key;
    setTogglingFlags((prev) => new Set(prev).add(key));

    try {
      await flagsApi.toggle(key, selectedEnv, projectId);
      await refetch();
    } catch (err) {
      console.error('Toggle failed:', err);
    } finally {
      setTogglingFlags((prev) => {
        const next = new Set(prev);
        next.delete(key);
        return next;
      });
    }
  };

  const handleCreateFlag = (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    createMutation.mutate();
  };

  const getFlagStatus = (flag: Flag): boolean => {
    return flag.environments[selectedEnv]?.enabled ?? false;
  };

  const getRollout = (flag: Flag): number => {
    return flag.environments[selectedEnv]?.rollout ?? 100;
  };

  if (!currentProject) {
    return (
      <Layout>
        <div className="flex items-center justify-center h-64">
          <RefreshCw className="w-8 h-8 text-green-600 animate-spin" />
        </div>
      </Layout>
    );
  }

  return (
    <Layout>
      {/* Header */}
      <div className="mb-6">
        <Link
          to="/projects"
          className="inline-flex items-center text-sm text-gray-600 hover:text-gray-900 mb-4"
        >
          <ChevronLeft className="w-4 h-4 mr-1" />
          Back to Projects
        </Link>
        
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div>
            <h1 className="text-2xl font-bold text-gray-900">
              {currentProject.name}
            </h1>
            <p className="text-gray-600 mt-1">
              Manage feature flags for this project
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

            <Button onClick={() => setIsCreateModalOpen(true)}>
              <Plus className="w-4 h-4 mr-2" />
              New Flag
            </Button>
          </div>
        </div>
      </div>

      {/* Environment Tabs */}
      <div className="border-b border-gray-200 mb-6">
        <nav className="flex gap-4 -mb-px overflow-x-auto">
          {environments.map((env) => (
            <button
              key={env.id}
              onClick={() => setSelectedEnv(env.name)}
              className={`
                whitespace-nowrap py-3 px-1 border-b-2 text-sm font-medium transition-colors cursor-pointer
                ${
                  selectedEnv === env.name
                    ? 'border-green-600 text-green-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }
              `}
            >
              {env.name.charAt(0).toUpperCase() + env.name.slice(1)}
            </button>
          ))}
        </nav>
      </div>

      {/* Flags List */}
      {isLoading ? (
        <div className="flex items-center justify-center h-64">
          <RefreshCw className="w-8 h-8 text-green-600 animate-spin" />
        </div>
      ) : !flags || flags.length === 0 ? (
        <div className="bg-white rounded-lg border border-gray-200 p-12 text-center">
          <FlagIcon className="w-12 h-12 text-gray-400 mx-auto mb-4" />
          <p className="text-gray-500 mb-4">No feature flags yet</p>
          <Button onClick={() => setIsCreateModalOpen(true)}>
            <Plus className="w-4 h-4 mr-2" />
            Create your first flag
          </Button>
        </div>
      ) : (
        <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Flag
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Key
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Status
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Rollout
                </th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {flags.map((flag) => (
                <tr key={flag.key} className="hover:bg-gray-50">
                  <td className="px-6 py-4">
                    <Link
                      to={`/projects/${projectId}/flags/${flag.key}`}
                      className="block"
                    >
                      <div className="text-sm font-medium text-gray-900 hover:text-green-600">
                        {flag.name}
                      </div>
                      {flag.description && (
                        <div className="text-sm text-gray-500 truncate max-w-xs">
                          {flag.description}
                        </div>
                      )}
                    </Link>
                  </td>
                  <td className="px-6 py-4">
                    <code className="px-2 py-1 text-xs bg-gray-100 rounded font-mono">
                      {flag.key}
                    </code>
                  </td>
                  <td className="px-6 py-4">
                    <Toggle
                      enabled={getFlagStatus(flag)}
                      onChange={() => handleToggle(flag)}
                      disabled={togglingFlags.has(flag.key)}
                    />
                  </td>
                  <td className="px-6 py-4">
                    <span className="text-sm text-gray-600">
                      {getRollout(flag)}%
                    </span>
                  </td>
                  <td className="px-6 py-4 text-right">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => setDeleteFlag(flag.key)}
                      className="text-red-600 hover:text-red-700 hover:bg-red-50"
                    >
                      <Trash2 className="w-4 h-4" />
                    </Button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Create Flag Modal */}
      <Modal
        isOpen={isCreateModalOpen}
        onClose={() => {
          setIsCreateModalOpen(false);
          resetCreateForm();
        }}
        title="Create Flag"
      >
        {error && (
          <div className="mb-4">
            <Alert type="error">{error}</Alert>
          </div>
        )}

        <form onSubmit={handleCreateFlag} className="space-y-4">
          <Input
            label="Flag Key"
            value={flagKey}
            onChange={(e) => setFlagKey(e.target.value.toLowerCase().replace(/[^a-z0-9-_]/g, '-'))}
            placeholder="enable-new-feature"
            required
          />
          <p className="text-xs text-gray-500 -mt-2">
            Use lowercase letters, numbers, hyphens, and underscores
          </p>

          <Input
            label="Display Name"
            value={flagName}
            onChange={(e) => setFlagName(e.target.value)}
            placeholder="Enable New Feature"
            required
          />

          <Input
            label="Description (optional)"
            value={flagDescription}
            onChange={(e) => setFlagDescription(e.target.value)}
            placeholder="Controls the new feature rollout"
          />

          <div className="flex justify-end gap-3 pt-2">
            <Button
              type="button"
              variant="secondary"
              onClick={() => {
                setIsCreateModalOpen(false);
                resetCreateForm();
              }}
            >
              Cancel
            </Button>
            <Button type="submit" loading={createMutation.isPending}>
              Create Flag
            </Button>
          </div>
        </form>
      </Modal>

      {/* Delete Confirmation Modal */}
      <Modal
        isOpen={!!deleteFlag}
        onClose={() => setDeleteFlag(null)}
        title="Delete Flag"
      >
        <p className="text-gray-600 mb-6">
          Are you sure you want to delete the flag{' '}
          <code className="px-1 py-0.5 bg-gray-100 rounded text-sm">
            {deleteFlag}
          </code>
          ? This action cannot be undone.
        </p>

        <div className="flex justify-end gap-3">
          <Button variant="secondary" onClick={() => setDeleteFlag(null)}>
            Cancel
          </Button>
          <Button
            variant="danger"
            loading={deleteMutation.isPending}
            onClick={() => deleteFlag && deleteMutation.mutate(deleteFlag)}
          >
            Delete Flag
          </Button>
        </div>
      </Modal>
    </Layout>
  );
}
