mod ai;
mod anki;
mod commands;
mod db;
mod desktop;
mod dictionary;
mod error;
mod media;
mod models;
mod srs;

use tauri::Manager;

use commands::{ai_cmd, anki_cmd, deck_cmd, dict_cmd, media_cmd, note_cmd, study_cmd};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let db_state = db::init_db(app.handle())?;
            app.manage(db_state);
            desktop::setup_desktop_features(app.handle())?;
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
            dict_cmd::lookup_word,
            dict_cmd::search_dictionary,
            dict_cmd::get_dictionary_count,
            dict_cmd::import_dictionary_tsv,
            dict_cmd::list_dictionary_packs,
            dict_cmd::download_and_import_pack,
            media_cmd::fetch_audio,
            ai_cmd::get_ai_settings,
            ai_cmd::save_ai_settings,
            ai_cmd::list_ai_models,
            ai_cmd::evaluate_ai_practice,
            anki_cmd::import_apkg,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
