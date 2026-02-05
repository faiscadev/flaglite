import { useState, useEffect } from 'react';
import { toast } from 'react-hot-toast';
import { Modal } from '../components/Modal';
import { Button } from '../components/Button';
import { flagsApi } from '../lib/api';

interface CreateFlagModalProps {
  isOpen: boolean;
  onClose: () => void;
  onCreated: () => void;
}

function generateKey(name: string): string {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '');
}

export function CreateFlagModal({ isOpen, onClose, onCreated }: CreateFlagModalProps) {
  const [name, setName] = useState('');
  const [key, setKey] = useState('');
  const [description, setDescription] = useState('');
  const [rollout, setRollout] = useState(100);
  const [loading, setLoading] = useState(false);

  // Auto-generate key from name
  useEffect(() => {
    setKey(generateKey(name));
  }, [name]);

  const resetForm = () => {
    setName('');
    setKey('');
    setDescription('');
    setRollout(100);
  };

  const handleClose = () => {
    resetForm();
    onClose();
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);

    try {
      await flagsApi.create({
        name,
        key,
        description: description || undefined,
      });
      toast.success(`Flag "${name}" created!`);
      resetForm();
      onCreated();
    } catch (error: any) {
      const message = error.response?.data?.error || 'Failed to create flag';
      toast.error(message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Modal isOpen={isOpen} onClose={handleClose} title="Create Flag">
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label
            htmlFor="name"
            className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
          >
            Name
          </label>
          <input
            type="text"
            id="name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            required
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500"
            placeholder="New Feature"
          />
        </div>

        <div>
          <label
            htmlFor="key"
            className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
          >
            Key
          </label>
          <input
            type="text"
            id="key"
            value={key}
            onChange={(e) => setKey(e.target.value)}
            required
            pattern="[a-z0-9-]+"
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 font-mono text-sm"
            placeholder="new-feature"
          />
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            Lowercase letters, numbers, and hyphens only
          </p>
        </div>

        <div>
          <label
            htmlFor="description"
            className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
          >
            Description
            <span className="text-gray-400 font-normal"> (optional)</span>
          </label>
          <textarea
            id="description"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={2}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 resize-none"
            placeholder="What does this flag control?"
          />
        </div>

        <div>
          <label
            htmlFor="rollout"
            className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
          >
            Rollout Percentage: {rollout}%
          </label>
          <input
            type="range"
            id="rollout"
            min="0"
            max="100"
            value={rollout}
            onChange={(e) => setRollout(Number(e.target.value))}
            className="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer accent-indigo-600"
          />
          <div className="flex justify-between text-xs text-gray-400 mt-1">
            <span>0%</span>
            <span>50%</span>
            <span>100%</span>
          </div>
        </div>

        <div className="flex gap-3 pt-4">
          <Button type="button" variant="secondary" onClick={handleClose} className="flex-1">
            Cancel
          </Button>
          <Button type="submit" loading={loading} className="flex-1">
            Create Flag
          </Button>
        </div>
      </form>
    </Modal>
  );
}
