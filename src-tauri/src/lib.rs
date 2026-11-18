mod ai;
mod anki;
mod commands;
mod db;
mod desktop;
mod dictionary;
mod error;
mod http;
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
    let builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_global_shortcut::Builder::new().build());

    builder
        .setup(|app| {
            let db_state = db::init_db(app.handle())?;
            app.manage(db_state);
            let dict_pack_state = db::init_dict_pack_db(app.handle())?;
            app.manage(dict_pack_state);
            app.manage(sync::gdrive::PendingGoogleOAuth::default());
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
            dict_cmd::get_dict_pack_info,
            dict_cmd::begin_dict_pack_install,
            dict_cmd::append_dict_pack_chunk,
            dict_cmd::finish_dict_pack_install,
            dict_cmd::remove_dict_pack,
            dict_cmd::fetch_collocations,
            media_cmd::fetch_audio,
            media_cmd::preview_tts_provider,
            media_cmd::speak_system_tts,
            media_cmd::get_tts_settings,
            media_cmd::save_tts_settings,
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
            sync_cmd::google_drive_status,
            sync_cmd::google_drive_start_login,
            sync_cmd::google_drive_complete_login,
            sync_cmd::google_drive_cancel_login,
            sync_cmd::google_drive_logout,
            sync_cmd::sync_google_drive_push,
            sync_cmd::sync_google_drive_pull,
            sync_cmd::google_drive_backup_info,
            reader_cmd::parse_epub,
            reader_cmd::load_reader_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
pub(crate) mod http_mock {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    pub struct CapturedRequest {
        pub request_line: String,
        pub headers: String,
        pub body: String,
    }

    pub fn spawn_one_shot(
        status: &str,
        content_type: &str,
        response_body: Vec<u8>,
    ) -> (String, mpsc::Receiver<CapturedRequest>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock port");
        let addr = listener.local_addr().expect("local addr");
        let (tx, rx) = mpsc::channel();
        let status_owned = status.to_string();
        let ct_owned = content_type.to_string();

        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = Vec::new();
                let mut tmp = [0_u8; 4096];
                let mut header_end = None;
                while let Ok(n) = stream.read(&mut tmp) {
                    if n == 0 {
                        break;
                    }
                    buf.extend_from_slice(&tmp[..n]);
                    if header_end.is_none()
                        && let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n")
                    {
                        header_end = Some(pos + 4);
                        let headers_str = String::from_utf8_lossy(&buf[..pos]);
                        let content_len = headers_str
                            .lines()
                            .find_map(|l| {
                                let (k, v) = l.split_once(':')?;
                                if k.eq_ignore_ascii_case("content-length") {
                                    v.trim().parse::<usize>().ok()
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(0);
                        if buf.len() >= pos + 4 + content_len {
                            break;
                        }
                    } else if let Some(h_end) = header_end {
                        let headers_str = String::from_utf8_lossy(&buf[..h_end]);
                        let content_len = headers_str
                            .lines()
                            .find_map(|l| {
                                let (k, v) = l.split_once(':')?;
                                if k.eq_ignore_ascii_case("content-length") {
                                    v.trim().parse::<usize>().ok()
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(0);
                        if buf.len() >= h_end + content_len {
                            break;
                        }
                    }
                }

                let split = header_end.unwrap_or(buf.len());
                let head = String::from_utf8_lossy(&buf[..split]).into_owned();
                let request_line = head.lines().next().unwrap_or("").to_string();
                let body = String::from_utf8_lossy(&buf[split..]).into_owned();
                let _ = tx.send(CapturedRequest {
                    request_line,
                    headers: head,
                    body,
                });

                let resp_head = format!(
                    "HTTP/1.1 {status_owned}\r\nContent-Type: {ct_owned}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    response_body.len()
                );
                let _ = stream.write_all(resp_head.as_bytes());
                let _ = stream.write_all(&response_body);
                let _ = stream.flush();
            }
        });

        (format!("http://{addr}"), rx)
    }
}
