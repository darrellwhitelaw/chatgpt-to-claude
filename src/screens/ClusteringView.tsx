interface ClusteringViewProps {
  stage: string;
  elapsedSecs?: number;
}

export function ClusteringView({ stage, elapsedSecs }: ClusteringViewProps) {
  const formatElapsed = (secs: number): string => {
    if (secs < 60) return `${secs}s`;
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return `${m}m ${s}s`;
  };

  return (
    <div className="flex flex-col items-center gap-4">
      {/* Spinner — identical to ProgressView */}
      <div className="w-8 h-8 rounded-full border-2 border-neutral-200 border-t-neutral-500 animate-spin" />
      {/* Stage label — plain English, no jargon (locked decision) */}
      <p className="text-sm text-neutral-500">{stage}</p>
      {/* Elapsed time — shown after batch submitted */}
      {elapsedSecs !== undefined && elapsedSecs > 0 && (
        <p className="text-xs text-neutral-300">{formatElapsed(elapsedSecs)}</p>
      )}
    </div>
  );
}
