interface ProgressViewProps {
  stage: string;
}

export function ProgressView({ stage }: ProgressViewProps) {
  return (
    <div className="flex flex-col items-center gap-4">
      {/* Spinner — CSS animation */}
      <div className="w-8 h-8 rounded-full border-2 border-neutral-200 border-t-neutral-500 animate-spin" />
      {/* Stage label — human-readable, no jargon, no numbers (locked decision) */}
      <p className="text-sm text-neutral-500">{stage}</p>
    </div>
  );
}
