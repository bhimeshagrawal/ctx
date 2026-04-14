import { readFile } from "node:fs/promises";
import path from "node:path";

export type InputPayload = {
  sourceType: "file" | "stdin" | "text";
  sourcePath: string | null;
  title: string | null;
  content: string;
};

type ReadInputOptions = {
  file?: string;
  text?: string;
  stdin?: boolean;
};

export async function readInput(options: ReadInputOptions): Promise<InputPayload> {
  const enabledSources = [Boolean(options.file), Boolean(options.text), Boolean(options.stdin)].filter(Boolean);
  if (enabledSources.length !== 1) {
    throw new Error("Exactly one input source is required: --file, --text, or --stdin");
  }

  if (options.file) {
    const content = await readFile(options.file, "utf8");
    return {
      sourceType: "file",
      sourcePath: path.resolve(options.file),
      title: path.basename(options.file),
      content
    };
  }

  if (options.text) {
    return {
      sourceType: "text",
      sourcePath: null,
      title: null,
      content: options.text
    };
  }

  return {
    sourceType: "stdin",
    sourcePath: null,
    title: null,
    content: await readStdin()
  };
}

async function readStdin(): Promise<string> {
  const chunks: Buffer[] = [];
  for await (const chunk of process.stdin) {
    chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
  }
  return Buffer.concat(chunks).toString("utf8");
}
