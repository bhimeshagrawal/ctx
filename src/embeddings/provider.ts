export interface EmbeddingProvider {
  readonly name: string;
  readonly model: string;
  init(): Promise<void>;
  getDimension(): Promise<number>;
  embed(texts: string[]): Promise<number[][]>;
  embedQuery(query: string): Promise<number[]>;
  healthCheck(): Promise<{ ok: boolean; detail: string }>;
}
