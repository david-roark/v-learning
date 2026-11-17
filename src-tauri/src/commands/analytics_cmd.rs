use tauri::State;

use crate::db::{DbState, analytics_repo};
use crate::error::{AppError, AppResult};
use crate::models::AnalyticsSummary;

#[tauri::command]
pub fn get_analytics_summary(
    db: State<'_, DbState>,
    deck_id: Option<String>,
) -> AppResult<AnalyticsSummary> {
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    analytics_repo::get_analytics_summary(&conn, deck_id.as_deref())
}
