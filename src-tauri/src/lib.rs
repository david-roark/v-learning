mod commands;
mod db;
mod error;
mod models;
mod srs;

use tauri::Manager;

use commands::{deck_cmd, note_cmd, study_cmd};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db_state = db::init_db(app.handle())?;
            app.manage(db_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            deck_cmd::list_decks,
            deck_cmd::create_deck,
            deck_cmd::delete_deck,
            note_cmd::list_notes,
            note_cmd::create_note,
            note_cmd::update_note,
            note_cmd::delete_note,
            study_cmd::get_study_queue,
            study_cmd::submit_review,
            study_cmd::get_daily_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
