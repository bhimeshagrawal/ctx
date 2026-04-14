export function normalizeContent(content: string): string {
  return content.replaceAll("\r\n", "\n").replace(/\n{3,}/g, "\n\n").trim();
}
