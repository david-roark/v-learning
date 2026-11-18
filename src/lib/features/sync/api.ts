import { invokeCmd } from "$lib/shared/tauri";
import type {
  BackupRestoreReport,
  GoogleDriveBackupInfo,
  GoogleDrivePushResult,
  GoogleDriveStartupSyncResult,
  GoogleDriveStatus,
  GoogleOAuthStart,
  LocalBackupEntry,
  WebDavSettings,
} from "$lib/shared/types";

export const syncApi = {
  exportBackup: () => invokeCmd<string>("export_backup"),
  importBackup: (jsonStr: string) => invokeCmd<BackupRestoreReport>("import_backup", { jsonStr }),
  getWebDavSettings: () => invokeCmd<WebDavSettings>("get_webdav_settings"),
  saveWebDavSettings: (settings: WebDavSettings) =>
    invokeCmd<void>("save_webdav_settings", { settings }),
  pushWebDav: () => invokeCmd<void>("sync_webdav_push"),
  pullWebDav: () => invokeCmd<BackupRestoreReport>("sync_webdav_pull"),
  getGoogleDriveStatus: () => invokeCmd<GoogleDriveStatus>("google_drive_status"),
  startGoogleDriveLogin: (clientId?: string, clientSecret?: string) =>
    invokeCmd<GoogleOAuthStart>("google_drive_start_login", {
      clientId: clientId?.trim() || undefined,
      clientSecret: clientSecret?.trim() || undefined,
    }),
  completeGoogleDriveLogin: () => invokeCmd<GoogleDriveStatus>("google_drive_complete_login"),
  cancelGoogleDriveLogin: () => invokeCmd<void>("google_drive_cancel_login"),
  logoutGoogleDrive: () => invokeCmd<GoogleDriveStatus>("google_drive_logout"),
  pushGoogleDrive: (force = false) =>
    invokeCmd<GoogleDrivePushResult>("sync_google_drive_push", { force }),
  pullGoogleDrive: () => invokeCmd<BackupRestoreReport>("sync_google_drive_pull"),
  mergeGoogleDrive: () => invokeCmd<BackupRestoreReport>("sync_google_drive_merge"),
  markGoogleDriveDirty: () => invokeCmd<void>("google_drive_mark_dirty"),
  startupGoogleDriveSync: () =>
    invokeCmd<GoogleDriveStartupSyncResult>("google_drive_startup_sync"),
  getGoogleDriveBackupInfo: () => invokeCmd<GoogleDriveBackupInfo>("google_drive_backup_info"),
  setGoogleDriveAutoSync: (enabled: boolean) =>
    invokeCmd<GoogleDriveStatus>("google_drive_set_auto_sync", { enabled }),
  listLocalBackups: () => invokeCmd<LocalBackupEntry[]>("list_local_backups"),
  createLocalBackup: () => invokeCmd<LocalBackupEntry | null>("create_local_backup"),
  restoreLocalBackup: (filename: string) =>
    invokeCmd<BackupRestoreReport>("restore_local_backup", { filename }),
};
