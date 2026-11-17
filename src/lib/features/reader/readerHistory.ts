export type ReaderSourceKind = "article" | "youtube" | "pdf" | "epub" | "text";

export interface ReaderHistoryItem {
  id: string;
  kind: ReaderSourceKind;
  title: string;
  subtitle: string;
  url?: string;
  fileName?: string;
  lastPos?: number;
  openedAt: number;
}

const STORAGE_KEY = "vlearning_reader_history";
const IDB_NAME = "vlearning_reader_cache";
const IDB_STORE = "files";
const MAX_HISTORY = 20;
const MAX_CACHED_FILES = 10;

export function loadReaderHistory(): ReaderHistoryItem[] {
  if (typeof localStorage === "undefined") return [];
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function saveReaderHistory(items: ReaderHistoryItem[]): void {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(items.slice(0, MAX_HISTORY)));
}

function openCacheDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(IDB_NAME, 1);
    req.onupgradeneeded = () => {
      if (!req.result.objectStoreNames.contains(IDB_STORE)) {
        req.result.createObjectStore(IDB_STORE);
      }
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

export async function cacheReaderFile(id: string, data: ArrayBuffer): Promise<void> {
  if (typeof indexedDB === "undefined") return;
  try {
    const db = await openCacheDb();
    await new Promise<void>((resolve, reject) => {
      const tx = db.transaction(IDB_STORE, "readwrite");
      tx.objectStore(IDB_STORE).put(data, id);
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    });
    db.close();
  } catch {
    // Ignore IndexedDB quota/private-mode errors
  }
}

export async function loadCachedReaderFile(id: string): Promise<ArrayBuffer | null> {
  if (typeof indexedDB === "undefined") return null;
  try {
    const db = await openCacheDb();
    const res = await new Promise<ArrayBuffer | null>((resolve, reject) => {
      const tx = db.transaction(IDB_STORE, "readonly");
      const req = tx.objectStore(IDB_STORE).get(id);
      req.onsuccess = () => resolve((req.result as ArrayBuffer | undefined) ?? null);
      req.onerror = () => reject(req.error);
    });
    db.close();
    return res;
  } catch {
    return null;
  }
}

export async function deleteCachedReaderFile(id: string): Promise<void> {
  if (typeof indexedDB === "undefined") return;
  try {
    const db = await openCacheDb();
    await new Promise<void>((resolve) => {
      const tx = db.transaction(IDB_STORE, "readwrite");
      tx.objectStore(IDB_STORE).delete(id);
      tx.oncomplete = () => resolve();
      tx.onerror = () => resolve();
    });
    db.close();
  } catch {
    // Ignore
  }
}

export async function clearAllCachedReaderFiles(): Promise<void> {
  if (typeof indexedDB === "undefined") return;
  try {
    const db = await openCacheDb();
    await new Promise<void>((resolve) => {
      const tx = db.transaction(IDB_STORE, "readwrite");
      tx.objectStore(IDB_STORE).clear();
      tx.oncomplete = () => resolve();
      tx.onerror = () => resolve();
    });
    db.close();
  } catch {
    // Ignore
  }
}

export function buildHistoryId(item: Pick<ReaderHistoryItem, "kind" | "url" | "fileName">): string {
  const key = item.url ? `url:${item.url}` : `file:${item.kind}:${item.fileName ?? ""}`;
  let h = 2166136261;
  for (let i = 0; i < key.length; i++) {
    h ^= key.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return `rh_${(h >>> 0).toString(16)}`;
}

export async function recordHistoryEntry(
  entry: Omit<ReaderHistoryItem, "id" | "openedAt">,
  fileBuffer?: ArrayBuffer,
): Promise<{ item: ReaderHistoryItem; items: ReaderHistoryItem[] }> {
  const id = buildHistoryId(entry);
  const existing = loadReaderHistory().filter((i) => i.id !== id);
  const item: ReaderHistoryItem = {
    ...entry,
    id,
    openedAt: Date.now(),
  };
  const next = [item, ...existing].slice(0, MAX_HISTORY);
  saveReaderHistory(next);

  if (fileBuffer) {
    await cacheReaderFile(id, fileBuffer);
    const fileItems = next.filter((i) => i.fileName);
    if (fileItems.length > MAX_CACHED_FILES) {
      for (const stale of fileItems.slice(MAX_CACHED_FILES)) {
        await deleteCachedReaderFile(stale.id);
      }
    }
  }
  return { item, items: next };
}

export function updateHistoryPosition(id: string | null, lastPos: number): ReaderHistoryItem[] {
  const items = loadReaderHistory();
  if (!id) return items;
  const next = items.map((item) => (item.id === id ? { ...item, lastPos } : item));
  saveReaderHistory(next);
  return next;
}

export async function removeHistoryEntry(id: string): Promise<ReaderHistoryItem[]> {
  const next = loadReaderHistory().filter((i) => i.id !== id);
  saveReaderHistory(next);
  await deleteCachedReaderFile(id);
  return next;
}

export async function clearAllHistory(): Promise<ReaderHistoryItem[]> {
  if (typeof localStorage !== "undefined") {
    localStorage.removeItem(STORAGE_KEY);
  }
  await clearAllCachedReaderFiles();
  return [];
}
