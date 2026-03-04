import type { ClusterPreviewItem } from '../store/appStore';

interface PreviewScreenProps {
  clusters: ClusterPreviewItem[];
  totalClustered: number;
  onConfirm: () => void;
  onCancel: () => void;
}

export function PreviewScreen({ clusters, totalClustered, onConfirm, onCancel }: PreviewScreenProps) {
  return (
    <div className="flex flex-col items-center gap-6 w-full max-w-md px-6">
      <p className="text-sm text-neutral-600 text-center">
        <span className="font-medium text-neutral-800">{totalClustered}</span>{' '}
        conversation{totalClustered !== 1 ? 's' : ''} organized into{' '}
        <span className="font-medium text-neutral-800">{clusters.length}</span>{' '}
        project{clusters.length !== 1 ? 's' : ''}
      </p>

      <div className="w-full max-h-64 overflow-y-auto rounded-xl border border-neutral-200 bg-white divide-y divide-neutral-100">
        {clusters.map((cluster) => (
          <div key={cluster.label} className="px-4 py-3">
            <div className="flex items-baseline justify-between gap-2">
              <p className="text-sm font-medium text-neutral-800 truncate">{cluster.label}</p>
              <span className="text-xs text-neutral-400 shrink-0">{cluster.count}</span>
            </div>
            {cluster.earliest && cluster.latest && (
              <p className="text-xs text-neutral-400 mt-0.5">
                {cluster.earliest} &ndash; {cluster.latest}
              </p>
            )}
            {cluster.titles.length > 0 && (
              <ul className="mt-1.5 space-y-0.5">
                {cluster.titles.slice(0, 3).map((title) => (
                  <li key={title} className="text-xs text-neutral-500 truncate">
                    {title}
                  </li>
                ))}
              </ul>
            )}
          </div>
        ))}
      </div>

      <div className="flex flex-col items-center gap-3 w-full">
        <button
          onClick={onConfirm}
          className="px-8 py-2.5 rounded-lg bg-neutral-900 text-white text-sm font-medium
                     hover:bg-neutral-700 transition-colors"
        >
          Confirm and Export
        </button>
        <button
          onClick={onCancel}
          className="text-sm text-neutral-400 hover:text-neutral-600 transition-colors underline underline-offset-2"
        >
          Cancel
        </button>
      </div>
    </div>
  );
}
