import { create } from 'zustand';

export type AppPhase = 'idle' | 'parsing' | 'complete' | 'error';

export interface Summary {
  total: number;
  earliestYear: number;
  latestYear: number;
}

interface AppState {
  phase: AppPhase;
  stage: string;          // Human-readable current stage label
  error: string | null;
  summary: Summary | null;

  setStage: (stage: string) => void;
  setError: (msg: string) => void;
  setComplete: (summary: Summary) => void;
  reset: () => void;
}

export const useAppStore = create<AppState>((set) => ({
  phase: 'idle',
  stage: '',
  error: null,
  summary: null,

  setStage: (stage) => set({ phase: 'parsing', stage, error: null }),
  setError: (msg) => set({ phase: 'error', error: msg }),
  setComplete: (summary) => set({ phase: 'complete', summary }),
  reset: () => set({ phase: 'idle', stage: '', error: null, summary: null }),
}));
