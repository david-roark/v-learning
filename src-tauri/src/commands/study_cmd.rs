use tauri::State;

use crate::db::{DbState, card_repo};
use crate::error::{AppError, AppResult};
use crate::models::{Card, DailyStats, StudyCard};

#[tauri::command]
pub fn get_study_queue(
    db: State<'_, DbState>,
    deck_id: String,
    limit: Option<i64>,
) -> AppResult<Vec<StudyCard>> {
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    card_repo::get_study_queue(&conn, &deck_id, limit.unwrap_or(50).clamp(1, 200))
}

#[tauri::command]
pub fn submit_review(
    db: State<'_, DbState>,
    card_id: String,
    rating: i64,
    duration_ms: Option<i64>,
) -> AppResult<()> {
    let mut conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    card_repo::submit_review(&mut conn, &card_id, rating, duration_ms)
}

#[tauri::command]
pub fn undo_review(db: State<'_, DbState>, prev_card: Card) -> AppResult<()> {
    let mut conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    card_repo::undo_review(&mut conn, &prev_card)
}

#[tauri::command]
pub fn get_daily_stats(db: State<'_, DbState>) -> AppResult<DailyStats> {
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    card_repo::get_daily_stats(&conn)
}
