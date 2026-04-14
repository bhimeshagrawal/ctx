export function renderData(data: unknown, json = false): void {
  if (json) {
    console.log(JSON.stringify(data, null, 2));
    return;
  }

  if (isSearchResult(data)) {
    console.log(`query: ${data.query}`);
    console.log(`results: ${data.count}`);
    for (const [index, item] of data.results.entries()) {
      console.log("");
      console.log(`${index + 1}. ${item.title ?? item.sourcePath ?? item.documentId}`);
      console.log(`score: ${item.finalScore.toFixed(3)} tags: ${item.tags.join(", ") || "-"}`);
      console.log(item.content.length > 240 ? `${item.content.slice(0, 240)}...` : item.content);
    }
    return;
  }

  if (typeof data === "object" && data !== null) {
    for (const [key, value] of Object.entries(data)) {
      console.log(`${key}: ${formatValue(value)}`);
    }
    return;
  }

  console.log(String(data));
}

function formatValue(value: unknown): string {
  if (Array.isArray(value)) {
    return value.join(", ");
  }
  if (typeof value === "object" && value !== null) {
    return JSON.stringify(value);
  }
  return String(value);
}

function isSearchResult(
  value: unknown
): value is {
  query: string;
  count: number;
  results: Array<{
    documentId: string;
    title: string | null;
    sourcePath: string | null;
    finalScore: number;
    tags: string[];
    content: string;
  }>;
} {
  return (
    typeof value === "object" &&
    value !== null &&
    "query" in value &&
    "count" in value &&
    "results" in value &&
    Array.isArray((value as { results: unknown[] }).results)
  );
}
