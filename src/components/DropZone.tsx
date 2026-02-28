import { useEffect, useRef, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { open } from '@tauri-apps/plugin-dialog';
import openaiZipIcon from '../assets/openai_zip_icon.png';
import { useIngest } from '../hooks/useIngest';

interface DropZoneProps {
  errorMessage?: string;
  onReset?: () => void;
}

export function DropZone({ errorMessage, onReset }: DropZoneProps) {
  const { startIngest } = useIngest();
  const [isDragging, setIsDragging] = useState(false);
  const [isBrowseHovered, setIsBrowseHovered] = useState(false);
  const unlistenRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    let mounted = true;

    getCurrentWindow()
      .onDragDropEvent((event) => {
        if (!mounted) return;

        if (event.payload.type === 'enter') {
          setIsDragging(true);
        } else if (event.payload.type === 'leave') {
          setIsDragging(false);
        } else if (event.payload.type === 'drop') {
          setIsDragging(false);
          // CRITICAL: Use Tauri's payload.paths — NOT HTML5 dataTransfer (gives no paths in Tauri)
          const paths: string[] = event.payload.paths ?? [];
          const zipPath = paths.find((p) => p.toLowerCase().endsWith('.zip'));
          if (zipPath) {
            startIngest(zipPath);
          }
          // Non-ZIP dropped: drop zone stays interactive, no action needed
        }
      })
      .then((unlisten) => {
        unlistenRef.current = unlisten;
      });

    return () => {
      mounted = false;
      unlistenRef.current?.();
    };
  }, [startIngest]);

  const handleBrowse = async () => {
    const path = await open({
      multiple: false,
      directory: false,
      filters: [{ name: 'ChatGPT Export', extensions: ['zip'] }],
    });
    if (path) {
      startIngest(path as string);
    }
  };

  return (
    <div className="flex flex-col items-center gap-4 w-full max-w-sm px-6">
      {/* Drop zone */}
      <div
        className={[
          'w-full rounded-xl border-2 border-dashed transition-colors duration-150',
          'flex flex-col items-center justify-center py-10 px-6',
          isDragging
            ? 'border-neutral-400 bg-neutral-50'
            : 'border-neutral-300 bg-white',
        ].join(' ')}
      >
        {/* ChatGPT ZIP icon — stands alone above the text+action group */}
        <img
          src={openaiZipIcon}
          alt="ChatGPT export"
          className={`w-16 h-16 object-contain transition-opacity duration-150 mb-5 ${isDragging ? 'opacity-70' : 'opacity-100'}`}
        />

        {/* Text + action grouped tightly together */}
        <div className="flex flex-col items-center gap-3">
          <p className="text-sm text-neutral-500 text-center leading-snug">
            Drop your ChatGPT export here
          </p>

          <button
            onClick={handleBrowse}
            onMouseEnter={() => setIsBrowseHovered(true)}
            onMouseLeave={() => setIsBrowseHovered(false)}
            className={`text-xs transition-colors px-4 py-1.5 rounded-full ${
              isBrowseHovered
                ? 'bg-neutral-900 text-white'
                : 'bg-neutral-100 text-neutral-500'
            }`}
          >
            or select file
          </button>
        </div>
      </div>

      {/* Inline error — shown adjacent to zone; zone stays interactive (locked decision) */}
      {errorMessage && (
        <p className="text-sm text-red-500 text-center leading-snug">
          {errorMessage}
          {onReset && (
            <button
              onClick={onReset}
              className="ml-2 underline text-red-400 hover:text-red-600"
            >
              Try again
            </button>
          )}
        </p>
      )}
    </div>
  );
}
