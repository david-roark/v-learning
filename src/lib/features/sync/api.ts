import { invokeCmd } from "$lib/shared/tauri";
import type {
  BackupRestoreReport,
  GoogleDriveBackupInfo,
  GoogleDriveStatus,
  GoogleOAuthStart,
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
  pushGoogleDrive: () => invokeCmd<void>("sync_google_drive_push"),
  pullGoogleDrive: () => invokeCmd<BackupRestoreReport>("sync_google_drive_pull"),
  getGoogleDriveBackupInfo: () => invokeCmd<GoogleDriveBackupInfo>("google_drive_backup_info"),
};
