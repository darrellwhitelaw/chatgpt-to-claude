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

        <div className="flex flex-col items-center gap-4">
          {/* Stats */}
          <div className="flex flex-col items-center gap-1.5 text-center">
            <p className="text-sm text-neutral-600 leading-relaxed whitespace-nowrap">
              Your{' '}
              <span className="font-medium text-neutral-800">{count.toLocaleString()}</span>{' '}
              conversation{count !== 1 ? 's' : ''} are ready for Claude
            </p>
            {mediaExtracted > 0 && (
              <p className="text-xs text-neutral-400">
                + {mediaExtracted.toLocaleString()} images & files
              </p>
            )}
            <p className="text-xs text-neutral-400">{displayPath}</p>
          </div>

          {/* Open Folder button */}
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

          {/* Next steps */}
          <div className="flex flex-col items-center gap-1 text-center">
            {mcpConfigured ? (
              <>
                <p className="text-xs text-neutral-400">Restart Claude Desktop</p>
                <p className="text-xs text-neutral-400">Open a new chat</p>
                <p className="text-xs text-neutral-400">Say "Load my ChatGPT history"</p>
              </>
            ) : (
              <>
                <p className="text-xs text-neutral-400">Open Claude Desktop or claude.ai</p>
                <p className="text-xs text-neutral-400">Create a Project</p>
                <p className="text-xs text-neutral-400">Add files from the folder below</p>
              </>
            )}
          </div>
        </div>
      </div>

      <button
        onClick={onStartOver}
        onMouseEnter={() => setIsStartOverHovered(true)}
        onMouseLeave={() => setIsStartOverHovered(false)}
        className={`transition-colors p-1 ${
          isStartOverHovered ? 'text-neutral-600' : 'text-neutral-300'
        }`}
        title="Start over"
      >
        <svg width="20" height="20" viewBox="0 0 100 100" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
          <path d="m37.133 64.016c-2.582-3.1289-3.9766-7.0703-3.9258-11.129 0.015625-2.2305 0.48047-4.4336 1.3633-6.4805 0.68359-1.5742 1.6211-3.0234 2.7734-4.2891 0.66406-0.74219 0.62891-1.8711-0.078125-2.5664l-1.3047-1.3047v-0.003906c-0.37891-0.37109-0.89453-0.56641-1.4219-0.54297-0.52734 0.023437-1.0234 0.26172-1.3711 0.66406-1.5547 1.7539-2.8203 3.75-3.7383 5.9062-1.2109 2.7969-1.8281 5.8086-1.8203 8.8516-0.082031 5.4805 1.9062 10.789 5.5625 14.871 3.0352 3.5352 7.1406 5.9844 11.691 6.9805 0.56641 0.12891 1.1562-0.003906 1.6133-0.36328 0.45313-0.35938 0.71875-0.90625 0.72266-1.4844v-1.8633c-0.011719-0.85938-0.61328-1.6016-1.4531-1.793-3.3867-0.83594-6.4102-2.75-8.6133-5.4531z"/>
          <path d="m65.883 37.238c-4.168-4.2695-9.918-6.625-15.883-6.5078h-0.21094l1.5586-1.5586v0.003906c0.83984-0.84375 0.83984-2.2031 0-3.043l-0.875-0.875v-0.003906c-0.40234-0.40234-0.95312-0.62891-1.5234-0.62891-0.57031 0-1.1172 0.22656-1.5195 0.62891l-6.7539 6.7539c-0.40234 0.40625-0.62891 0.95313-0.62891 1.5234s0.22656 1.1172 0.62891 1.5195l6.7539 6.7539c0.83984 0.83984 2.2031 0.83984 3.043 0l0.875-0.875c0.83984-0.83984 0.83984-2.2031 0-3.043l-1.5586-1.5586h0.21094c4.3477-0.015625 8.5352 1.6602 11.668 4.6758 3.1328 3.0156 4.9688 7.1328 5.1133 11.48 0.14453 4.1836-1.2617 8.2734-3.9453 11.488-2.2148 2.6797-5.2148 4.5938-8.5781 5.4766-0.84375 0.20312-1.4453 0.95312-1.457 1.8203v1.8359c0.003907 0.58203 0.26953 1.1289 0.73047 1.4883 0.45703 0.35938 1.0508 0.49219 1.6172 0.35547 4.9141-1.1641 9.2891-3.9531 12.418-7.9141 3.1289-3.9609 4.8281-8.8633 4.8242-13.914 0.11719-5.9648-2.2383-11.715-6.5078-15.883z"/>
        </svg>
      </button>
    </div>
  );
}
