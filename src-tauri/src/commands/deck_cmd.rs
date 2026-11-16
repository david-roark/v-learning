use tauri::State;

use crate::db::{DbState, deck_repo};
use crate::error::{AppError, AppResult};
use crate::models::{CreateDeckInput, Deck, DeckWithStats};

#[tauri::command]
pub fn list_decks(db: State<'_, DbState>) -> AppResult<Vec<DeckWithStats>> {
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    deck_repo::list_decks(&conn)
}

#[tauri::command]
pub fn create_deck(db: State<'_, DbState>, input: CreateDeckInput) -> AppResult<Deck> {
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    deck_repo::create_deck(&conn, input)
}

#[tauri::command]
pub fn delete_deck(db: State<'_, DbState>, deck_id: String) -> AppResult<()> {
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    deck_repo::delete_deck(&conn, &deck_id)
}
