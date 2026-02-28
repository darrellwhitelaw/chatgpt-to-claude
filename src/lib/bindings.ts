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

// ClusterEvent — must stay in sync with src-tauri/src/commands/cluster.rs ClusterEvent
// These events are emitted via Channel<ClusterEvent> during the clustering pipeline
export type ClusterEvent =
  | { event: 'estimatingTokens' }
  | { event: 'tokensCounted'; data: { tokens: number; estimatedUsd: number } }
  | { event: 'pass1Started' }
  | { event: 'pass1Complete'; data: { clusterLabels: string[] } }
  | { event: 'batchSubmitted'; data: { batchId: string } }
  | { event: 'polling'; data: { elapsedSecs: number } }
  | { event: 'complete'; data: { assignedCount: number } }
  | { event: 'error'; data: { message: string } };

export type StartClusteringArgs = {
  // No args needed — cluster command reads from SQLite and Keychain internally
};
