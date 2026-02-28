import { invoke } from '@tauri-apps/api/core';
import { useAppStore } from '../store/appStore';

interface CostEstimate {
  input_tokens: number;
  estimated_usd: number;
}

export function useCluster() {
  const { setCostReady } = useAppStore();

  const fetchCostEstimate = async (): Promise<void> => {
    try {
      const result: CostEstimate = await invoke('estimate_cost');
      setCostReady(result.input_tokens, result.estimated_usd);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      if (msg.startsWith('INVALID_API_KEY:')) {
        // Bad key — Keychain already cleared by Rust; show ApiKeyScreen with error
        // Set clusterError and transition to awaiting-key in a single atomic update
        useAppStore.setState({
          clusterError: 'Invalid API key — check console.anthropic.com',
          phase: 'awaiting-key',
        });
      } else {
        useAppStore.getState().setClusterError(msg);
      }
    }
  };

  return { fetchCostEstimate };
}
