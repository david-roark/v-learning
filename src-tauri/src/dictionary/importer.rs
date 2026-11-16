use std::collections::HashSet;

use rusqlite::{Connection, params};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::db::dict_repo;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub inserted: i64,
    pub updated: i64,
    pub skipped: i64,
    pub total_after: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportProgress {
    pub stage: String,
    pub current: i64,
    pub total: i64,
    pub message: String,
}

pub fn emit_progress(app: Option<&AppHandle>, stage: &str, current: i64, total: i64, message: &str) {
    if let Some(handle) = app {
        let _ = handle.emit(
            "dict-import-progress",
            ImportProgress {
                stage: stage.to_string(),
                current,
                total,
                message: message.to_string(),
            },
        );
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryPack {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub size_hint: &'static str,
    pub url: &'static str,
    pub format: &'static str,
}

pub const AVAILABLE_PACKS: &[DictionaryPack] = &[
    DictionaryPack {
        id: "anhviet-109k",
        name: "Từ điển Anh–Việt (Hồ Ngọc Đức ~109.000 từ)",
        description: "Bộ từ điển mở phổ biến nhất: nghĩa tiếng Việt, loại từ, IPA và ví dụ.",
        size_hint: "~18 MB",
        url: "https://raw.githubusercontent.com/yenthanh132/avdict-database-sqlite-converter/master/anhviet109K.txt",
        format: "fvdp",
    },
    DictionaryPack {
        id: "ipa-en-us",
        name: "Phiên âm IPA Tiếng Anh Mỹ (~125.000 từ)",
        description: "Bổ sung phiên âm IPA chuẩn US cho hầu hết từ vựng tiếng Anh.",
        size_hint: "~3 MB",
        url: "https://raw.githubusercontent.com/open-dict-data/ipa-dict/master/data/en_US.txt",
        format: "ipa-tsv",
    },
];

pub fn find_pack(pack_id: &str) -> AppResult<&'static DictionaryPack> {
    AVAILABLE_PACKS
        .iter()
        .find(|p| p.id == pack_id)
        .ok_or_else(|| AppError::NotFound(format!("Không tìm thấy gói từ điển: {pack_id}")))
}

fn parse_header(line: &str) -> (String, Option<String>) {
    let raw = line.trim_start_matches('\u{feff}').trim_start_matches('@').trim();
    if let Some(slash_idx) = raw.find('/') {
        let word = raw[..slash_idx].trim().to_string();
        let ipa = raw[slash_idx..].trim().to_string();
        (word, if ipa.is_empty() { None } else { Some(ipa) })
    } else {
        (raw.to_string(), None)
    }
}

pub fn import_fvdp_text(
    conn: &mut Connection,
    content: &str,
    app: Option<&AppHandle>,
) -> AppResult<ImportReport> {
    let total_before = dict_repo::count_entries(conn)?;
    let total_entries = content.lines().filter(|l| l.trim_start_matches('\u{feff}').trim_start().starts_with('@')).count() as i64;

    let tx = conn.transaction()?;
    let mut processed = 0_i64;
    let mut skipped = 0_i64;
    let mut seen = HashSet::new();

    {
        let mut stmt = tx.prepare(
            "INSERT INTO dictionary_entries (word, ipa, pos, meaning_vi, example_sentence, source)
             VALUES (?1, ?2, ?3, ?4, ?5, 'anhviet109k')
             ON CONFLICT(word) DO UPDATE SET
               ipa = COALESCE(excluded.ipa, dictionary_entries.ipa),
               pos = COALESCE(excluded.pos, dictionary_entries.pos),
               meaning_vi = COALESCE(excluded.meaning_vi, dictionary_entries.meaning_vi),
               example_sentence = COALESCE(excluded.example_sentence, dictionary_entries.example_sentence)",
        )?;

        let mut cur_word = String::new();
        let mut cur_ipa: Option<String> = None;
        let mut cur_pos: Option<String> = None;
        let mut meanings: Vec<String> = Vec::new();
        let mut cur_example: Option<String> = None;

        let flush = |stmt: &mut rusqlite::Statement<'_>,
                     seen: &mut HashSet<String>,
                     skipped: &mut i64,
                     word: &str,
                     ipa: &Option<String>,
                     pos: &Option<String>,
                     meanings: &[String],
                     example: &Option<String>|
         -> rusqlite::Result<bool> {
            if word.is_empty() {
                return Ok(false);
            }
            let key = word.to_lowercase();
            if !seen.insert(key) {
                *skipped += 1;
                return Ok(false);
            }
            let meaning_vi = if meanings.is_empty() {
                None
            } else {
                Some(meanings.join("; "))
            };
            stmt.execute(params![word, ipa, pos, meaning_vi, example])?;
            Ok(true)
        };

        for raw_line in content.lines() {
            let line = raw_line.trim_start_matches('\u{feff}').trim();
            if line.starts_with('@') {
                if flush(
                    &mut stmt,
                    &mut seen,
                    &mut skipped,
                    &cur_word,
                    &cur_ipa,
                    &cur_pos,
                    &meanings,
                    &cur_example,
                )? {
                    processed += 1;
                    if processed % 2500 == 0 {
                        emit_progress(
                            app,
                            "importing",
                            processed,
                            total_entries,
                            &format!("Đang import {processed} / {total_entries} từ..."),
                        );
                    }
                }
                let (w, ipa) = parse_header(line);
                cur_word = w;
                cur_ipa = ipa;
                cur_pos = None;
                meanings.clear();
                cur_example = None;
            } else if line.starts_with('*') && cur_pos.is_none() {
                let pos = line.trim_start_matches('*').trim();
                if !pos.is_empty() {
                    cur_pos = Some(pos.to_string());
                }
            } else if line.starts_with('-') && meanings.len() < 4 {
                let m = line.trim_start_matches('-').trim();
                if !m.is_empty() {
                    meanings.push(m.to_string());
                }
            } else if line.starts_with('=') && cur_example.is_none() {
                let ex = line.trim_start_matches('=').split('+').next().unwrap_or("").trim();
                if !ex.is_empty() {
                    cur_example = Some(ex.to_string());
                }
            }
        }

        if flush(
            &mut stmt,
            &mut seen,
            &mut skipped,
            &cur_word,
            &cur_ipa,
            &cur_pos,
            &meanings,
            &cur_example,
        )? {
            processed += 1;
        }
    }

    tx.commit()?;
    let total_after = dict_repo::count_entries(conn)?;
    let inserted = (total_after - total_before).max(0);
    let updated = (processed - inserted).max(0);
    emit_progress(app, "done", processed, total_entries, "Hoàn tất import!");

    Ok(ImportReport {
        inserted,
        updated,
        skipped,
        total_after,
    })
}

pub fn import_ipa_tsv(
    conn: &mut Connection,
    content: &str,
    app: Option<&AppHandle>,
) -> AppResult<ImportReport> {
    let total_before = dict_repo::count_entries(conn)?;
    let total_lines = content.lines().filter(|l| !l.trim().is_empty()).count() as i64;

    let tx = conn.transaction()?;
    let mut processed = 0_i64;
    let mut skipped = 0_i64;
    let mut seen = HashSet::new();

    {
        let mut stmt = tx.prepare(
            "INSERT INTO dictionary_entries (word, ipa, source)
             VALUES (?1, ?2, 'ipa-dict')
             ON CONFLICT(word) DO UPDATE SET
               ipa = COALESCE(excluded.ipa, dictionary_entries.ipa)",
        )?;

        for line in content.lines() {
            let mut parts = line.split('\t');
            let word = parts.next().unwrap_or("").trim();
            let ipa = parts.next().unwrap_or("").split(',').next().unwrap_or("").trim();
            if word.is_empty() || ipa.is_empty() {
                continue;
            }
            if !seen.insert(word.to_lowercase()) {
                skipped += 1;
                continue;
            }
            stmt.execute(params![word, ipa])?;
            processed += 1;
            if processed % 5000 == 0 {
                emit_progress(
                    app,
                    "importing",
                    processed,
                    total_lines,
                    &format!("Đang import IPA {processed} / {total_lines} từ..."),
                );
            }
        }
    }

    tx.commit()?;
    let total_after = dict_repo::count_entries(conn)?;
    let inserted = (total_after - total_before).max(0);
    let updated = (processed - inserted).max(0);
    emit_progress(app, "done", processed, total_lines, "Hoàn tất import IPA!");

    Ok(ImportReport {
        inserted,
        updated,
        skipped,
        total_after,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{dict_repo, open_in_memory_for_test};

    #[test]
    fn import_fvdp_and_ipa_tsv_dedup_and_report() {
        let mut conn = open_in_memory_for_test();

        let fvdp = "\u{feff}@serendipity /ˌser.ənˈdɪp.ə.ti/\n*  danh từ\n- sự tình cờ may mắn\n=pure serendipity+ hoàn toàn tình cờ\n@serendipity /ˌser.ənˈdɪp.ə.ti/\n- trùng lặp\n";
        let r1 = import_fvdp_text(&mut conn, fvdp, None).expect("import fvdp");
        assert_eq!(r1.inserted, 1);
        assert_eq!(r1.skipped, 1);

        let entry = dict_repo::find_exact(&conn, "serendipity")
            .expect("find")
            .expect("exists");
        assert_eq!(entry.meaning_vi.as_deref(), Some("sự tình cờ may mắn"));

        let ipa_tsv = "serendipity\t/ˌsɛɹənˈdɪpəti/\nephemeral\t/ɪˈfɛmɚəl/\n";
        let r2 = import_ipa_tsv(&mut conn, ipa_tsv, None).expect("import ipa");
        assert_eq!(r2.inserted, 0); // ephemeral already in starter dict, serendipity already imported
        assert_eq!(r2.updated, 2);
    }
}
