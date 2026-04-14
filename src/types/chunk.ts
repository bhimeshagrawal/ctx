export type ChunkRow = {
  id: string;
  documentId: string;
  chunkIndex: number;
  content: string;
  contentHash: string;
  tokenEstimate: number;
  vector: number[] | Float32Array;
  vectorJson: string;
  title: string | null;
  sourcePath: string | null;
  tags: string[];
  createdAt: string;
  metadata: string;
};

export type SearchCandidate = ChunkRow & {
  vectorScore: number;
  keywordScore: number;
  finalScore: number;
};
