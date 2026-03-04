import { useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useAppStore } from './store/appStore';
import { useCluster } from './hooks/useCluster';
import { DropZone } from './components/DropZone';
import { ProgressView } from './components/ProgressView';
import { SummaryCard } from './components/SummaryCard';
import { ExportSuccessScreen } from './screens/ExportSuccessScreen';
import { ApiKeyScreen } from './screens/ApiKeyScreen';
import { CostScreen } from './screens/CostScreen';
import { ClusteringView } from './screens/ClusteringView';
import { PreviewScreen } from './screens/PreviewScreen';
import type { ClusterPreview } from './lib/bindings';

interface ExportResult {
  files_written: number;
  folder_path: string;
  mcp_configured: boolean;
  media_extracted: number;
  memory_path: string | null;
}

function BackButton({ onClick }: { onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className="absolute top-4 left-4 text-neutral-300 hover:text-neutral-600 transition-colors p-1"
      aria-label="Back"
    >
      <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
        <path d="M12.5 15L7.5 10L12.5 5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
      </svg>
    </button>
  );
}

export default function App() {
  const {
    phase,
    stage,
    error,
    summary,
    reset,
    setExporting,
    setExportSuccess,
    setAwaitingKey,
    setPreview,
    exportPath,
    exportCount,
    mcpConfigured,
    mediaExtracted,
    memoryPath,
    tokenEstimate,
    costEstimateUsd,
    clusterError,
    clusterPreview,
    elapsedSecs,
  } = useAppStore();

  const { fetchCostEstimate, startClustering } = useCluster();

  const handleExport = async () => {
    setExporting();
    try {
      const result = await invoke<ExportResult>('export_conversations');
      setExportSuccess(result.folder_path, result.files_written, result.mcp_configured, result.media_extracted, result.memory_path);
    } catch (err) {
      useAppStore.setState({ phase: 'error', error: String(err) });
    }
  };

  const handleOrganizeWithAI = () => {
    setAwaitingKey();
  };

  const handleBackToSummary = () => {
    useAppStore.setState({ phase: 'complete' });
  };

  // Auto-fetch cost estimate once API key is stored
  useEffect(() => {
    if (phase === 'key-stored') {
      fetchCostEstimate();
    }
  }, [phase]);

  // Fetch cluster preview data when entering preview phase
  useEffect(() => {
    if (phase === 'preview' && !clusterPreview) {
      invoke<ClusterPreview>('get_cluster_preview')
        .then((result) => setPreview(result.clusters))
        .catch((err) => useAppStore.setState({ phase: 'error', error: String(err) }));
    }
  }, [phase, clusterPreview]);

  const handlePreviewConfirm = async () => {
    setExporting();
    try {
      const result = await invoke<ExportResult>('export_conversations');
      setExportSuccess(result.folder_path, result.files_written, result.mcp_configured, result.media_extracted, result.memory_path);
    } catch (err) {
      useAppStore.setState({ phase: 'error', error: String(err) });
    }
  };

  return (
    <div className="h-screen w-screen bg-white flex items-center justify-center pb-8">
      {phase === 'idle' && <DropZone />}
      {phase === 'parsing' && <ProgressView stage={stage} />}
      {phase === 'exporting' && <ProgressView stage="Exporting conversations…" />}

      {phase === 'error' && (
        <DropZone errorMessage={error ?? clusterError ?? undefined} onReset={reset} />
      )}

      {phase === 'complete' && summary && (
        <>
          <BackButton onClick={reset} />
          <SummaryCard
            total={summary.total}
            earliestYear={summary.earliestYear}
            latestYear={summary.latestYear}
            onExport={handleExport}
            onOrganizeWithAI={handleOrganizeWithAI}
          />
        </>
      )}

      {phase === 'awaiting-key' && (
        <>
          <BackButton onClick={handleBackToSummary} />
          <ApiKeyScreen initialError={clusterError ?? undefined} />
        </>
      )}

      {phase === 'key-stored' && (
        <>
          <BackButton onClick={handleBackToSummary} />
          <ProgressView stage="Checking API key..." />
        </>
      )}

      {phase === 'cost-ready' && tokenEstimate !== null && costEstimateUsd !== null && (
        <>
          <BackButton onClick={handleBackToSummary} />
          <CostScreen
            tokens={tokenEstimate}
            estimatedUsd={costEstimateUsd}
            onProceed={startClustering}
          />
        </>
      )}

      {phase === 'clustering' && (
        <ClusteringView stage={stage} elapsedSecs={elapsedSecs} />
      )}

      {phase === 'preview' && clusterPreview && (
        <>
          <BackButton onClick={handleBackToSummary} />
          <PreviewScreen
            clusters={clusterPreview}
            totalClustered={clusterPreview.reduce((sum, c) => sum + c.count, 0)}
            onConfirm={handlePreviewConfirm}
            onCancel={handleBackToSummary}
          />
        </>
      )}

      {phase === 'preview' && !clusterPreview && (
        <ProgressView stage="Loading preview..." />
      )}

      {phase === 'export-success' && exportPath && exportCount !== null && (
        <ExportSuccessScreen
          count={exportCount}
          folderPath={exportPath}
          mcpConfigured={mcpConfigured ?? false}
          mediaExtracted={mediaExtracted ?? 0}
          memoryPath={memoryPath}
          onStartOver={reset}
        />
      )}

      <button
        onClick={() => openUrl('https://darrellwhitelaw.com')}
        className="absolute bottom-4 text-xs text-neutral-300 hover:text-neutral-500 transition-colors underline underline-offset-2"
      >
        made for me
      </button>
    </div>
  );
}
