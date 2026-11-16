use tauri::State;

use crate::db::{DbState, note_repo};
use crate::error::{AppError, AppResult};
use crate::models::{CreateNoteInput, Note, UpdateNoteInput};

#[tauri::command]
pub fn list_notes(db: State<'_, DbState>, deck_id: String) -> AppResult<Vec<Note>> {
    let conn = db
        .0
        .lock()
        .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    note_repo::list_notes_by_deck(&conn, &deck_id)
}

#[tauri::command]
pub fn create_note(db: State<'_, DbState>, input: CreateNoteInput) -> AppResult<Note> {
    let mut conn = db
        .0
        .lock()
        .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    note_repo::create_note(&mut conn, input)
}

#[tauri::command]
pub fn update_note(db: State<'_, DbState>, input: UpdateNoteInput) -> AppResult<()> {
    let conn = db
        .0
        .lock()
        .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    note_repo::update_note(&conn, input)
}

#[tauri::command]
pub fn delete_note(db: State<'_, DbState>, note_id: String) -> AppResult<()> {
    let conn = db
        .0
        .lock()
        .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    note_repo::delete_note(&conn, &note_id)
}
