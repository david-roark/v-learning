use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};
use crate::models::LocalBackupEntry;
use crate::sync::export_backup_json;

pub const MAX_LOCAL_SNAPSHOTS: usize = 10;

pub fn backup_dir(app: &AppHandle) -> AppResult<PathBuf> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(app_dir.join("backups"))
}

fn db_has_user_data(conn: &Connection) -> AppResult<bool> {
    let has_data: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM decks) OR EXISTS(SELECT 1 FROM notes) OR EXISTS(SELECT 1 FROM review_logs)",
        [],
        |r| r.get(0),
    )?;
    Ok(has_data)
}

fn parse_snapshot_meta(filename: &str) -> Option<(String, i64)> {
    let stem = filename.strip_prefix("vlearning-")?.strip_suffix(".json")?;
    let (reason, ts_str) = stem.rsplit_once('-')?;
    let ts = ts_str.parse::<i64>().ok()?;
    Some((reason.to_string(), ts))
}

pub fn create_snapshot_in_dir(
    dir: &Path,
    conn: &Connection,
    reason: &str,
    max_keep: usize,
) -> AppResult<Option<LocalBackupEntry>> {
    if !db_has_user_data(conn)? {
        return Ok(None);
    }

    fs::create_dir_all(dir)
        .map_err(|e| AppError::Internal(format!("Không tạo được thư mục backups: {e}")))?;

    let clean_reason: String = reason
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let ts = Utc::now().timestamp_millis();
    let filename = format!("vlearning-{clean_reason}-{ts}.json");
    let path = dir.join(&filename);
    let json = export_backup_json(conn)?;
    let size_bytes = u64::try_from(json.len()).unwrap_or(0);

    fs::write(&path, json.as_bytes())
        .map_err(|e| AppError::Internal(format!("Không ghi được snapshot backup: {e}")))?;

    prune_snapshots_in_dir(dir, max_keep)?;

    Ok(Some(LocalBackupEntry {
        filename,
        created_at: ts,
        size_bytes,
        reason: clean_reason,
    }))
}

fn prune_snapshots_in_dir(dir: &Path, max_keep: usize) -> AppResult<()> {
    let mut entries = list_snapshots_in_dir(dir)?;
    if entries.len() <= max_keep {
        return Ok(());
    }
    for old in entries.drain(max_keep..) {
        let _ = fs::remove_file(dir.join(&old.filename));
    }
    Ok(())
}

pub fn list_snapshots_in_dir(dir: &Path) -> AppResult<Vec<LocalBackupEntry>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut list = Vec::new();
    let read_dir = fs::read_dir(dir)
        .map_err(|e| AppError::Internal(format!("Không đọc được thư mục backups: {e}")))?;

    for item in read_dir.flatten() {
        let path = item.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some((reason, created_at)) = parse_snapshot_meta(name) else {
            continue;
        };
        let size_bytes = item.metadata().map(|m| m.len()).unwrap_or(0);
        list.push(LocalBackupEntry {
            filename: name.to_string(),
            created_at,
            size_bytes,
            reason,
        });
    }

    list.sort_by_key(|b| std::cmp::Reverse(b.created_at));
    Ok(list)
}

pub fn read_snapshot_in_dir(dir: &Path, filename: &str) -> AppResult<String> {
    if filename.contains('/')
        || filename.contains('\\')
        || filename.contains("..")
        || parse_snapshot_meta(filename).is_none()
    {
        return Err(AppError::Validation(
            "Tên file snapshot không hợp lệ.".into(),
        ));
    }

    let path = dir.join(filename);
    if !path.is_file() {
        return Err(AppError::NotFound(
            "Không tìm thấy bản sao lưu trên máy.".into(),
        ));
    }

    fs::read_to_string(&path)
        .map_err(|e| AppError::Internal(format!("Không đọc được snapshot: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{deck_repo, note_repo, open_in_memory_for_test};
    use crate::models::{CreateDeckInput, CreateNoteInput};

    fn temp_backup_dir(suffix: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "vlearning_snap_test_{}_{suffix}",
            uuid::Uuid::new_v4().simple()
        ));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn snapshot_skips_empty_db_and_creates_lists_reads_and_prunes() {
        let dir = temp_backup_dir("roundtrip");
        let mut conn = open_in_memory_for_test();

        // Empty DB -> None
        let empty = create_snapshot_in_dir(&dir, &conn, "auto", 3).expect("empty ok");
        assert!(empty.is_none());
        assert!(list_snapshots_in_dir(&dir).expect("list").is_empty());

        // Add 1 deck + 1 note
        let deck = deck_repo::create_deck(
            &conn,
            CreateDeckInput {
                name: "Test".into(),
                description: None,
                new_cards_per_day: None,
                desired_retention: None,
            },
        )
        .expect("deck");
        note_repo::create_note(
            &mut conn,
            CreateNoteInput {
                deck_id: deck.id,
                note_type: "vocab".into(),
                word: "resilient".into(),
                ipa: None,
                meaning: "kiên cường".into(),
                example_sentence: None,
                collocations: None,
                tags: None,
            },
        )
        .expect("note");

        // Create 4 snapshots with max_keep=2 -> should keep 2 newest
        for i in 0..4 {
            let entry = create_snapshot_in_dir(&dir, &conn, "pre-restore", 2)
                .expect("create")
                .expect("some");
            assert_eq!(entry.reason, "pre-restore");
            if i < 3 {
                std::thread::sleep(std::time::Duration::from_millis(3));
            }
        }

        let listed = list_snapshots_in_dir(&dir).expect("list");
        assert_eq!(listed.len(), 2);
        assert!(listed[0].created_at >= listed[1].created_at);

        let content = read_snapshot_in_dir(&dir, &listed[0].filename).expect("read");
        assert!(content.contains("resilient"));

        // Path traversal rejected
        assert!(read_snapshot_in_dir(&dir, "../etc/passwd").is_err());

        let _ = fs::remove_dir_all(&dir);
    }
}
