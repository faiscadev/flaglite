import { useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { RefreshCw, Globe, Copy, Check } from 'lucide-react';
import { Layout } from '../components/Layout';
import { Button } from '../components/Button';
import { projectsApi } from '../lib/api';
import { useAuth } from '../context/AuthContext';
import { useState } from 'react';

export function ProjectEnvironmentsPage() {
  const { projectId } = useParams<{ projectId: string }>();
  const navigate = useNavigate();
  const { currentProject, environments, setCurrentProject, setSelectedProjectId } = useAuth();
  const [copiedKey, setCopiedKey] = useState<string | null>(null);

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

  const handleCopyApiKey = async (apiKey: string, envName: string) => {
    try {
      await navigator.clipboard.writeText(apiKey);
      setCopiedKey(envName);
      setTimeout(() => setCopiedKey(null), 2000);
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
        <h1 className="text-2xl font-bold text-zinc-900">Environments</h1>
        <p className="text-zinc-500 mt-1">
          Manage environments for {currentProject.name}
        </p>
      </div>

      {/* Environments Grid */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {environments.map((env) => (
          <div
            key={env.id}
            className="bg-white rounded-xl border border-zinc-200 p-6"
          >
            <div className="flex items-center gap-3 mb-4">
              <div className="w-10 h-10 rounded-lg bg-zinc-100 flex items-center justify-center">
                <Globe className="w-5 h-5 text-zinc-600" />
              </div>
              <div>
                <h3 className="font-semibold text-zinc-900 capitalize">
                  {env.name}
                </h3>
                <p className="text-xs text-zinc-500">
                  Created {new Date(env.created_at).toLocaleDateString()}
                </p>
              </div>
            </div>

            <div className="space-y-3">
              <div>
                <label className="text-xs font-medium text-zinc-500 uppercase tracking-wide">
                  API Key
                </label>
                <div className="mt-1 flex items-center gap-2">
                  <code className="flex-1 px-3 py-2 text-xs bg-zinc-100 rounded-lg font-mono truncate">
                    {env.api_key.substring(0, 20)}...
                  </code>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleCopyApiKey(env.api_key, env.name)}
                    className="shrink-0"
                  >
                    {copiedKey === env.name ? (
                      <Check className="w-4 h-4 text-green-600" />
                    ) : (
                      <Copy className="w-4 h-4" />
                    )}
                  </Button>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>

      {environments.length === 0 && (
        <div className="bg-white rounded-xl border border-zinc-200 p-12 text-center">
          <Globe className="w-12 h-12 text-zinc-400 mx-auto mb-4" />
          <p className="text-zinc-500">No environments found</p>
        </div>
      )}
    </Layout>
  );
}
