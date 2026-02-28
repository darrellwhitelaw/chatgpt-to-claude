import { useState } from 'react';
import { useKeychain } from '../hooks/useKeychain';
import { useAppStore } from '../store/appStore';

interface ApiKeyScreenProps {
  initialError?: string;
}

export function ApiKeyScreen({ initialError }: ApiKeyScreenProps) {
  const [key, setKey] = useState('');
  const [error, setError] = useState<string | null>(initialError ?? null);
  const [isLoading, setIsLoading] = useState(false);
  const { setApiKey } = useKeychain();
  const { setKeyStored } = useAppStore();

  const handleContinue = async () => {
    if (!key.trim()) {
      setError('Enter your Anthropic API key');
      return;
    }
    setIsLoading(true);
    setError(null);
    try {
      await setApiKey(key.trim());
      setKeyStored();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save key — try again');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="flex flex-col items-center gap-6 w-full max-w-sm px-6">
      <div className="flex flex-col gap-1.5 w-full">
        <label className="text-sm text-neutral-600 font-medium">
          Anthropic API key
        </label>
        <input
          type="password"
          value={key}
          onChange={(e) => setKey(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && handleContinue()}
          placeholder="sk-ant-..."
          className="w-full px-3 py-2 rounded-lg border border-neutral-200 text-sm
                     focus:outline-none focus:ring-2 focus:ring-neutral-300
                     bg-white text-neutral-800 placeholder-neutral-300"
          autoComplete="off"
          spellCheck={false}
        />
        {error && (
          <p className="text-xs text-red-500 mt-0.5">{error}</p>
        )}
      </div>
      <button
        onClick={handleContinue}
        disabled={isLoading}
        className="px-8 py-2.5 rounded-lg bg-neutral-900 text-white text-sm font-medium
                   hover:bg-neutral-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {isLoading ? 'Saving...' : 'Continue'}
      </button>
    </div>
  );
}
