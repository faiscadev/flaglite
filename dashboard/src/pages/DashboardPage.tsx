import { useState, useEffect } from 'react';
import { toast } from 'react-hot-toast';
import { Plus, RefreshCw, FolderOpen } from 'lucide-react';
import { Layout } from '../components/Layout';
import { Button } from '../components/Button';
import { Toggle } from '../components/Toggle';
import { CreateFlagModal } from './CreateFlagModal';
import { flagsApi } from '../lib/api';
import { useProject } from '../context/ProjectContext';
import type { Flag } from '../lib/api';

// Environment options matching the API
const ENVIRONMENTS = [
  { value: 'development', label: 'Development' },
  { value: 'staging', label: 'Staging' },
  { value: 'production', label: 'Production' },
] as const;

export function DashboardPage() {
  const { currentProject, projects, selectProject, loading: projectsLoading } = useProject();
  const [flags, setFlags] = useState<Flag[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [selectedEnv, setSelectedEnv] = useState<string>('development');
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [togglingFlags, setTogglingFlags] = useState<Set<string>>(new Set());

  const fetchFlags = async (showLoader = true) => {
    if (!currentProject) {
      setFlags([]);
      setLoading(false);
      return;
    }

    if (showLoader) setLoading(true);
    else setRefreshing(true);

    try {
      const data = await flagsApi.list(currentProject.id, selectedEnv);
      setFlags(data);
    } catch (error) {
      toast.error('Failed to load flags');
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  };

  useEffect(() => {
    if (currentProject) {
      fetchFlags();
    }
  }, [currentProject, selectedEnv]);

  const handleToggle = async (flag: Flag) => {
    if (!currentProject) return;
    
    const key = flag.key;
    setTogglingFlags((prev) => new Set(prev).add(key));

    try {
      await flagsApi.toggle(currentProject.id, key, selectedEnv);
      await fetchFlags(false);
      toast.success(`Flag "${flag.name}" toggled`);
    } catch (error) {
      toast.error('Failed to toggle flag');
    } finally {
      setTogglingFlags((prev) => {
        const next = new Set(prev);
        next.delete(key);
        return next;
      });
    }
  };

  // Show loading state while projects are loading
  if (projectsLoading) {
    return (
      <Layout>
        <div className="flex items-center justify-center h-64">
          <RefreshCw className="w-8 h-8 text-indigo-600 animate-spin" />
        </div>
      </Layout>
    );
  }

  // Show message if no projects exist
  if (!currentProject && projects.length === 0) {
    return (
      <Layout>
        <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-12 text-center">
          <FolderOpen className="w-12 h-12 text-gray-400 mx-auto mb-4" />
          <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-2">
            No Projects Yet
          </h2>
          <p className="text-gray-500 dark:text-gray-400 mb-4">
            Create a project to start managing feature flags.
          </p>
        </div>
      </Layout>
    );
  }

  return (
    <Layout>
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-8">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            Feature Flags
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mt-1">
            Manage your feature flags across environments
          </p>
        </div>

        <div className="flex items-center gap-3">
          {/* Project Selector */}
          {projects.length > 1 && (
            <select
              value={currentProject?.id || ''}
              onChange={(e) => {
                const project = projects.find(p => p.id === e.target.value);
                if (project) selectProject(project);
              }}
              className="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
            >
              {projects.map((project) => (
                <option key={project.id} value={project.id}>
                  {project.name}
                </option>
              ))}
            </select>
          )}

          {/* Environment Selector */}
          <select
            value={selectedEnv}
            onChange={(e) => setSelectedEnv(e.target.value)}
            className="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
          >
            {ENVIRONMENTS.map((env) => (
              <option key={env.value} value={env.value}>
                {env.label}
              </option>
            ))}
          </select>

          <Button
            variant="ghost"
            size="sm"
            onClick={() => fetchFlags(false)}
            disabled={refreshing}
          >
            <RefreshCw className={`w-4 h-4 ${refreshing ? 'animate-spin' : ''}`} />
          </Button>

          <Button onClick={() => setIsModalOpen(true)}>
            <Plus className="w-4 h-4 mr-2" />
            Create Flag
          </Button>
        </div>
      </div>

      {/* Current Project Badge */}
      {currentProject && projects.length === 1 && (
        <div className="mb-4">
          <span className="inline-flex items-center px-3 py-1 rounded-full text-sm bg-indigo-100 dark:bg-indigo-900/50 text-indigo-700 dark:text-indigo-300">
            <FolderOpen className="w-4 h-4 mr-2" />
            {currentProject.name}
          </span>
        </div>
      )}

      {/* Flags Table */}
      {loading ? (
        <div className="flex items-center justify-center h-64">
          <RefreshCw className="w-8 h-8 text-indigo-600 animate-spin" />
        </div>
      ) : flags.length === 0 ? (
        <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-12 text-center">
          <p className="text-gray-500 dark:text-gray-400 mb-4">
            No feature flags yet
          </p>
          <Button onClick={() => setIsModalOpen(true)}>
            <Plus className="w-4 h-4 mr-2" />
            Create your first flag
          </Button>
        </div>
      ) : (
        <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
          <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
            <thead className="bg-gray-50 dark:bg-gray-900/50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  Name
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  Key
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  Status
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200 dark:divide-gray-700">
              {flags.map((flag) => (
                <tr key={flag.id} className="hover:bg-gray-50 dark:hover:bg-gray-700/50">
                  <td className="px-6 py-4">
                    <div className="text-sm font-medium text-gray-900 dark:text-gray-100">
                      {flag.name}
                    </div>
                    {flag.description && (
                      <div className="text-sm text-gray-500 dark:text-gray-400">
                        {flag.description}
                      </div>
                    )}
                  </td>
                  <td className="px-6 py-4">
                    <code className="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-700 rounded">
                      {flag.key}
                    </code>
                  </td>
                  <td className="px-6 py-4">
                    <Toggle
                      enabled={flag.enabled}
                      onChange={() => handleToggle(flag)}
                      disabled={togglingFlags.has(flag.key)}
                    />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Create Flag Modal */}
      {currentProject && (
        <CreateFlagModal
          isOpen={isModalOpen}
          onClose={() => setIsModalOpen(false)}
          onCreated={() => {
            setIsModalOpen(false);
            fetchFlags(false);
          }}
          projectId={currentProject.id}
        />
      )}
    </Layout>
  );
}
