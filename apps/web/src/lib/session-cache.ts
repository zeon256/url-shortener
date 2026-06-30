import type { HistoryRecord } from "./history";

const HISTORY_KEY = "url-shortener:history";
const HEALTH_KEY = "url-shortener:api-health";

export function readCachedHistory(): HistoryRecord[] {
  try {
    const raw = sessionStorage.getItem(HISTORY_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as HistoryRecord[];
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (entry) =>
        typeof entry.id === "string" &&
        typeof entry.originalUrl === "string" &&
        typeof entry.shortUrl === "string" &&
        typeof entry.shortCode === "string" &&
        typeof entry.createdAt === "number"
    );
  } catch {
    return [];
  }
}

export function writeCachedHistory(records: HistoryRecord[]) {
  try {
    sessionStorage.setItem(HISTORY_KEY, JSON.stringify(records));
  } catch {
    // Storage full or unavailable — ignore.
  }
}

export function readCachedHealth(): boolean | undefined {
  try {
    const raw = sessionStorage.getItem(HEALTH_KEY);
    if (raw === "true") return true;
    if (raw === "false") return false;
    return undefined;
  } catch {
    return undefined;
  }
}

export function writeCachedHealth(online: boolean) {
  try {
    sessionStorage.setItem(HEALTH_KEY, online ? "true" : "false");
  } catch {
    // Ignore.
  }
}

export function toHistoryRecords(entries: { id: string; originalUrl: string; shortUrl: string; shortCode: string; createdAt: number }[]): HistoryRecord[] {
  return entries.map(({ id, originalUrl, shortUrl, shortCode, createdAt }) => ({
    id,
    originalUrl,
    shortUrl,
    shortCode,
    createdAt,
  }));
}
