use tauri::{AppHandle, State};

use crate::db::{DbState, settings_repo};
use crate::error::{AppError, AppResult};
use crate::models::{
    BackupRestoreReport, GoogleDriveBackupInfo, GoogleDrivePushResult, GoogleDriveStatus,
    GoogleOAuthStart, LocalBackupEntry, WebDavSettings,
};
use crate::sync;
use crate::sync::gdrive::PendingGoogleOAuth;
use crate::sync::snapshots::{
    MAX_LOCAL_SNAPSHOTS, backup_dir, create_snapshot_in_dir, list_snapshots_in_dir,
    read_snapshot_bytes_in_dir,
};

#[tauri::command]
pub fn export_backup(db: State<'_, DbState>) -> AppResult<String> {
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    sync::export_backup_json(&conn)
}

#[tauri::command]
pub fn import_backup(
    app: AppHandle,
    db: State<'_, DbState>,
    json_str: String,
) -> AppResult<BackupRestoreReport> {
    let dir = backup_dir(&app)?;
    let mut conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    let _ = create_snapshot_in_dir(&dir, &conn, "pre-restore", MAX_LOCAL_SNAPSHOTS)?;
    sync::import_backup_json(&mut conn, &json_str)
}

#[tauri::command]
pub fn get_webdav_settings(db: State<'_, DbState>) -> AppResult<WebDavSettings> {
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    settings_repo::get_webdav_settings(&conn)
}

#[tauri::command]
pub fn save_webdav_settings(db: State<'_, DbState>, settings: WebDavSettings) -> AppResult<()> {
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    settings_repo::save_webdav_settings(&conn, &settings)
}

#[tauri::command]
pub async fn sync_webdav_push(db: State<'_, DbState>) -> AppResult<()> {
    let (settings, payload) = {
        let conn =
            db.0.lock()
                .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
        (
            settings_repo::get_webdav_settings(&conn)?,
            sync::export_backup_json(&conn)?,
        )
    };
    sync::webdav_push(&settings, payload).await
}

#[tauri::command]
pub async fn sync_webdav_pull(
    app: AppHandle,
    db: State<'_, DbState>,
) -> AppResult<BackupRestoreReport> {
    let settings = {
        let conn =
            db.0.lock()
                .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
        settings_repo::get_webdav_settings(&conn)?
    };
    let json_str = sync::webdav_pull(&settings).await?;
    let dir = backup_dir(&app)?;
    let mut conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    let _ = create_snapshot_in_dir(&dir, &conn, "pre-restore", MAX_LOCAL_SNAPSHOTS)?;
    sync::import_backup_json(&mut conn, &json_str)
}

#[tauri::command]
pub fn google_drive_status(db: State<'_, DbState>) -> AppResult<GoogleDriveStatus> {
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    sync::gdrive::get_status(&conn)
}

#[tauri::command]
pub fn google_drive_start_login(
    db: State<'_, DbState>,
    pending: State<'_, PendingGoogleOAuth>,
    client_id: Option<String>,
    client_secret: Option<String>,
) -> AppResult<GoogleOAuthStart> {
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    sync::gdrive::start_login(
        &conn,
        &pending,
        client_id.as_deref(),
        client_secret.as_deref(),
    )
}

#[tauri::command]
pub async fn google_drive_complete_login(
    db: State<'_, DbState>,
    pending: State<'_, PendingGoogleOAuth>,
) -> AppResult<GoogleDriveStatus> {
    let session = sync::gdrive::take_pending_session(&pending)?;
    let (token, email) = sync::gdrive::complete_login_flow(session).await?;
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    sync::gdrive::save_login_tokens(&conn, &token, &email)
}

#[tauri::command]
pub fn google_drive_logout(db: State<'_, DbState>) -> AppResult<GoogleDriveStatus> {
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    sync::gdrive::logout(&conn)
}

#[tauri::command]
pub fn google_drive_cancel_login(pending: State<'_, PendingGoogleOAuth>) -> AppResult<()> {
    sync::gdrive::cancel_login(&pending)
}

