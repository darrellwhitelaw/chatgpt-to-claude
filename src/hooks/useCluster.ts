import { invoke, Channel } from '@tauri-apps/api/core';
import { useAppStore } from '../store/appStore';
import type { ClusterEvent } from '../lib/bindings';

interface CostEstimate {
  input_tokens: number;
  estimated_usd: number;
}

export function useCluster() {
  const {
    setCostReady,
    setClusteringComplete,
  } = useAppStore();

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
        useAppStore.getState().setError(msg);
      }
    }
  };

  const startClustering = async (): Promise<void> => {
    // Transition to clustering phase immediately so ClusteringView renders
    useAppStore.setState({ phase: 'clustering', stage: 'Discovering clusters...' });

    const onEvent = new Channel<ClusterEvent>();

    onEvent.onmessage = (msg) => {
      switch (msg.event) {
        case 'pass1Started':
          // Phase is already 'clustering' — only update stage label
          useAppStore.setState({ stage: 'Discovering clusters...' });
          break;
        case 'pass1Complete':
          useAppStore.setState({ stage: 'Clustering conversations...' });
          break;
        case 'batchSubmitted':
          // Update batchId and stage in one atomic setState
          useAppStore.setState({ batchId: msg.data.batchId, stage: 'Clustering conversations...' });
          break;
        case 'polling':
          useAppStore.setState({ stage: 'Clustering conversations...', elapsedSecs: msg.data.elapsedSecs });
          break;
        case 'complete':
          useAppStore.setState({ stage: 'Saving results...' });
          setTimeout(() => setClusteringComplete(), 500); // brief feedback
          break;
        case 'error':
          useAppStore.setState({
            phase: 'error',
            clusterError: msg.data.message,
          });
          break;
      }
    };

    try {
      await invoke('start_clustering', { onEvent });
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      useAppStore.setState({ phase: 'error', clusterError: msg });
    }
  };

  return { fetchCostEstimate, startClustering };
}
