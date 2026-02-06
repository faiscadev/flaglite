import { useState, useEffect } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { ChevronLeft, RefreshCw, Copy, Check } from 'lucide-react';
import { Layout } from '../components/Layout';
import { Toggle } from '../components/Toggle';
import { flagsApi, projectsApi } from '../lib/api';
import { useAuth } from '../context/AuthContext';

type SdkLanguage = 'javascript' | 'python' | 'go' | 'rust';

const SDK_LANGUAGES: { id: SdkLanguage; name: string }[] = [
  { id: 'javascript', name: 'JavaScript' },
  { id: 'python', name: 'Python' },
  { id: 'go', name: 'Go' },
  { id: 'rust', name: 'Rust' },
];

function generateCodeSnippet(
  language: SdkLanguage,
  flagKey: string,
  apiKey: string
): string {
  const apiUrl = import.meta.env.VITE_API_URL || 'https://api.flaglite.dev';

  switch (language) {
    case 'javascript':
      return `import { FlagLiteClient } from '@flaglite/sdk';

const client = new FlagLiteClient({
  apiKey: '${apiKey}',
  apiUrl: '${apiUrl}',
});

// Check if flag is enabled
const isEnabled = await client.isEnabled('${flagKey}');

if (isEnabled) {
  // Feature is enabled
  showNewFeature();
}

// With user ID for percentage rollouts
const isEnabledForUser = await client.isEnabled('${flagKey}', {
  userId: currentUser.id,
});`;

    case 'python':
      return `from flaglite import FlagLiteClient

client = FlagLiteClient(
    api_key="${apiKey}",
    api_url="${apiUrl}",
)

# Check if flag is enabled
is_enabled = client.is_enabled("${flagKey}")

if is_enabled:
    # Feature is enabled
    show_new_feature()

# With user ID for percentage rollouts
is_enabled_for_user = client.is_enabled(
    "${flagKey}",
    user_id=current_user.id,
)`;

    case 'go':
      return `package main

import (
    "github.com/faiscadev/flaglite-go"
)

func main() {
    client := flaglite.NewClient(flaglite.Config{
        APIKey: "${apiKey}",
        APIURL: "${apiUrl}",
    })

    // Check if flag is enabled
    isEnabled, err := client.IsEnabled("${flagKey}")
    if err != nil {
        log.Fatal(err)
    }

    if isEnabled {
        // Feature is enabled
        showNewFeature()
    }

    // With user ID for percentage rollouts
    isEnabledForUser, err := client.IsEnabled(
        "${flagKey}",
        flaglite.WithUserID(currentUser.ID),
    )
}`;

    case 'rust':
      return `use flaglite::FlagLiteClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = FlagLiteClient::new(
        "${apiKey}",
        Some("${apiUrl}"),
    );

    // Check if flag is enabled
    let is_enabled = client.is_enabled("${flagKey}").await?;

    if is_enabled {
        // Feature is enabled
        show_new_feature();
    }

    // With user ID for percentage rollouts
    let is_enabled_for_user = client
        .is_enabled_with_user("${flagKey}", &current_user.id)
        .await?;

    Ok(())
}`;
  }
}

