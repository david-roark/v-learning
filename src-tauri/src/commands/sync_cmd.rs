use tauri::State;

use crate::db::{DbState, settings_repo};
use crate::error::{AppError, AppResult};
use crate::models::{BackupRestoreReport, WebDavSettings};
use crate::sync;

#[tauri::command]
pub fn export_backup(db: State<'_, DbState>) -> AppResult<String> {
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    sync::export_backup_json(&conn)
}

#[tauri::command]
pub fn import_backup(db: State<'_, DbState>, json_str: String) -> AppResult<BackupRestoreReport> {
    let mut conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
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
pub async fn sync_webdav_pull(db: State<'_, DbState>) -> AppResult<BackupRestoreReport> {
    let settings = {
        let conn =
            db.0.lock()
                .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
        settings_repo::get_webdav_settings(&conn)?
    };
    let json_str = sync::webdav_pull(&settings).await?;
    let mut conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    sync::import_backup_json(&mut conn, &json_str)
}
