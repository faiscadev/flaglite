import { FlagIcon } from '../components/FlagIcon';
import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { useMutation } from '@tanstack/react-query';
import { useAuth } from '../context/AuthContext';
import { authApi, getErrorMessage } from '../lib/api';
import { Button } from '../components/Button';
import { Input } from '../components/Input';
import { Alert } from '../components/Alert';
import { Copy, Check } from 'lucide-react';

export function SignupPage() {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [projectName, setProjectName] = useState('');
  const [error, setError] = useState('');
  const [apiKey, setApiKey] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const { login } = useAuth();
  const navigate = useNavigate();

  const mutation = useMutation({
    mutationFn: () =>
      authApi.signup({
        username: username || undefined,
        password,
        project_name: projectName || undefined,
      }),
    onSuccess: (data) => {
      // Show API key first - user needs to copy it
      setApiKey(data.api_key.key);
      // Store auth info for after they proceed
      sessionStorage.setItem('pending_auth', JSON.stringify({
        token: data.token,
        user: data.user,
        project: data.project,
        environments: data.environments,
      }));
    },
    onError: (err) => {
      setError(getErrorMessage(err));
    },
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    mutation.mutate();
  };

  const handleCopyApiKey = async () => {
    if (apiKey) {
      await navigator.clipboard.writeText(apiKey);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleProceed = () => {
    const pendingAuth = sessionStorage.getItem('pending_auth');
    if (pendingAuth) {
      const data = JSON.parse(pendingAuth);
      login(data.token, data.user, data.project, data.environments);
      sessionStorage.removeItem('pending_auth');
      navigate('/projects');
    }
  };

  // Show API key screen after successful signup
  if (apiKey) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 px-4">
        <div className="w-full max-w-md">
          <div className="bg-white rounded-xl shadow-lg p-8">
            <div className="text-center mb-6">
              <div className="w-12 h-12 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <Check className="w-6 h-6 text-green-600" />
              </div>
              <h2 className="text-xl font-semibold text-gray-900">
                Account Created!
              </h2>
              <p className="text-gray-600 mt-2 text-sm">
                Save your API key now. It won't be shown again.
              </p>
            </div>

            <div className="mb-6">
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Your API Key
              </label>
              <div className="relative">
                <code className="block w-full p-3 bg-gray-50 border border-gray-200 rounded-lg text-sm font-mono break-all pr-12">
                  {apiKey}
                </code>
                <button
                  onClick={handleCopyApiKey}
                  className="absolute right-2 top-1/2 -translate-y-1/2 p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-200 rounded"
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

            <Alert type="warning">
              This is the only time you'll see this API key. Store it securely!
            </Alert>

            <Button onClick={handleProceed} className="w-full mt-6">
              I've saved my API key
            </Button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 px-4">
      <div className="w-full max-w-md">
        {/* Logo */}
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold text-green-600"><FlagIcon className="w-8 h-8 inline-block mr-2" />FlagLite</h1>
          <p className="text-gray-600 mt-2">Simple feature flag management</p>
        </div>

        {/* Card */}
        <div className="bg-white rounded-xl shadow-lg p-8">
          <h2 className="text-xl font-semibold text-gray-900 mb-6">
            Create an account
          </h2>

          {error && (
            <div className="mb-4">
              <Alert type="error">{error}</Alert>
            </div>
          )}

          <form onSubmit={handleSubmit} className="space-y-4">
            <Input
              label="Username (optional)"
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              placeholder="cool-developer"
              autoComplete="username"
            />
            <p className="text-xs text-gray-500 -mt-2">
              Leave blank for an auto-generated username
            </p>

            <Input
              label="Password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
              minLength={8}
              placeholder="••••••••"
              autoComplete="new-password"
            />

            <Input
              label="First Project Name (optional)"
              type="text"
              value={projectName}
              onChange={(e) => setProjectName(e.target.value)}
              placeholder="my-awesome-app"
            />
            <p className="text-xs text-gray-500 -mt-2">
              Defaults to "default" if not provided
            </p>

            <Button
              type="submit"
              loading={mutation.isPending}
              className="w-full"
            >
              Sign up
            </Button>
          </form>

          <div className="mt-6 text-center">
            <Link
              to="/login"
              className="text-sm text-green-600 hover:underline"
            >
              Already have an account? Sign in
            </Link>
          </div>
        </div>
      </div>
    </div>
  );
}
