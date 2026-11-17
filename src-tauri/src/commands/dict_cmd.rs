use tauri::{AppHandle, Manager, State};

use crate::db::dict_pack_repo::{self, DictPackInfo, DictPackState};
use crate::db::{DbState, dict_repo};
use crate::dictionary;
use crate::error::{AppError, AppResult};
use crate::models::DictionaryEntry;

#[tauri::command]
pub async fn lookup_word(
    db: State<'_, DbState>,
    dict_pack: State<'_, DictPackState>,
    word: String,
) -> AppResult<DictionaryEntry> {
    {
        let conn =
            db.0.lock()
                .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
        if let Some(local) = dict_repo::find_exact(&conn, &word)? {
            return Ok(local);
        }
    }

    {
        let pack_guard = dict_pack
            .0
            .lock()
            .map_err(|_| AppError::Internal("Dictionary pack lock poisoned".into()))?;
        if let Some(pack_conn) = pack_guard.as_ref()
            && let Some(pack_entry) = dict_pack_repo::find_exact_or_lemma(pack_conn, &word)?
        {
            return Ok(pack_entry);
        }
    }

    let online = dictionary::fetch_online_entry(&word).await?;
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    dict_repo::upsert_entry(&conn, &online)
}

#[tauri::command]
pub fn search_dictionary(
    db: State<'_, DbState>,
    dict_pack: State<'_, DictPackState>,
    query: String,
    limit: Option<i64>,
) -> AppResult<Vec<DictionaryEntry>> {
    let cap = limit.unwrap_or(20).clamp(1, 50);
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    if query.trim().is_empty() {
        return dict_repo::list_entries(&conn, limit.unwrap_or(30).clamp(1, 100), 0);
    }

    let mut results = dict_repo::search_fts(&conn, &query, cap)?;
    let pack_guard = dict_pack
        .0
        .lock()
        .map_err(|_| AppError::Internal("Dictionary pack lock poisoned".into()))?;
    if let Some(pack_conn) = pack_guard.as_ref()
        && (results.len() as i64) < cap
    {
        let remaining = cap - results.len() as i64;
        let pack_hits = dict_pack_repo::search_fts(pack_conn, &query, remaining)?;
        for hit in pack_hits {
            if !results
                .iter()
                .any(|r| r.word.eq_ignore_ascii_case(&hit.word))
            {
                results.push(hit);
            }
        }
    }
    Ok(results)
}

#[tauri::command]
pub fn get_dictionary_count(
    db: State<'_, DbState>,
    dict_pack: State<'_, DictPackState>,
) -> AppResult<i64> {
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    let mut total = dict_repo::count_entries(&conn)?;
    let pack_guard = dict_pack
        .0
        .lock()
        .map_err(|_| AppError::Internal("Dictionary pack lock poisoned".into()))?;
    if let Some(pack_conn) = pack_guard.as_ref() {
        total += dict_pack_repo::count_entries(pack_conn)?;
    }
    Ok(total)
}

#[tauri::command]
pub fn import_dictionary_tsv(db: State<'_, DbState>, content: String) -> AppResult<i64> {
    let mut conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    dict_repo::bulk_import_tsv(&mut conn, &content)
}

#[tauri::command]
pub async fn fetch_collocations(word: String) -> AppResult<Vec<String>> {
    dictionary::fetch_collocations(&word).await
}

#[tauri::command]
pub fn get_dict_pack_info(dict_pack: State<'_, DictPackState>) -> AppResult<DictPackInfo> {
    let guard = dict_pack
        .0
        .lock()
        .map_err(|_| AppError::Internal("Dictionary pack lock poisoned".into()))?;
    dict_pack_repo::read_pack_info(guard.as_ref())
}

#[tauri::command]
pub fn begin_dict_pack_install(app: AppHandle) -> AppResult<()> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pack_path = app_dir.join("dict.db");
    dict_pack_repo::begin_pack_install(&pack_path)
}

#[tauri::command]
pub fn append_dict_pack_chunk(app: AppHandle, request: tauri::ipc::Request<'_>) -> AppResult<()> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pack_path = app_dir.join("dict.db");
    match request.body() {
        tauri::ipc::InvokeBody::Raw(bytes) => dict_pack_repo::append_pack_chunk(&pack_path, bytes),
        tauri::ipc::InvokeBody::Json(_) => Err(AppError::Validation(
            "Expected raw binary chunk for dict pack upload".into(),
        )),
    }
}

#[tauri::command]
pub fn finish_dict_pack_install(
    app: AppHandle,
    db: State<'_, DbState>,
    dict_pack: State<'_, DictPackState>,
) -> AppResult<DictPackInfo> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pack_path = app_dir.join("dict.db");
    let info = dict_pack_repo::finish_pack_install(&dict_pack, &pack_path)?;
    // Clean up any legacy bulk-imported pack rows from user DB when switching to dict.db pack
    if let Ok(conn) = db.0.lock() {
        let _ = conn.execute(
            "DELETE FROM dictionary_entries WHERE source IN ('anhviet109k', 'ipa-dict', 'fvdp', 'ipa-tsv')",
            [],
        );
    }
    Ok(info)
}

#[tauri::command]
pub fn remove_dict_pack(app: AppHandle, dict_pack: State<'_, DictPackState>) -> AppResult<()> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pack_path = app_dir.join("dict.db");
    dict_pack_repo::remove_pack(&dict_pack, &pack_path)
}
