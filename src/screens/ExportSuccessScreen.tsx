import { useState } from 'react';
import { revealItemInDir } from '@tauri-apps/plugin-opener';
import openaiToClaudeFolderIcon from '../assets/openai_to_claude_folder_icon.png';

interface ExportSuccessScreenProps {
  count: number;
  folderPath: string;
  mcpConfigured: boolean;
  mediaExtracted: number;
  onStartOver: () => void;
}

export function ExportSuccessScreen({ count, folderPath, mcpConfigured, mediaExtracted, onStartOver }: ExportSuccessScreenProps) {
  const [isOpenHovered, setIsOpenHovered] = useState(false);
  const [isStartOverHovered, setIsStartOverHovered] = useState(false);

  const displayPath = folderPath
    .replace(/^\/Users\/[^/]+/, '~')
    .replace(/^\/private\/Users\/[^/]+/, '~');

  const handleOpenFolder = async () => {
    try {
      await revealItemInDir(folderPath);
    } catch {
      // ignore
    }
  };

  return (
    <div className="flex flex-col items-center gap-4 w-full max-w-sm px-6">
      <div className="w-full rounded-xl border border-neutral-200 bg-white flex flex-col items-center py-10 px-6">
        <img
          src={openaiToClaudeFolderIcon}
          alt="Export complete"
          className="w-16 h-16 object-contain mb-5"
        />

        <div className="flex flex-col items-center gap-3">
          {/* Stats */}
          <div className="flex flex-col items-center gap-1">
            <p className="text-5xl font-thin text-neutral-800 tabular-nums">
              {count.toLocaleString()}
            </p>
            <p className="text-sm text-neutral-500 leading-snug">
              conversation{count !== 1 ? 's' : ''} exported
            </p>
            {mediaExtracted > 0 && (
              <p className="text-xs text-neutral-400">+ {mediaExtracted.toLocaleString()} images & files</p>
            )}
            <p className="text-xs text-neutral-400">{displayPath}</p>
          </div>

          {/* Next steps */}
          <div className="flex flex-col items-center gap-1 text-center">
            {mcpConfigured ? (
              <>
                <p className="text-xs text-neutral-400">1. Restart Claude Desktop</p>
                <p className="text-xs text-neutral-400">2. Open a new chat</p>
                <p className="text-xs text-neutral-400">3. Say "Load my ChatGPT history"</p>
              </>
            ) : (
              <>
                <p className="text-xs text-neutral-400">1. Open Claude Desktop or claude.ai</p>
                <p className="text-xs text-neutral-400">2. Create a Project</p>
                <p className="text-xs text-neutral-400">3. Add files from the folder below</p>
              </>
            )}
          </div>

          <button
            onClick={handleOpenFolder}
            onMouseEnter={() => setIsOpenHovered(true)}
            onMouseLeave={() => setIsOpenHovered(false)}
            className={`text-xs transition-colors px-4 py-1.5 rounded-full ${
              isOpenHovered
                ? 'bg-neutral-900 text-white'
                : 'bg-neutral-100 text-neutral-500'
            }`}
          >
            Open Folder
          </button>
        </div>
      </div>

      <button
        onClick={onStartOver}
        onMouseEnter={() => setIsStartOverHovered(true)}
        onMouseLeave={() => setIsStartOverHovered(false)}
        className={`text-xs transition-colors ${
          isStartOverHovered ? 'text-neutral-600' : 'text-neutral-400'
        }`}
      >
        Start over
      </button>
    </div>
  );
}
