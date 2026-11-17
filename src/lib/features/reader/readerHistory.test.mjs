import assert from "node:assert/strict";
import test from "node:test";
import {
  buildHistoryId,
  clearAllHistory,
  loadReaderHistory,
  recordHistoryEntry,
  removeHistoryEntry,
  updateHistoryPosition,
} from "./readerHistory.ts";

function installMockLocalStorage() {
  const store = new Map();
  globalThis.localStorage = {
    getItem(key) {
      return store.has(key) ? store.get(key) : null;
    },
    setItem(key, value) {
      store.set(key, String(value));
    },
    removeItem(key) {
      store.delete(key);
    },
    clear() {
      store.clear();
    },
  };
  return store;
}

test("buildHistoryId is deterministic and distinguishes url vs fileName", () => {
  const a1 = buildHistoryId({ kind: "article", url: "https://example.com/a" });
  const a2 = buildHistoryId({ kind: "article", url: "https://example.com/a" });
  const b = buildHistoryId({ kind: "youtube", url: "https://youtu.be/abc" });
  const f1 = buildHistoryId({ kind: "pdf", fileName: "book.pdf" });
  const f2 = buildHistoryId({ kind: "epub", fileName: "book.pdf" });

  assert.equal(a1, a2);
  assert.ok(a1.startsWith("rh_"));
  assert.notEqual(a1, b);
  assert.notEqual(f1, f2);
});

test("recordHistoryEntry deduplicates, keeps newest first, and caps at 20 items", async () => {
  installMockLocalStorage();

  for (let i = 1; i <= 25; i++) {
    await recordHistoryEntry({
      kind: "article",
      title: `Article ${i}`,
      subtitle: "example.com",
      url: `https://example.com/${i}`,
    });
  }

  let items = loadReaderHistory();
  assert.equal(items.length, 20);
  assert.equal(items[0].title, "Article 25");

  // Re-opening an older item moves it to the top without creating duplicates
  const reopened = await recordHistoryEntry({
    kind: "article",
    title: "Article 10 (Updated)",
    subtitle: "example.com",
    url: "https://example.com/10",
  });
  items = reopened.items;
  assert.equal(items.length, 20);
  assert.equal(items[0].title, "Article 10 (Updated)");
  assert.equal(items.filter((i) => i.url === "https://example.com/10").length, 1);
});

test("updateHistoryPosition, removeHistoryEntry, and clearAllHistory update storage", async () => {
  installMockLocalStorage();

  const { item } = await recordHistoryEntry({
    kind: "youtube",
    title: "TED Talk",
    subtitle: "youtube.com",
    url: "https://youtu.be/xyz",
  });

  const updated = updateHistoryPosition(item.id, 142);
  assert.equal(updated[0].lastPos, 142);
  assert.equal(loadReaderHistory()[0].lastPos, 142);

  const afterRemove = await removeHistoryEntry(item.id);
  assert.equal(afterRemove.length, 0);

  await recordHistoryEntry({
    kind: "text",
    title: "Notes",
    subtitle: "100 words",
  });
  assert.equal(loadReaderHistory().length, 1);

  const cleared = await clearAllHistory();
  assert.equal(cleared.length, 0);
  assert.equal(loadReaderHistory().length, 0);
});

test("loadReaderHistory returns empty array on corrupted JSON", () => {
  const store = installMockLocalStorage();
  store.set("vlearning_reader_history", "{bad json");
  assert.deepEqual(loadReaderHistory(), []);
});
