export type ToastKind = "success" | "error" | "info" | "warning";

export interface ToastItem {
  id: number;
  kind: ToastKind;
  message: string;
  durationMs: number;
}

export const MAX_VISIBLE_TOASTS = 4;

export function enqueueToast(
  items: ToastItem[],
  next: ToastItem,
  maxItems = MAX_VISIBLE_TOASTS,
): ToastItem[] {
  const cleanMsg = next.message.trim();
  if (!cleanMsg) return items;

  // Deduplicate identical active message (keep newest)
  const filtered = items.filter((item) => !(item.kind === next.kind && item.message === cleanMsg));
  const updated = [...filtered, { ...next, message: cleanMsg }];
  if (updated.length <= maxItems) return updated;
  return updated.slice(updated.length - maxItems);
}

export function removeToast(items: ToastItem[], id: number): ToastItem[] {
  return items.filter((item) => item.id !== id);
}
