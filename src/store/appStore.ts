import { create } from 'zustand';

export type AppPhase =
  | 'idle'
  | 'parsing'
  | 'complete'             // Summary card shown
  | 'exporting'            // Writing markdown files to disk
  | 'export-success'       // Export complete — show success screen
  | 'awaiting-key'         // AI path: no API key — show ApiKeyScreen
  | 'key-stored'           // AI path: key confirmed — ready to fetch cost
  | 'cost-ready'           // AI path: show CostScreen
  | 'clustering'           // AI path: batch polling active
  | 'error';

export type ExportMode = 'with-ai' | 'without-ai' | null;

export interface Summary {
  total: number;
  earliestYear: number;
  latestYear: number;
}

interface AppState {
  phase: AppPhase;
  stage: string;
  error: string | null;
  summary: Summary | null;
  exportMode: ExportMode;
  exportPath: string | null;        // path written to on export-success
  exportCount: number | null;       // files written on export-success
  mcpConfigured: boolean | null;    // whether Claude Desktop MCP was auto-configured
  mediaExtracted: number | null;    // images + group chats extracted from ZIP

  // AI path fields
  tokenEstimate: number | null;
  costEstimateUsd: number | null;
  batchId: string | null;
  clusterError: string | null;
  elapsedSecs: number;

  // Phase 1 actions
  setStage: (stage: string) => void;
  setError: (msg: string) => void;
  setComplete: (summary: Summary) => void;
  reset: () => void;

  // Export path actions
  setExporting: () => void;
  setExportSuccess: (path: string, count: number, mcpConfigured: boolean, mediaExtracted: number) => void;

  // AI path actions
  setAwaitingKey: () => void;
  setKeyStored: () => void;
  setCostReady: (tokenEstimate: number, costEstimateUsd: number) => void;
  setClustering: (batchId: string) => void;
  setClusteringComplete: () => void;
  setClusterError: (msg: string) => void;
}

export const useAppStore = create<AppState>((set) => ({
  phase: 'idle',
  stage: '',
  error: null,
  summary: null,
  exportMode: null,
  exportPath: null,
  exportCount: null,
  mcpConfigured: null,
  mediaExtracted: null,

  tokenEstimate: null,
  costEstimateUsd: null,
  batchId: null,
  clusterError: null,
  elapsedSecs: 0,

  // Phase 1
  setStage: (stage) => set({ phase: 'parsing', stage, error: null }),
  setError: (msg) => set({ phase: 'error', error: msg }),
  setComplete: (summary) => set({ phase: 'complete', summary }),
  reset: () => set({
    phase: 'idle', stage: '', error: null, summary: null,
    exportMode: null, exportPath: null, exportCount: null, mcpConfigured: null, mediaExtracted: null,
    tokenEstimate: null, costEstimateUsd: null, batchId: null,
    clusterError: null, elapsedSecs: 0,
  }),

  // Export path
  setExporting: () => set({ phase: 'exporting', exportMode: 'without-ai' }),
  setExportSuccess: (path, count, mcpConfigured, mediaExtracted) =>
    set({ phase: 'export-success', exportPath: path, exportCount: count, mcpConfigured, mediaExtracted }),

  // AI path
  setAwaitingKey: () => set({ phase: 'awaiting-key', exportMode: 'with-ai' }),
  setKeyStored: () => set({ phase: 'key-stored' }),
  setCostReady: (tokenEstimate, costEstimateUsd) =>
    set({ phase: 'cost-ready', tokenEstimate, costEstimateUsd }),
  setClustering: (batchId) => set({ phase: 'clustering', batchId }),
  setClusteringComplete: () => set({ phase: 'export-success', exportMode: 'with-ai' }),
  setClusterError: (msg) => set({ phase: 'error', clusterError: msg }),
}));
