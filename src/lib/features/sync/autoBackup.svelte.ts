import { toast } from "$lib/shared/toast.svelte";
import { syncApi } from "./api";

const AUTO_BACKUP_DEBOUNCE_MS = 45_000;

let timer: ReturnType<typeof setTimeout> | null = null;
let dirty = false;
let running = false;
let pendingConflictTime = $state<string | null>(null);

export const MUTATING_COMMANDS: Record<string, true> = {
  create_deck: true,
  delete_deck: true,
  create_note: true,
  update_note: true,
  delete_note: true,
  submit_review: true,
  undo_review: true,
  import_apkg: true,
};

export function getPendingDriveConflict(): string | null {
  return pendingConflictTime;
}

export function setPendingDriveConflict(time: string | null) {
  pendingConflictTime = time;
}

async function runAutoBackup(includeDrive: boolean) {
  if (running || !dirty) return;
  running = true;
  dirty = false;
  try {
    await syncApi.createLocalBackup();
    if (!includeDrive) return;

    const status = await syncApi.getGoogleDriveStatus();
    if (!status.connected || !status.autoSync) return;

    const res = await syncApi.pushGoogleDrive(false);
    if (res.conflict) {
      pendingConflictTime = res.remoteModifiedTime;
      toast.warning(
        "Google Drive có bản sao lưu mới hơn từ thiết bị khác. Hãy mở Sao lưu & Đồng bộ để chọn Gộp hoặc Ghi đè.",
      );
    } else {
      pendingConflictTime = null;
    }
  } catch {
    // Background auto-backup failures should stay non-intrusive
  } finally {
    running = false;
  }
}

export function scheduleAutoBackup() {
  dirty = true;
  void syncApi.markGoogleDriveDirty().catch(() => {});
  if (timer) clearTimeout(timer);
  timer = setTimeout(() => {
    timer = null;
    void runAutoBackup(true);
  }, AUTO_BACKUP_DEBOUNCE_MS);
}

export function flushPendingAutoBackup() {
  if (!dirty) return;
  if (timer) {
    clearTimeout(timer);
    timer = null;
  }
  void runAutoBackup(false);
}

export async function runStartupGoogleDriveSync() {
  try {
    const res = await syncApi.startupGoogleDriveSync();
    if (res.action === "pulled" && res.report) {
      pendingConflictTime = null;
      toast.success(
        `Đã tự động đồng bộ từ Google Drive (${res.report.decks} bộ thẻ, ${res.report.notes} ghi chú).`,
      );
      window.dispatchEvent(new CustomEvent("vlearning:data-restored"));
    } else if (res.action === "pushed") {
      pendingConflictTime = null;
      toast.info("Đã đồng bộ thay đổi chưa lưu lên Google Drive.");
    } else if (res.action === "conflict") {
      pendingConflictTime = res.remoteModifiedTime;
      toast.warning(
        "Phát hiện xung đột giữa máy này và Google Drive. Hãy mở Sao lưu & Đồng bộ để Gộp hoặc chọn bản giữ lại.",
        7000,
      );
    }
  } catch {
    // Offline or network error at startup should stay silent
  }
}
