mod ai;
mod anki;
mod commands;
mod db;
mod desktop;
mod dictionary;
mod error;
mod media;
mod models;
mod reader;
mod srs;
mod sync;

use tauri::Manager;

use commands::{
    ai_cmd, analytics_cmd, anki_cmd, deck_cmd, dict_cmd, media_cmd, note_cmd, reader_cmd,
    study_cmd, sync_cmd,
};

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
            study_cmd::undo_review,
            study_cmd::get_daily_stats,
            dict_cmd::lookup_word,
            dict_cmd::search_dictionary,
            dict_cmd::get_dictionary_count,
            dict_cmd::import_dictionary_tsv,
            dict_cmd::list_dictionary_packs,
            dict_cmd::download_and_import_pack,
            dict_cmd::fetch_collocations,
            media_cmd::fetch_audio,
            desktop::read_clipboard_text,
            ai_cmd::get_ai_settings,
            ai_cmd::save_ai_settings,
            ai_cmd::list_ai_models,
            ai_cmd::evaluate_ai_practice,
            anki_cmd::import_apkg,
            analytics_cmd::get_analytics_summary,
            sync_cmd::export_backup,
            sync_cmd::import_backup,
            sync_cmd::get_webdav_settings,
            sync_cmd::save_webdav_settings,
            sync_cmd::sync_webdav_push,
            sync_cmd::sync_webdav_pull,
            reader_cmd::parse_epub,
            reader_cmd::load_reader_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
