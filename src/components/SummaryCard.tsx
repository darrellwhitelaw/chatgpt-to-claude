import { useState } from 'react';
import openaiFolderIcon from '../assets/openai_folder_icon.png';

interface SummaryCardProps {
  total: number;
  earliestYear: number;
  latestYear: number;
  onExport: () => void;
}

export function SummaryCard({
  total,
  earliestYear,
  latestYear,
  onExport,
}: SummaryCardProps) {
  const [isExportHovered, setIsExportHovered] = useState(false);

  const yearRange =
    earliestYear === latestYear
      ? String(earliestYear)
      : `${earliestYear} \u2013 ${latestYear}`;

  return (
    <div className="flex flex-col items-center gap-4 w-full max-w-sm px-6">
      <div className="w-full rounded-xl border border-neutral-200 bg-white flex flex-col items-center py-10 px-6">
        <img
          src={openaiFolderIcon}
          alt="ChatGPT export"
          className="w-16 h-16 object-contain mb-5"
        />

        <div className="flex flex-col items-center gap-3">
          <div className="flex flex-col items-center gap-2 text-center">
            <p className="text-sm text-neutral-600 leading-relaxed">
              You had{' '}
              <span className="font-medium text-neutral-800">{total.toLocaleString()}</span>{' '}
              conversation{total !== 1 ? 's' : ''} in your ChatGPT history
              {earliestYear ? (
                <>
                  {' '}from{' '}
                  <span className="font-medium text-neutral-800">{yearRange}</span>
                </>
              ) : null}
            </p>
          </div>

          <button
            onClick={onExport}
            onMouseEnter={() => setIsExportHovered(true)}
            onMouseLeave={() => setIsExportHovered(false)}
            className={`text-xs transition-colors px-4 py-1.5 rounded-full ${
              isExportHovered
                ? 'bg-neutral-900 text-white'
                : 'bg-neutral-100 text-neutral-500'
            }`}
          >
            Export to Claude
          </button>
        </div>
      </div>
    </div>
  );
}
