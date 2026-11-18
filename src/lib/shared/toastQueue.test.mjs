import assert from "node:assert/strict";
import test from "node:test";
import { enqueueToast, removeToast } from "./toastQueue.ts";

test("enqueueToast ignores empty messages and trims whitespace", () => {
  const empty = enqueueToast([], { id: 1, kind: "info", message: "   ", durationMs: 3000 });
  assert.equal(empty.length, 0);

  const added = enqueueToast([], {
    id: 1,
    kind: "success",
    message: "  Saved! ",
    durationMs: 3000,
  });
  assert.equal(added.length, 1);
  assert.equal(added[0].message, "Saved!");
});

test("enqueueToast deduplicates identical kind+message and caps queue size", () => {
  let list = enqueueToast([], { id: 1, kind: "error", message: "Network error", durationMs: 4000 });
  list = enqueueToast(list, { id: 2, kind: "error", message: "Network error", durationMs: 4000 });
  assert.equal(list.length, 1);
  assert.equal(list[0].id, 2);

  list = enqueueToast(list, { id: 3, kind: "info", message: "A", durationMs: 3000 }, 2);
  list = enqueueToast(list, { id: 4, kind: "info", message: "B", durationMs: 3000 }, 2);
  assert.deepEqual(
    list.map((x) => x.id),
    [3, 4],
  );
});

test("removeToast removes matching id", () => {
  const list = [
    { id: 1, kind: "info", message: "One", durationMs: 3000 },
    { id: 2, kind: "success", message: "Two", durationMs: 3000 },
  ];
  const next = removeToast(list, 1);
  assert.deepEqual(
    next.map((x) => x.id),
    [2],
  );
  assert.equal(removeToast(next, 999).length, 1);
});

test("enqueueToast keeps distinct kinds with identical message", () => {
  let list = enqueueToast([], { id: 1, kind: "info", message: "Done", durationMs: 3000 });
  list = enqueueToast(list, { id: 2, kind: "success", message: "Done", durationMs: 3000 });
  assert.equal(list.length, 2);
});
