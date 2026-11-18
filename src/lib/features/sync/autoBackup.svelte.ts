import { toast } from "$lib/shared/toast.svelte";
import { syncApi } from "./api";

const AUTO_BACKUP_DEBOUNCE_MS = 45_000;

let timer: ReturnType<typeof setTimeout> | null = null;
let dirty = false;
let running = false;

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
      toast.warning(
        "Google Drive có bản sao lưu mới hơn từ thiết bị khác. Hãy mở Sao lưu & Đồng bộ để kiểm tra trước khi ghi đè.",
      );
    }
  } catch {
    // Background auto-backup failures should stay non-intrusive
  } finally {
    running = false;
  }
}

export function scheduleAutoBackup() {
  dirty = true;
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
