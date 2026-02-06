import { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { RefreshCw, Copy, Check, Settings } from 'lucide-react';
import { Layout } from '../components/Layout';
import { Button } from '../components/Button';
import { projectsApi } from '../lib/api';
import { useAuth } from '../context/AuthContext';

export function ProjectSettingsPage() {
  const { projectId } = useParams<{ projectId: string }>();
  const navigate = useNavigate();
  const { currentProject, setCurrentProject, setSelectedProjectId } = useAuth();
  const [copiedApiKey, setCopiedApiKey] = useState(false);

  // Load project if not in context or if different project
  useEffect(() => {
    const loadProject = async () => {
      if (projectId && (!currentProject || currentProject.id !== projectId)) {
        try {
          const projects = await projectsApi.list();
          const project = projects.find((p) => p.id === projectId);
          if (project) {
            const envs = await projectsApi.getEnvironments(projectId);
            setCurrentProject(project, envs);
            setSelectedProjectId(projectId);
          } else {
            navigate('/projects');
          }
        } catch (err) {
          console.error('Failed to load project:', err);
          navigate('/projects');
        }
      }
    };
    loadProject();
  }, [currentProject, projectId, navigate, setCurrentProject, setSelectedProjectId]);

  const handleCopyApiKey = async () => {
    if (!currentProject) return;
    try {
      await navigator.clipboard.writeText(currentProject.api_key);
      setCopiedApiKey(true);
      setTimeout(() => setCopiedApiKey(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
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
        <h1 className="text-2xl font-bold text-zinc-900">Settings</h1>
        <p className="text-zinc-500 mt-1">
          Project settings for {currentProject.name}
        </p>
      </div>

      <div className="max-w-2xl space-y-6">
        {/* Project Info */}
        <div className="bg-white rounded-xl border border-zinc-200 p-6">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-lg bg-zinc-100 flex items-center justify-center">
              <Settings className="w-5 h-5 text-zinc-600" />
            </div>
            <div>
              <h2 className="font-semibold text-zinc-900">Project Information</h2>
              <p className="text-sm text-zinc-500">Basic project details</p>
            </div>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-zinc-700 mb-1">
                Project Name
              </label>
              <div className="px-3 py-2 bg-zinc-50 border border-zinc-200 rounded-lg text-zinc-900">
                {currentProject.name}
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-zinc-700 mb-1">
                Project ID
              </label>
              <div className="px-3 py-2 bg-zinc-50 border border-zinc-200 rounded-lg text-zinc-600 font-mono text-sm">
                {currentProject.id}
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-zinc-700 mb-1">
                Created
              </label>
              <div className="px-3 py-2 bg-zinc-50 border border-zinc-200 rounded-lg text-zinc-600">
                {new Date(currentProject.created_at).toLocaleString()}
              </div>
            </div>
          </div>
        </div>

        {/* API Key */}
        <div className="bg-white rounded-xl border border-zinc-200 p-6">
          <h2 className="font-semibold text-zinc-900 mb-4">Project API Key</h2>
          <p className="text-sm text-zinc-500 mb-4">
            Use this API key to authenticate requests to the FlagLite API for this project.
          </p>
          
          <div className="flex items-center gap-2">
            <code className="flex-1 px-3 py-2 bg-zinc-100 rounded-lg font-mono text-sm text-zinc-700 truncate">
              {currentProject.api_key}
            </code>
            <Button
              variant="secondary"
              size="sm"
              onClick={handleCopyApiKey}
            >
              {copiedApiKey ? (
                <>
                  <Check className="w-4 h-4 mr-1 text-green-600" />
                  Copied
                </>
              ) : (
                <>
                  <Copy className="w-4 h-4 mr-1" />
                  Copy
                </>
              )}
            </Button>
          </div>
        </div>

        {/* Danger Zone - placeholder for future features */}
        <div className="bg-white rounded-xl border border-red-200 p-6">
          <h2 className="font-semibold text-red-600 mb-2">Danger Zone</h2>
          <p className="text-sm text-zinc-500 mb-4">
            Destructive actions for this project. Be careful!
          </p>
          <Button variant="danger" disabled>
            Delete Project (Coming Soon)
          </Button>
        </div>
      </div>
    </Layout>
  );
}
