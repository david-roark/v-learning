use tauri::State;

use crate::anki;
use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::models::ApkgImportReport;

#[tauri::command]
pub fn import_apkg(
    db: State<'_, DbState>,
    deck_name: String,
    bytes: Vec<u8>,
) -> AppResult<ApkgImportReport> {
    let mut conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    anki::import_apkg_bytes(&mut conn, &deck_name, &bytes)
}
