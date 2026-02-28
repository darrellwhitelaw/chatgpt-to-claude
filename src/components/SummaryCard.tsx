interface SummaryCardProps {
  total: number;
  earliestYear: number;
  latestYear: number;
  onContinue: () => void;
}

export function SummaryCard({
  total,
  earliestYear,
  latestYear,
  onContinue,
}: SummaryCardProps) {
  const yearRange =
    earliestYear === latestYear
      ? String(earliestYear)
      : `${earliestYear} \u2013 ${latestYear}`;

  return (
    <div className="flex flex-col items-center gap-6 text-center px-6">
      {/* Summary — exact format: "Found [N] conversations ([year range])" */}
      <div>
        <p className="text-2xl font-medium text-neutral-800">
          Found {total.toLocaleString()} conversation{total !== 1 ? 's' : ''}
        </p>
        <p className="text-sm text-neutral-400 mt-1">{yearRange}</p>
      </div>

      {/* Single prominent Continue button — locked decision */}
      <button
        onClick={onContinue}
        className="px-8 py-2.5 rounded-lg bg-neutral-900 text-white text-sm font-medium hover:bg-neutral-700 transition-colors"
      >
        Continue
      </button>
    </div>
  );
}
