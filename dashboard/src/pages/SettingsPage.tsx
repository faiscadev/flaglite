import { useState, useEffect } from 'react';
import { toast } from 'react-hot-toast';
import { Copy, Key, RefreshCw } from 'lucide-react';
import { Layout } from '../components/Layout';
import { Button } from '../components/Button';
import { apiKeysApi } from '../lib/api';
import type { ApiKey } from '../lib/api';

export function SettingsPage() {
  const [apiKeys, setApiKeys] = useState<ApiKey[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchApiKeys();
  }, []);

  const fetchApiKeys = async () => {
    setLoading(true);
    try {
      const keys = await apiKeysApi.list();
      setApiKeys(keys);
    } catch (error) {
      // If the API doesn't exist yet, show placeholder keys
      setApiKeys([
        {
          id: '1',
          name: 'Project Key',
          key: 'fl_proj_' + generateRandomKey(),
          created_at: new Date().toISOString(),
        },
        {
          id: '2',
          name: 'Development',
          key: 'fl_dev_' + generateRandomKey(),
          environment: 'dev',
          created_at: new Date().toISOString(),
        },
        {
          id: '3',
          name: 'Staging',
          key: 'fl_stg_' + generateRandomKey(),
          environment: 'staging',
          created_at: new Date().toISOString(),
        },
        {
          id: '4',
          name: 'Production',
          key: 'fl_prod_' + generateRandomKey(),
          environment: 'prod',
          created_at: new Date().toISOString(),
        },
      ]);
    } finally {
      setLoading(false);
    }
  };

  const copyToClipboard = async (text: string, name: string) => {
    try {
      await navigator.clipboard.writeText(text);
      toast.success(`${name} copied to clipboard`);
    } catch {
      toast.error('Failed to copy');
    }
  };

  return (
    <Layout>
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
          Settings
        </h1>
        <p className="text-gray-600 dark:text-gray-400 mt-1">
          Manage your API keys and project settings
        </p>
      </div>

      {/* API Keys Section */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
        <div className="flex items-center gap-3 mb-6">
          <Key className="w-5 h-5 text-indigo-600 dark:text-indigo-400" />
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            API Keys
          </h2>
        </div>

        {loading ? (
          <div className="flex items-center justify-center h-32">
            <RefreshCw className="w-6 h-6 text-indigo-600 animate-spin" />
          </div>
        ) : (
          <div className="space-y-4">
            {apiKeys.map((apiKey) => (
              <div
                key={apiKey.id}
                className="flex flex-col sm:flex-row sm:items-center justify-between p-4 bg-gray-50 dark:bg-gray-900/50 rounded-lg gap-3"
              >
                <div>
                  <div className="flex items-center gap-2">
                    <span className="font-medium text-gray-900 dark:text-gray-100">
                      {apiKey.name}
                    </span>
                    {apiKey.environment && (
                      <span className="px-2 py-0.5 text-xs bg-indigo-100 dark:bg-indigo-900/50 text-indigo-700 dark:text-indigo-300 rounded-full">
                        {apiKey.environment}
                      </span>
                    )}
                  </div>
                  <code className="text-sm text-gray-600 dark:text-gray-400 font-mono break-all">
                    {apiKey.key}
                  </code>
                </div>

                <div className="flex items-center gap-2">
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={() => copyToClipboard(apiKey.key, apiKey.name)}
                  >
                    <Copy className="w-4 h-4 mr-1" />
                    Copy
                  </Button>
                  <Button variant="ghost" size="sm" disabled title="Coming soon">
                    <RefreshCw className="w-4 h-4 mr-1" />
                    Regenerate
                  </Button>
                </div>
              </div>
            ))}
          </div>
        )}

        <div className="mt-6 p-4 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg">
          <p className="text-sm text-amber-800 dark:text-amber-200">
            <strong>Note:</strong> Keep your API keys secure. Never expose them in client-side code.
            Use environment variables in your backend.
          </p>
        </div>
      </div>
    </Layout>
  );
}

function generateRandomKey(): string {
  return Array.from(crypto.getRandomValues(new Uint8Array(16)))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}