export function FlagDetailPage() {
  const { projectId, flagKey } = useParams<{
    projectId: string;
    flagKey: string;
  }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { currentProject, environments, setCurrentProject } = useAuth();

  const [selectedEnv, setSelectedEnv] = useState('development');
  const [selectedSdk, setSelectedSdk] = useState<SdkLanguage>('javascript');
  const [copied, setCopied] = useState(false);
  const [toggling, setToggling] = useState(false);

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

  const { data: flag, isLoading, refetch } = useQuery({
    queryKey: ['flag', projectId, flagKey],
    queryFn: () => flagsApi.get(flagKey!, projectId),
    enabled: !!currentProject && !!flagKey && !!projectId,
  });

  const handleToggle = async () => {
    if (!flag || !projectId) return;
    setToggling(true);

    try {
      await flagsApi.toggle(flag.key, selectedEnv, projectId);
      await refetch();
      queryClient.invalidateQueries({ queryKey: ['flags', projectId] });
    } catch (err) {
      console.error('Toggle failed:', err);
    } finally {
      setToggling(false);
    }
  };

  const handleCopyCode = async () => {
    if (!flag || !currentProject) return;
    const selectedEnvObj = environments.find((e) => e.name === selectedEnv);
    const apiKey = selectedEnvObj?.api_key || currentProject.api_key;
    const code = generateCodeSnippet(selectedSdk, flag.key, apiKey);
    
    await navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const getFlagStatus = (): boolean => {
    return flag?.environments[selectedEnv]?.enabled ?? false;
  };

  const getRollout = (): number => {
    return flag?.environments[selectedEnv]?.rollout ?? 100;
  };

  if (!currentProject || isLoading) {
    return (
      <Layout>
        <div className="flex items-center justify-center h-64">
          <RefreshCw className="w-8 h-8 text-green-600 animate-spin" />
        </div>
      </Layout>
    );
  }

  if (!flag) {
    return (
      <Layout>
        <div className="text-center py-12">
          <p className="text-gray-500">Flag not found</p>
          <Link
            to={`/projects/${projectId}`}
            className="text-green-600 hover:underline mt-2 inline-block"
          >
            Back to flags
          </Link>
        </div>
      </Layout>
    );
  }

  const selectedEnvObj = environments.find((e) => e.name === selectedEnv);
  const apiKey = selectedEnvObj?.api_key || currentProject.api_key;
  const codeSnippet = generateCodeSnippet(selectedSdk, flag.key, apiKey);

  return (
    <Layout>
      {/* Header */}
      <div className="mb-6">
        <Link
          to={`/projects/${projectId}`}
          className="inline-flex items-center text-sm text-gray-600 hover:text-gray-900 mb-4"
        >
          <ChevronLeft className="w-4 h-4 mr-1" />
          Back to Flags
        </Link>

        <div className="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-4">
          <div>
            <h1 className="text-2xl font-bold text-gray-900">{flag.name}</h1>
            {flag.description && (
              <p className="text-gray-600 mt-1">{flag.description}</p>
            )}
            <code className="inline-block mt-2 px-2 py-1 text-sm bg-gray-100 rounded font-mono">
              {flag.key}
            </code>
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

      <div className="grid gap-6 lg:grid-cols-2">
        {/* Flag Status Card */}
        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">
            Flag Status
          </h2>

          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-gray-700">Enabled</span>
              <Toggle
                enabled={getFlagStatus()}
                onChange={handleToggle}
                disabled={toggling}
              />
            </div>

            <div className="flex items-center justify-between">
              <span className="text-gray-700">Rollout Percentage</span>
              <span className="text-gray-900 font-medium">{getRollout()}%</span>
            </div>

            <div className="pt-4 border-t border-gray-200">
              <p className="text-sm text-gray-500">
                Environment:{' '}
                <span className="font-medium text-gray-900">
                  {selectedEnv.charAt(0).toUpperCase() + selectedEnv.slice(1)}
                </span>
              </p>
            </div>
          </div>
        </div>

        {/* Environment API Key */}
        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">
            Environment API Key
          </h2>

          <p className="text-sm text-gray-600 mb-3">
            Use this API key for the <strong>{selectedEnv}</strong> environment:
          </p>

          <div className="relative">
            <code className="block w-full p-3 bg-gray-50 border border-gray-200 rounded-lg text-sm font-mono break-all pr-12">
              {apiKey}
            </code>
            <button
              onClick={async () => {
                await navigator.clipboard.writeText(apiKey);
                setCopied(true);
                setTimeout(() => setCopied(false), 2000);
              }}
              className="absolute right-2 top-1/2 -translate-y-1/2 p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-200 rounded cursor-pointer"
              title="Copy to clipboard"
            >
              {copied ? (
                <Check className="w-4 h-4 text-green-600" />
              ) : (
                <Copy className="w-4 h-4" />
              )}
            </button>
          </div>
        </div>
      </div>

      {/* SDK Code Snippets */}
      <div className="mt-6 bg-white rounded-lg border border-gray-200 p-6">
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-4">
          <h2 className="text-lg font-semibold text-gray-900">
            SDK Integration
          </h2>

          <div className="flex items-center gap-2">
            {SDK_LANGUAGES.map((lang) => (
              <button
                key={lang.id}
                onClick={() => setSelectedSdk(lang.id)}
                className={`
                  px-3 py-1.5 text-sm font-medium rounded-lg transition-colors cursor-pointer
                  ${
                    selectedSdk === lang.id
                      ? 'bg-green-100 text-green-700'
                      : 'text-gray-600 hover:bg-gray-100'
                  }
                `}
              >
                {lang.name}
              </button>
            ))}
          </div>
        </div>

        <div className="relative">
          <pre className="p-4 bg-gray-900 text-gray-100 rounded-lg overflow-x-auto text-sm">
            <code>{codeSnippet}</code>
          </pre>

          <button
            onClick={handleCopyCode}
            className="absolute top-3 right-3 p-2 text-gray-400 hover:text-white bg-gray-800 hover:bg-gray-700 rounded-lg transition-colors cursor-pointer"
            title="Copy code"
          >
            {copied ? (
              <Check className="w-4 h-4 text-green-400" />
            ) : (
              <Copy className="w-4 h-4" />
            )}
          </button>
        </div>

        <p className="mt-4 text-sm text-gray-500">
          Install the SDK for your language and use the code above to check this
          flag in your application.
        </p>
      </div>
    </Layout>
  );
}
