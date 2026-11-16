import { invokeCmd } from "$lib/shared/tauri";
import type { BackupRestoreReport, WebDavSettings } from "$lib/shared/types";

export const syncApi = {
  exportBackup: () => invokeCmd<string>("export_backup"),
  importBackup: (jsonStr: string) => invokeCmd<BackupRestoreReport>("import_backup", { jsonStr }),
  getWebDavSettings: () => invokeCmd<WebDavSettings>("get_webdav_settings"),
  saveWebDavSettings: (settings: WebDavSettings) =>
    invokeCmd<void>("save_webdav_settings", { settings }),
  pushWebDav: () => invokeCmd<void>("sync_webdav_push"),
  pullWebDav: () => invokeCmd<BackupRestoreReport>("sync_webdav_pull"),
};
