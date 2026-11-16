use tauri::State;

use crate::ai;
use crate::db::{DbState, settings_repo};
use crate::error::{AppError, AppResult};
use crate::models::{AiPracticeFeedback, AiSettings};

#[tauri::command]
pub fn get_ai_settings(db: State<'_, DbState>) -> AppResult<AiSettings> {
    let conn = db
        .0
        .lock()
        .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    settings_repo::get_ai_settings(&conn)
}

#[tauri::command]
pub fn save_ai_settings(db: State<'_, DbState>, settings: AiSettings) -> AppResult<()> {
    let conn = db
        .0
        .lock()
        .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    settings_repo::save_ai_settings(&conn, &settings)
}

#[tauri::command]
pub async fn list_ai_models(base_url: String, api_key: String) -> AppResult<Vec<String>> {
    ai::fetch_models(&base_url, &api_key).await
}

#[tauri::command]
pub async fn evaluate_ai_practice(
    db: State<'_, DbState>,
    target_words: Vec<String>,
    user_text: String,
) -> AppResult<AiPracticeFeedback> {
    let settings = {
        let conn = db
            .0
            .lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
        settings_repo::get_ai_settings(&conn)?
    };

    ai::evaluate_practice(&settings, &target_words, &user_text).await
}