#[tauri::command]
pub async fn sync_google_drive_push(
    db: State<'_, DbState>,
    force: Option<bool>,
) -> AppResult<GoogleDrivePushResult> {
    let (settings, payload) = {
        let conn =
            db.0.lock()
                .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
        (
            settings_repo::get_google_drive_settings(&conn)?,
            sync::export_backup_gzip(&conn)?,
        )
    };
    let (mut updated, changed) = sync::gdrive::refresh_access_token_if_needed(settings).await?;
    if changed {
        let conn =
            db.0.lock()
                .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
        settings_repo::save_google_drive_settings(&conn, &updated)?;
    }

    if !force.unwrap_or(false) {
        let info = sync::gdrive::drive_get_backup_info(&updated.access_token).await?;
        if info.exists
            && sync::gdrive::check_push_conflict(
                info.modified_time.as_deref(),
                &updated.last_synced_remote_modified,
            )
        {
            return Ok(GoogleDrivePushResult {
                pushed: false,
                conflict: true,
                remote_modified_time: info.modified_time,
            });
        }
    }

    let remote_modified_time =
        sync::gdrive::drive_push_backup(&updated.access_token, &payload).await?;
    if let Some(ref mod_time) = remote_modified_time {
        updated.last_synced_remote_modified = mod_time.clone();
        let conn =
            db.0.lock()
                .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
        settings_repo::save_google_drive_settings(&conn, &updated)?;
    }

    Ok(GoogleDrivePushResult {
        pushed: true,
        conflict: false,
        remote_modified_time,
    })
}

#[tauri::command]
pub async fn sync_google_drive_pull(
    app: AppHandle,
    db: State<'_, DbState>,
) -> AppResult<BackupRestoreReport> {
    let settings = {
        let conn =
            db.0.lock()
                .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
        settings_repo::get_google_drive_settings(&conn)?
    };
    let (mut updated, changed) = sync::gdrive::refresh_access_token_if_needed(settings).await?;
    if changed {
        let conn =
            db.0.lock()
                .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
        settings_repo::save_google_drive_settings(&conn, &updated)?;
    }
    let (bytes, mod_time) = sync::gdrive::drive_pull_backup(&updated.access_token).await?;
    let dir = backup_dir(&app)?;
    let mut conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    let _ = create_snapshot_in_dir(&dir, &conn, "pre-restore", MAX_LOCAL_SNAPSHOTS)?;
    let report = sync::import_backup_bytes(&mut conn, &bytes)?;
    if let Some(mt) = mod_time {
        updated.last_synced_remote_modified = mt;
        settings_repo::save_google_drive_settings(&conn, &updated)?;
    }
    Ok(report)
}

#[tauri::command]
pub fn google_drive_set_auto_sync(
    db: State<'_, DbState>,
    enabled: bool,
) -> AppResult<GoogleDriveStatus> {
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    sync::gdrive::set_auto_sync(&conn, enabled)
}

#[tauri::command]
pub fn list_local_backups(app: AppHandle) -> AppResult<Vec<LocalBackupEntry>> {
    let dir = backup_dir(&app)?;
    list_snapshots_in_dir(&dir)
}

#[tauri::command]
pub fn create_local_backup(
    app: AppHandle,
    db: State<'_, DbState>,
) -> AppResult<Option<LocalBackupEntry>> {
    let dir = backup_dir(&app)?;
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    create_snapshot_in_dir(&dir, &conn, "auto", MAX_LOCAL_SNAPSHOTS)
}

#[tauri::command]
pub fn restore_local_backup(
    app: AppHandle,
    db: State<'_, DbState>,
    filename: String,
) -> AppResult<BackupRestoreReport> {
    let dir = backup_dir(&app)?;
    let bytes = read_snapshot_bytes_in_dir(&dir, &filename)?;
    let mut conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    let _ = create_snapshot_in_dir(&dir, &conn, "pre-restore", MAX_LOCAL_SNAPSHOTS)?;
    sync::import_backup_bytes(&mut conn, &bytes)
}

#[tauri::command]
pub async fn google_drive_backup_info(db: State<'_, DbState>) -> AppResult<GoogleDriveBackupInfo> {
    let settings = {
        let conn =
            db.0.lock()
                .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
        settings_repo::get_google_drive_settings(&conn)?
    };
    let (updated, changed) = sync::gdrive::refresh_access_token_if_needed(settings).await?;
    if changed {
        let conn =
            db.0.lock()
                .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
        settings_repo::save_google_drive_settings(&conn, &updated)?;
    }
    sync::gdrive::drive_get_backup_info(&updated.access_token).await
}
