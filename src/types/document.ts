export type DocumentRow = {
  id: string;
  sourceType: "file" | "stdin" | "text";
  sourcePath: string | null;
  sourceHash: string;
  title: string | null;
  tags: string[];
  createdAt: string;
  updatedAt: string;
  metadata: string;
};
