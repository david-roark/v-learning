use tauri::{AppHandle, State};

use crate::db::{DbState, dict_repo};
use crate::error::{AppError, AppResult};
use crate::media;

#[tauri::command]
pub async fn fetch_audio(
    app: AppHandle,
    db: State<'_, DbState>,
    word: String,
) -> AppResult<Vec<u8>> {
    let audio_url = {
        let conn =
            db.0.lock()
                .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
        dict_repo::find_exact(&conn, &word)?.and_then(|entry| entry.audio_url)
    };

    media::get_or_fetch_audio(&app, &word, audio_url).await
}
