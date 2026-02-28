import { useAppStore } from './store/appStore';
import { DropZone } from './components/DropZone';
import { ProgressView } from './components/ProgressView';
import { SummaryCard } from './components/SummaryCard';

export default function App() {
  const { phase, stage, error, summary, reset } = useAppStore();

  return (
    <div className="h-screen w-screen bg-white flex items-center justify-center">
      {phase === 'idle' && <DropZone />}
      {phase === 'parsing' && <ProgressView stage={stage} />}
      {phase === 'error' && <DropZone errorMessage={error ?? undefined} onReset={reset} />}
      {phase === 'complete' && summary && (
        <SummaryCard
          total={summary.total}
          earliestYear={summary.earliestYear}
          latestYear={summary.latestYear}
          onContinue={() => {
            // Phase 2 navigation wired here in the next phase
            console.log('Continue clicked — Phase 2 entry point');
          }}
        />
      )}
    </div>
  );
}
