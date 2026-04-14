export type ProfileRow = {
  id: string;
  name: string;
  defaultTopK: number;
  defaultChunkSize: number;
  defaultChunkOverlap: number;
  outputMode: "text" | "json";
  embeddingModel: string;
  metadata: string;
};
