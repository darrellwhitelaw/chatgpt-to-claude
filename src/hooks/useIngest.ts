import { invoke, Channel } from '@tauri-apps/api/core';
import type { IngestEvent } from '../lib/bindings';
import { useAppStore } from '../store/appStore';

export function useIngest() {
  const { setStage, setError, setComplete } = useAppStore();

  const startIngest = async (zipPath: string) => {
    const onEvent = new Channel<IngestEvent>();

    onEvent.onmessage = (msg) => {
      switch (msg.event) {
        case 'started':
          setStage('Starting...');
          break;
        case 'extractingZip':
          setStage('Extracting ZIP...');
          break;
        case 'parsingConversations':
          setStage('Parsing conversations...');
          break;
        case 'buildingIndex':
          setStage('Building index...');
          break;
        case 'complete':
          setComplete({
            total: msg.data.total,
            earliestYear: msg.data.earliestYear,
            latestYear: msg.data.latestYear,
          });
          break;
        case 'error':
          setError(msg.data.message);
          break;
      }
    };

    try {
      await invoke('parse_zip', { path: zipPath, onEvent });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  return { startIngest };
}
