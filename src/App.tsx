import { useEffect } from 'react';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useAppStore } from './store/appStore';
import { useKeychain } from './hooks/useKeychain';
import { useCluster } from './hooks/useCluster';
import { DropZone } from './components/DropZone';
import { ProgressView } from './components/ProgressView';
import { SummaryCard } from './components/SummaryCard';
import { ApiKeyScreen } from './screens/ApiKeyScreen';
import { CostScreen } from './screens/CostScreen';
import { ClusteringView } from './screens/ClusteringView';

export default function App() {
  const {
    phase,
    stage,
    error,
    summary,
    reset,
    setAwaitingKey,
    setKeyStored,
    setExportWithoutAI,
    clusterError,
    tokenEstimate,
    costEstimateUsd,
    elapsedSecs,
  } = useAppStore();
  const { getApiKey } = useKeychain();
  const { fetchCostEstimate, startClustering } = useCluster();

  // Auto-trigger cost estimation when phase transitions to key-stored
  useEffect(() => {
    if (phase === 'key-stored') {
      fetchCostEstimate();
    }
  }, [phase]); // eslint-disable-line react-hooks/exhaustive-deps

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

      {/* Phase 1 error — parse failure — returns to drop zone */}
      {phase === 'error' && !clusterError && (
        <DropZone errorMessage={error ?? undefined} onReset={reset} />
      )}
      {/* Phase 2 error — clustering failure — "Try again" returns to cost screen */}
      {phase === 'error' && clusterError && (
        <div className="flex flex-col items-center gap-4 text-center px-6">
          <p className="text-sm text-red-500">{clusterError}</p>
          <button
            onClick={() => useAppStore.setState({ phase: 'cost-ready', clusterError: null })}
            className="text-sm text-neutral-500 hover:text-neutral-700 underline underline-offset-2"
          >
            Try again
          </button>
        </div>
      )}

      {phase === 'complete' && summary && (
        <SummaryCard
          total={summary.total}
          earliestYear={summary.earliestYear}
          latestYear={summary.latestYear}
          onContinueWithAI={handleSummaryContinue}
          onExportWithoutAI={setExportWithoutAI}
          hasApiKey={false}
          onChangeKey={() => setAwaitingKey()}
        />
      )}

      {phase === 'awaiting-key' && (
        <ApiKeyScreen initialError={clusterError ?? undefined} />
      )}
      {phase === 'key-stored' && (
        <ProgressView stage="Counting tokens..." />
      )}
      {phase === 'cost-ready' && tokenEstimate !== null && costEstimateUsd !== null && (
        <CostScreen
          tokens={tokenEstimate}
          estimatedUsd={costEstimateUsd}
          onProceed={startClustering}
        />
      )}
      {phase === 'clustering' && (
        <ClusteringView stage={stage} elapsedSecs={elapsedSecs} />
      )}
      {phase === 'preview-ready' && (
        // Phase 3 entry point — reached via either with-ai or without-ai path
        <ProgressView stage="Preparing preview…" />
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
