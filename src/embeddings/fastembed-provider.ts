import { EmbeddingModel, ExecutionProvider, FlagEmbedding } from "fastembed";
import path from "node:path";
import type { EmbeddingProvider } from "./provider.js";

type FastEmbedOptions = {
  cacheDir: string;
  model: string;
  showDownloadProgress: boolean;
};

export class FastEmbedProvider implements EmbeddingProvider {
  readonly name = "fastembed";
  readonly model: string;
  private readonly cacheDir: string;
  private readonly showDownloadProgress: boolean;
  private instance: FlagEmbedding | null = null;

  constructor(options: FastEmbedOptions) {
    this.model = options.model;
    this.cacheDir = path.join(options.cacheDir, "fastembed");
    this.showDownloadProgress = options.showDownloadProgress;
  }

  async init(): Promise<void> {
    if (this.instance) {
      return;
    }

    this.instance = await FlagEmbedding.init({
      model: this.model as EmbeddingModel,
      executionProviders: [ExecutionProvider.CPU],
      cacheDir: this.cacheDir,
      showDownloadProgress: this.showDownloadProgress
    });
  }

  async getDimension(): Promise<number> {
    await this.init();
    const models = this.instance!.listSupportedModels();
    const match = models.find((entry) => entry.model === this.model);
    if (!match) {
      throw new Error(`Unsupported embedding model: ${this.model}`);
    }
    return match.dim;
  }

  async embed(texts: string[]): Promise<number[][]> {
    await this.init();
    const batches: number[][] = [];
    for await (const batch of this.instance!.embed(texts, 32)) {
      batches.push(...batch);
    }
    return batches;
  }

  async embedQuery(query: string): Promise<number[]> {
    await this.init();
    return this.instance!.queryEmbed(query);
  }

  async healthCheck(): Promise<{ ok: boolean; detail: string }> {
    try {
      await this.embedQuery("health check");
      return { ok: true, detail: `ready:${this.model}` };
    } catch (error) {
      return {
        ok: false,
        detail: error instanceof Error ? error.message : String(error)
      };
    }
  }
}
