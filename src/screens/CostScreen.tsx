import { useAppStore } from '../store/appStore';

interface CostScreenProps {
  tokens: number;
  estimatedUsd: number;
}

export function CostScreen({ tokens, estimatedUsd }: CostScreenProps) {
  const { setClustering } = useAppStore();

  const formatTokens = (n: number): string => {
    if (n >= 1_000_000) return `~${(n / 1_000_000).toFixed(1)}M`;
    if (n >= 1_000) return `~${Math.round(n / 1_000)}K`;
    return `${n}`;
  };

  const handleProceed = () => {
    // Plan 02-05 will invoke the actual batch command here.
    // For now, transition phase to trigger ClusteringView placeholder.
    setClustering('pending'); // placeholder batchId — replaced in 02-05
  };

  const handleCancel = () => {
    // Return to summary card — phase is 'complete' with summary still in store
    useAppStore.setState({ phase: 'complete' });
  };

  const HIGH_COST_THRESHOLD = 3.00;

  return (
    <div className="flex flex-col items-center gap-6 w-full max-w-sm px-6 text-center">
      {/* Token + cost estimate — single line with middot separator (locked decision) */}
      <p className="text-2xl font-medium text-neutral-800">
        {formatTokens(tokens)} tokens · estimated ${estimatedUsd.toFixed(2)}
      </p>

      {/* High-cost warning (> $3.00) */}
      {estimatedUsd > HIGH_COST_THRESHOLD && (
        <div className="w-full rounded-lg bg-amber-50 border border-amber-200 px-4 py-3">
          <p className="text-sm text-amber-700">
            This is higher than typical — your export is large.
          </p>
        </div>
      )}

      {/* Actions */}
      <div className="flex flex-col items-center gap-3 w-full">
        <button
          onClick={handleProceed}
          className="w-full px-8 py-2.5 rounded-lg bg-neutral-900 text-white text-sm font-medium
                     hover:bg-neutral-700 transition-colors"
        >
          Proceed
        </button>
        <button
          onClick={handleCancel}
          className="text-sm text-neutral-400 hover:text-neutral-600 transition-colors underline underline-offset-2"
        >
          Cancel
        </button>
      </div>
    </div>
  );
}
