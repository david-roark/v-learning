pub mod analytics_repo;
pub mod card_repo;
pub mod deck_repo;
pub mod dict_repo;
pub mod migrations;
pub mod note_repo;
pub mod settings_repo;

use std::fs;
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};

pub struct DbState(pub Mutex<Connection>);

pub fn init_db(app: &AppHandle) -> AppResult<DbState> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    fs::create_dir_all(&app_dir).map_err(|e| AppError::Internal(e.to_string()))?;

    let db_path = app_dir.join("vlearning.db");
    let conn = Connection::open(db_path)?;

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;
         PRAGMA synchronous = NORMAL;",
    )?;

    migrations::run_migrations(&conn)?;
    Ok(DbState(Mutex::new(conn)))
}

#[cfg(test)]
pub fn open_in_memory_for_test() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory sqlite");
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("enable foreign keys");
    migrations::run_migrations(&conn).expect("run migrations");
    conn
}
