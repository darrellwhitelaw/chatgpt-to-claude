// IPC types — must stay in sync with src-tauri/src/commands/ingest.rs IngestEvent

export type IngestEvent =
  | { event: 'started'; data: never }
  | { event: 'extractingZip'; data: never }
  | { event: 'parsingConversations'; data: { processed: number } }
  | { event: 'buildingIndex'; data: never }
  | { event: 'complete'; data: { total: number; earliestYear: number; latestYear: number } }
  | { event: 'error'; data: { message: string } };

export type ParseZipArgs = {
  path: string;
};
