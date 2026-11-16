use tauri::{AppHandle, State};

use crate::db::{DbState, dict_repo};
use crate::dictionary::{self, importer};
use crate::error::{AppError, AppResult};
use crate::models::DictionaryEntry;

#[tauri::command]
pub async fn lookup_word(db: State<'_, DbState>, word: String) -> AppResult<DictionaryEntry> {
    {
        let conn =
            db.0.lock()
                .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
        if let Some(local) = dict_repo::find_exact(&conn, &word)? {
            return Ok(local);
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
    query: String,
    limit: Option<i64>,
) -> AppResult<Vec<DictionaryEntry>> {
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    if query.trim().is_empty() {
        return dict_repo::list_entries(&conn, limit.unwrap_or(30).clamp(1, 100), 0);
    }
    dict_repo::search_fts(&conn, &query, limit.unwrap_or(20).clamp(1, 50))
}

#[tauri::command]
pub fn get_dictionary_count(db: State<'_, DbState>) -> AppResult<i64> {
    let conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    dict_repo::count_entries(&conn)
}

#[tauri::command]
pub fn import_dictionary_tsv(db: State<'_, DbState>, content: String) -> AppResult<i64> {
    let mut conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;
    dict_repo::bulk_import_tsv(&mut conn, &content)
}

#[tauri::command]
pub fn list_dictionary_packs() -> Vec<importer::DictionaryPack> {
    importer::AVAILABLE_PACKS.to_vec()
}

#[tauri::command]
pub async fn fetch_collocations(word: String) -> AppResult<Vec<String>> {
    dictionary::fetch_collocations(&word).await
}

#[tauri::command]
pub async fn download_and_import_pack(
    app: AppHandle,
    db: State<'_, DbState>,
    pack_id: String,
) -> AppResult<importer::ImportReport> {
    let pack = importer::find_pack(&pack_id)?;
    importer::emit_progress(
        Some(&app),
        "downloading",
        0,
        0,
        &format!("Đang tải {} ({})...", pack.name, pack.size_hint),
    );

    let resp = reqwest::get(pack.url)
        .await
        .map_err(|e| AppError::Internal(format!("Lỗi tải gói từ điển: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::Internal(format!(
            "Không thể tải gói từ điển (HTTP {})",
            resp.status()
        )));
    }

    let text = resp
        .text()
        .await
        .map_err(|e| AppError::Internal(format!("Lỗi đọc nội dung gói từ điển: {e}")))?;

    importer::emit_progress(
        Some(&app),
        "importing",
        0,
        0,
        "Đang phân tích và lưu vào SQLite...",
    );

    let mut conn =
        db.0.lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".into()))?;

    match pack.format {
        "fvdp" => importer::import_fvdp_text(&mut conn, &text, Some(&app)),
        "ipa-tsv" => importer::import_ipa_tsv(&mut conn, &text, Some(&app)),
        _ => Err(AppError::Validation("Định dạng gói không hợp lệ".into())),
    }
}
