export type TextChunk = {
  index: number;
  content: string;
  tokenEstimate: number;
};

export function chunkText(content: string, chunkSize: number, chunkOverlap: number): TextChunk[] {
  if (chunkOverlap >= chunkSize) {
    throw new Error("Chunk overlap must be smaller than chunk size");
  }

  const chunks: TextChunk[] = [];
  let cursor = 0;
  let index = 0;

  while (cursor < content.length) {
    const end = Math.min(content.length, cursor + chunkSize);
    const chunk = content.slice(cursor, end).trim();
    if (chunk.length > 0) {
      chunks.push({
        index,
        content: chunk,
        tokenEstimate: Math.max(1, Math.ceil(chunk.length / 4))
      });
      index += 1;
    }

    if (end >= content.length) {
      break;
    }
    cursor = Math.max(0, end - chunkOverlap);
  }

  return chunks;
}
