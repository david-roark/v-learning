use tauri::{AppHandle, State};

use crate::db::{DbState, dict_repo, settings_repo};
use crate::error::{AppError, AppResult};
use crate::media;
use crate::models::TtsSettings;

#[tauri::command]
pub fn get_tts_settings(db: State<'_, DbState>) -> AppResult<TtsSettings> {
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    settings_repo::get_tts_settings(&conn)
}

#[tauri::command]
pub fn save_tts_settings(db: State<'_, DbState>, settings: TtsSettings) -> AppResult<()> {
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    settings_repo::save_tts_settings(&conn, &settings)
}

#[tauri::command]
pub async fn fetch_audio(
    app: AppHandle,
    db: State<'_, DbState>,
    word: String,
) -> AppResult<Vec<u8>> {
    let (audio_url, tts_settings) = {
        let conn =
            db.0.lock()
                .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
        let audio = dict_repo::find_exact(&conn, &word)?.and_then(|entry| entry.audio_url);
        let settings = settings_repo::get_tts_settings(&conn)?;
        (audio, settings)
    };

    media::get_or_fetch_audio(&app, &word, audio_url, &tts_settings).await
}

#[tauri::command]
pub async fn preview_tts_provider(
    provider: String,
    voice: String,
    text: String,
) -> AppResult<Vec<u8>> {
    media::preview_provider_audio(&provider, &voice, &text).await
}

#[tauri::command]
pub fn speak_system_tts(text: String, speed: Option<f64>) -> AppResult<()> {
    media::speak_with_os_tts(&text, speed.unwrap_or(1.0))
}
