export type ChunkRow = {
  id: string;
  documentId: string;
  chunkIndex: number;
  content: string;
  contentHash: string;
  tokenEstimate: number;
  embedding: number[];
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
