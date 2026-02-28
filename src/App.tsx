import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useAppStore } from './store/appStore';
import { DropZone } from './components/DropZone';
import { ProgressView } from './components/ProgressView';
import { SummaryCard } from './components/SummaryCard';
import { ExportSuccessScreen } from './screens/ExportSuccessScreen';

interface ExportResult {
  files_written: number;
  folder_path: string;
  mcp_configured: boolean;
  media_extracted: number;
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
    exportPath,
    exportCount,
    mcpConfigured,
    mediaExtracted,
  } = useAppStore();

  const handleExport = async () => {
    setExporting();
    try {
      const result = await invoke<ExportResult>('export_conversations');
      setExportSuccess(result.folder_path, result.files_written, result.mcp_configured, result.media_extracted);
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
        <DropZone errorMessage={error ?? undefined} onReset={reset} />
      )}

      {phase === 'complete' && summary && (
        <>
          <button
            onClick={reset}
            className="absolute top-4 left-4 text-neutral-300 hover:text-neutral-600 transition-colors p-1"
            aria-label="Back"
          >
            <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
              <path d="M12.5 15L7.5 10L12.5 5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </button>
          <SummaryCard
            total={summary.total}
            earliestYear={summary.earliestYear}
            latestYear={summary.latestYear}
            onExport={handleExport}
          />
        </>
      )}

      {phase === 'export-success' && exportPath && exportCount !== null && (
        <ExportSuccessScreen
          count={exportCount}
          folderPath={exportPath}
          mcpConfigured={mcpConfigured ?? false}
          mediaExtracted={mediaExtracted ?? 0}
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
