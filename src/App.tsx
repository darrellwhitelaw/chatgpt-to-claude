import { openUrl } from '@tauri-apps/plugin-opener';
import { useAppStore } from './store/appStore';
import { useKeychain } from './hooks/useKeychain';
import { DropZone } from './components/DropZone';
import { ProgressView } from './components/ProgressView';
import { SummaryCard } from './components/SummaryCard';
import { ApiKeyScreen } from './screens/ApiKeyScreen';

export default function App() {
  const { phase, stage, error, summary, reset, setAwaitingKey, setKeyStored, clusterError } = useAppStore();
  const { getApiKey } = useKeychain();

  const handleSummaryContinue = async () => {
    try {
      await getApiKey(); // succeeds if key exists in Keychain
      setKeyStored();    // skip ApiKeyScreen, go to cost estimation
    } catch {
      setAwaitingKey();  // no key stored — show ApiKeyScreen
    }
  };

  return (
    <div className="h-screen w-screen bg-white flex items-center justify-center pb-8">
      {phase === 'idle' && <DropZone />}
      {phase === 'parsing' && <ProgressView stage={stage} />}
      {phase === 'error' && <DropZone errorMessage={error ?? undefined} onReset={reset} />}
      {phase === 'complete' && summary && (
        <SummaryCard
          total={summary.total}
          earliestYear={summary.earliestYear}
          latestYear={summary.latestYear}
          onContinue={handleSummaryContinue}
          hasApiKey={false}
          onChangeKey={() => setAwaitingKey()}
        />
      )}

      {phase === 'awaiting-key' && (
        <ApiKeyScreen initialError={clusterError ?? undefined} />
      )}
      {phase === 'key-stored' && (
        // CostScreen renders here — Plan 02-04 adds this
        <ProgressView stage="Counting tokens..." />
      )}
      {phase === 'cost-ready' && (
        // CostScreen renders here — Plan 02-04 replaces this placeholder
        <ProgressView stage="Ready to cluster" />
      )}
      {phase === 'clustering' && (
        // ClusteringView renders here — Plan 02-05 replaces this placeholder
        <ProgressView stage={stage} />
      )}
      {phase === 'clustering-complete' && (
        // Phase 3 entry point — placeholder
        <ProgressView stage="Clustering complete" />
      )}

      {/* Footer */}
      <button
        onClick={() => openUrl('https://darrellwhitelaw.com')}
        className="absolute bottom-4 text-xs text-neutral-300 hover:text-neutral-500 transition-colors underline underline-offset-2"
      >
        made for me
      </button>
    </div>
  );
}
