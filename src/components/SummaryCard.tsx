interface SummaryCardProps {
  total: number;
  earliestYear: number;
  latestYear: number;
  onContinueWithAI: () => void;
  onExportWithoutAI: () => void;
  hasApiKey?: boolean;
  onChangeKey?: () => void;
}

export function SummaryCard({
  total,
  earliestYear,
  latestYear,
  onContinueWithAI,
  onExportWithoutAI,
  hasApiKey,
  onChangeKey,
}: SummaryCardProps) {
  const yearRange =
    earliestYear === latestYear
      ? String(earliestYear)
      : `${earliestYear} \u2013 ${latestYear}`;

  return (
    <div className="flex flex-col items-center gap-6 text-center px-6">
      {/* Summary */}
      <div>
        <p className="text-2xl font-medium text-neutral-800">
          Found {total.toLocaleString()} conversation{total !== 1 ? 's' : ''}
        </p>
        <p className="text-sm text-neutral-400 mt-1">{yearRange}</p>
      </div>

      {/* Primary action — with AI clustering + summaries */}
      <div className="flex flex-col items-center gap-3">
        <button
          onClick={onContinueWithAI}
          className="px-8 py-2.5 rounded-lg bg-neutral-900 text-white text-sm font-medium hover:bg-neutral-700 transition-colors"
        >
          Continue with AI
        </button>

        {/* Secondary action — skip AI, export by year */}
        <button
          onClick={onExportWithoutAI}
          className="text-xs text-neutral-400 hover:text-neutral-600 transition-colors underline underline-offset-2"
        >
          Export without AI
        </button>
      </div>

      {/* Subtle "Change key" link — only visible when a key is already stored */}
      {hasApiKey && onChangeKey && (
        <button
          onClick={onChangeKey}
          className="text-xs text-neutral-400 hover:text-neutral-600 transition-colors underline underline-offset-2"
        >
          Change key
        </button>
      )}
    </div>
  );
}
