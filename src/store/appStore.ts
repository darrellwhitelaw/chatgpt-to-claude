import { create } from 'zustand';

export type AppPhase =
  | 'idle'
  | 'parsing'
  | 'complete'             // Phase 1: summary card shown
  | 'awaiting-key'         // Phase 2: no API key in Keychain — show ApiKeyScreen
  | 'key-stored'           // Phase 2: key confirmed valid — ready to fetch cost
  | 'cost-ready'           // Phase 2: token count returned — show CostScreen with Proceed/Cancel
  | 'clustering'           // Phase 2: batch submitted, polling active — show ClusteringView
  | 'clustering-complete'  // Phase 2: batch done, SQLite written — ready for Phase 3
  | 'error';

export interface Summary {
  total: number;
  earliestYear: number;
  latestYear: number;
}

interface AppState {
  phase: AppPhase;
  stage: string;               // Human-readable current stage label
  error: string | null;
  summary: Summary | null;

  // Phase 2 fields
  tokenEstimate: number | null;      // exact count from /v1/messages/count_tokens
  costEstimateUsd: number | null;    // computed cost in USD
  batchId: string | null;            // Anthropic batch ID from submission
  clusterError: string | null;       // error message for clustering failure screen

  // Phase 1 actions
  setStage: (stage: string) => void;
  setError: (msg: string) => void;
  setComplete: (summary: Summary) => void;
  reset: () => void;

  // Phase 2 actions
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

  // Phase 2 initial state
  tokenEstimate: null,
  costEstimateUsd: null,
  batchId: null,
  clusterError: null,

  // Phase 1 actions
  setStage: (stage) => set({ phase: 'parsing', stage, error: null }),
  setError: (msg) => set({ phase: 'error', error: msg }),
  setComplete: (summary) => set({ phase: 'complete', summary }),
  reset: () => set({
    phase: 'idle', stage: '', error: null, summary: null,
    tokenEstimate: null, costEstimateUsd: null, batchId: null, clusterError: null,
  }),

  // Phase 2 actions
  setAwaitingKey: () => set({ phase: 'awaiting-key' }),
  setKeyStored: () => set({ phase: 'key-stored' }),
  setCostReady: (tokenEstimate, costEstimateUsd) =>
    set({ phase: 'cost-ready', tokenEstimate, costEstimateUsd }),
  setClustering: (batchId) => set({ phase: 'clustering', batchId }),
  setClusteringComplete: () => set({ phase: 'clustering-complete' }),
  setClusterError: (msg) => set({ phase: 'error', clusterError: msg }),
}));
