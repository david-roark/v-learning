import { enqueueToast, removeToast, type ToastItem, type ToastKind } from "./toastQueue";

let nextId = 1;
const timers: Record<number, ReturnType<typeof setTimeout>> = {};

let items = $state<ToastItem[]>([]);

function push(kind: ToastKind, message: unknown, durationMs: number): number {
  const text = String(message ?? "").trim();
  if (!text) return -1;

  const id = nextId++;
  const nextList = enqueueToast(items, { id, kind, message: text, durationMs });

  // Clean up timers for any evicted or deduplicated toasts
  const activeIds = new Set(nextList.map((t) => t.id));
  for (const key of Object.keys(timers)) {
    const timerId = Number(key);
    if (!activeIds.has(timerId)) {
      clearTimeout(timers[timerId]);
      delete timers[timerId];
    }
  }

  items = nextList;

  if (durationMs > 0) {
    timers[id] = setTimeout(() => {
      dismiss(id);
    }, durationMs);
  }

  return id;
}

function dismiss(id: number) {
  const handle = timers[id];
  if (handle) {
    clearTimeout(handle);
    delete timers[id];
  }
  items = removeToast(items, id);
}

function clear() {
  for (const key of Object.keys(timers)) {
    const timerId = Number(key);
    clearTimeout(timers[timerId]);
    delete timers[timerId];
  }
  items = [];
}

export const toast = {
  get items(): ToastItem[] {
    return items;
  },
  success: (message: unknown, durationMs = 3500) => push("success", message, durationMs),
  error: (message: unknown, durationMs = 5500) => push("error", message, durationMs),
  info: (message: unknown, durationMs = 3500) => push("info", message, durationMs),
  warning: (message: unknown, durationMs = 4500) => push("warning", message, durationMs),
  dismiss,
  clear,
};
