const DB_NAME = "url-shortener";
const DB_VERSION = 1;
const STORE = "history";
const MAX_ENTRIES = 5;

export const HISTORY_MAX_ENTRIES = MAX_ENTRIES;

export interface HistoryRecord {
  id: string;
  originalUrl: string;
  shortUrl: string;
  shortCode: string;
  createdAt: number;
}

function openDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);

    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains(STORE)) {
        const store = db.createObjectStore(STORE, { keyPath: "id" });
        store.createIndex("originalUrl", "originalUrl", { unique: true });
        store.createIndex("createdAt", "createdAt");
      }
    };

    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error ?? new Error("IndexedDB open failed"));
  });
}

function runTransaction<T>(
  mode: IDBTransactionMode,
  run: (store: IDBObjectStore) => IDBRequest<T>
): Promise<T> {
  return openDb().then(
    (db) =>
      new Promise((resolve, reject) => {
        const tx = db.transaction(STORE, mode);
        const store = tx.objectStore(STORE);
        const request = run(store);

        request.onsuccess = () => resolve(request.result);
        request.onerror = () =>
          reject(request.error ?? new Error("IndexedDB request failed"));
      })
  );
}

async function trimStale(records: HistoryRecord[]): Promise<HistoryRecord[]> {
  const sorted = records.sort((a, b) => b.createdAt - a.createdAt);
  const kept = sorted.slice(0, MAX_ENTRIES);
  const stale = sorted.slice(MAX_ENTRIES);

  if (stale.length > 0) {
    await Promise.all(
      stale.map((entry) =>
        runTransaction("readwrite", (store) => store.delete(entry.id))
      )
    );
  }

  return kept;
}

export async function loadHistory(): Promise<HistoryRecord[]> {
  const records = await runTransaction("readonly", (store) => store.getAll());
  return trimStale(records);
}

export async function findByOriginalUrl(
  originalUrl: string
): Promise<HistoryRecord | undefined> {
  return runTransaction("readonly", (store) =>
    store.index("originalUrl").get(originalUrl)
  );
}

export async function saveHistoryRecord(record: HistoryRecord): Promise<void> {
  await runTransaction("readwrite", (store) => store.put(record));
  const records = await runTransaction("readonly", (store) => store.getAll());
  await trimStale(records);
}

export async function bumpHistoryRecord(record: HistoryRecord): Promise<void> {
  await saveHistoryRecord({ ...record, createdAt: Date.now() });
}

export function createHistoryId() {
  return typeof crypto !== "undefined" && "randomUUID" in crypto
    ? crypto.randomUUID()
    : `${Date.now()}-${Math.random().toString(36).slice(2)}`;
}
