pub mod gdrive;
pub mod snapshots;

use std::collections::HashMap;
use std::io::{Read, Write};

use chrono::Utc;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::db::note_repo::row_to_note;
use crate::error::{AppError, AppResult};
use crate::models::{BackupRestoreReport, Card, Deck, Note, WebDavSettings};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReviewLogRow {
    id: String,
    card_id: String,
    rating: i64,
    state: i64,
    elapsed_days: i64,
    scheduled_days: i64,
    duration_ms: Option<i64>,
    reviewed_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupPayload {
    version: i64,
    exported_at: i64,
    decks: Vec<Deck>,
    notes: Vec<Note>,
    cards: Vec<Card>,
    review_logs: Vec<ReviewLogRow>,
}

fn build_backup_payload(conn: &Connection) -> AppResult<BackupPayload> {
    let decks = {
        let mut stmt = conn.prepare(
            "SELECT id, name, description, new_cards_per_day, desired_retention, created_at FROM decks",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(Deck {
                id: r.get(0)?,
                name: r.get(1)?,
                description: r.get(2)?,
                new_cards_per_day: r.get(3)?,
                desired_retention: r.get(4)?,
                created_at: r.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    let notes = {
        let mut stmt = conn.prepare(
            "SELECT id, deck_id, note_type, word, ipa, meaning, example_sentence,
                    collocations, tags, created_at, updated_at FROM notes",
        )?;
        let rows = stmt.query_map([], |r| row_to_note(r, 0))?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    let cards = {
        let mut stmt = conn.prepare(
            "SELECT id, note_id, card_template, state, due, stability, difficulty,
                    elapsed_days, scheduled_days, reps, lapses, last_review FROM cards",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(Card {
                id: r.get(0)?,
                note_id: r.get(1)?,
                card_template: r.get(2)?,
                state: r.get(3)?,
                due: r.get(4)?,
                stability: r.get(5)?,
                difficulty: r.get(6)?,
                elapsed_days: r.get(7)?,
                scheduled_days: r.get(8)?,
                reps: r.get(9)?,
                lapses: r.get(10)?,
                last_review: r.get(11)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    let review_logs = {
        let mut stmt = conn.prepare(
            "SELECT id, card_id, rating, state, elapsed_days, scheduled_days, duration_ms, reviewed_at FROM review_logs",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(ReviewLogRow {
                id: r.get(0)?,
                card_id: r.get(1)?,
                rating: r.get(2)?,
                state: r.get(3)?,
                elapsed_days: r.get(4)?,
                scheduled_days: r.get(5)?,
                duration_ms: r.get(6)?,
                reviewed_at: r.get(7)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    Ok(BackupPayload {
        version: 1,
        exported_at: Utc::now().timestamp_millis(),
        decks,
        notes,
        cards,
        review_logs,
    })
}

pub fn export_backup_json(conn: &Connection) -> AppResult<String> {
    let payload = build_backup_payload(conn)?;
    serde_json::to_string_pretty(&payload)
        .map_err(|e| AppError::Internal(format!("Lỗi tạo JSON backup: {e}")))
}

pub fn export_backup_gzip(conn: &Connection) -> AppResult<Vec<u8>> {
    let payload = build_backup_payload(conn)?;
    let compact = serde_json::to_vec(&payload)
        .map_err(|e| AppError::Internal(format!("Lỗi tạo JSON backup: {e}")))?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(&compact)
        .map_err(|e| AppError::Internal(format!("Lỗi nén backup gzip: {e}")))?;
    encoder
        .finish()
        .map_err(|e| AppError::Internal(format!("Lỗi hoàn tất nén gzip: {e}")))
}

fn decode_backup_bytes(data: &[u8]) -> AppResult<BackupPayload> {
    let json_str = if data.starts_with(&[0x1f, 0x8b]) {
        let mut decoder = GzDecoder::new(data);
        let mut s = String::new();
        decoder
            .read_to_string(&mut s)
            .map_err(|e| AppError::Validation(format!("File backup gzip không hợp lệ: {e}")))?;
        s
    } else {
        std::str::from_utf8(data)
            .map_err(|e| AppError::Validation(format!("File backup UTF-8 không hợp lệ: {e}")))?
            .to_string()
    };
    serde_json::from_str(&json_str)
        .map_err(|e| AppError::Validation(format!("File backup JSON không hợp lệ: {e}")))
}

pub fn import_backup_bytes(conn: &mut Connection, data: &[u8]) -> AppResult<BackupRestoreReport> {
    let payload = decode_backup_bytes(data)?;
    apply_backup_payload(conn, &payload)
}

pub fn import_backup_json(conn: &mut Connection, json_str: &str) -> AppResult<BackupRestoreReport> {
    let payload: BackupPayload = serde_json::from_str(json_str)
        .map_err(|e| AppError::Validation(format!("File backup JSON không hợp lệ: {e}")))?;
    apply_backup_payload(conn, &payload)
}

pub fn merge_backup_bytes(conn: &mut Connection, data: &[u8]) -> AppResult<BackupRestoreReport> {
    let local = build_backup_payload(conn)?;
    let remote = decode_backup_bytes(data)?;

    let mut decks_map: HashMap<String, Deck> =
        local.decks.into_iter().map(|d| (d.id.clone(), d)).collect();
    for d in remote.decks {
        decks_map.entry(d.id.clone()).or_insert(d);
    }

    let mut notes_map: HashMap<String, Note> =
        local.notes.into_iter().map(|n| (n.id.clone(), n)).collect();
    for n in remote.notes {
        match notes_map.get(&n.id) {
            Some(existing) if existing.updated_at >= n.updated_at => {}
            _ => {
                notes_map.insert(n.id.clone(), n);
            }
        }
    }

    let mut cards_map: HashMap<String, Card> =
        local.cards.into_iter().map(|c| (c.id.clone(), c)).collect();
    for c in remote.cards {
        match cards_map.get(&c.id) {
            Some(existing)
                if (existing.last_review.unwrap_or(0), existing.reps)
                    >= (c.last_review.unwrap_or(0), c.reps) => {}
            _ => {
                cards_map.insert(c.id.clone(), c);
            }
        }
    }

    let mut logs_map: HashMap<String, ReviewLogRow> = local
        .review_logs
        .into_iter()
        .map(|l| (l.id.clone(), l))
        .collect();
    for l in remote.review_logs {
        logs_map.entry(l.id.clone()).or_insert(l);
    }

    let merged = BackupPayload {
        version: 1,
        exported_at: Utc::now().timestamp_millis(),
        decks: decks_map.into_values().collect(),
        notes: notes_map.into_values().collect(),
        cards: cards_map.into_values().collect(),
        review_logs: logs_map.into_values().collect(),
    };
    apply_backup_payload(conn, &merged)
}

fn apply_backup_payload(
    conn: &mut Connection,
    payload: &BackupPayload,
) -> AppResult<BackupRestoreReport> {
    let tx = conn.transaction()?;

    for d in &payload.decks {
        tx.execute(
            "INSERT INTO decks (id, name, description, new_cards_per_day, desired_retention, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
               name = excluded.name,
               description = excluded.description,
               new_cards_per_day = excluded.new_cards_per_day,
               desired_retention = excluded.desired_retention",
            params![d.id, d.name, d.description, d.new_cards_per_day, d.desired_retention, d.created_at],
        )?;
    }

    for n in &payload.notes {
        let collocs = serde_json::to_string(&n.collocations).unwrap_or_else(|_| "[]".into());
        let tags = serde_json::to_string(&n.tags).unwrap_or_else(|_| "[]".into());
        tx.execute(
            "INSERT INTO notes (id, deck_id, note_type, word, ipa, meaning, example_sentence, collocations, tags, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
               word = excluded.word, ipa = excluded.ipa, meaning = excluded.meaning,
               example_sentence = excluded.example_sentence, collocations = excluded.collocations,
               tags = excluded.tags, updated_at = excluded.updated_at",
            params![n.id, n.deck_id, n.note_type, n.word, n.ipa, n.meaning, n.example_sentence, collocs, tags, n.created_at, n.updated_at],
        )?;
    }

    for c in &payload.cards {
        tx.execute(
            "INSERT INTO cards (id, note_id, card_template, state, due, stability, difficulty, elapsed_days, scheduled_days, reps, lapses, last_review)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(id) DO UPDATE SET
               state = excluded.state, due = excluded.due, stability = excluded.stability,
               difficulty = excluded.difficulty, elapsed_days = excluded.elapsed_days,
               scheduled_days = excluded.scheduled_days, reps = excluded.reps,
               lapses = excluded.lapses, last_review = excluded.last_review",
            params![c.id, c.note_id, c.card_template, c.state, c.due, c.stability, c.difficulty, c.elapsed_days, c.scheduled_days, c.reps, c.lapses, c.last_review],
        )?;
    }

    let mut inserted_logs = 0_i64;
    for r in &payload.review_logs {
        let changed = tx.execute(
            "INSERT OR IGNORE INTO review_logs (id, card_id, rating, state, elapsed_days, scheduled_days, duration_ms, reviewed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![r.id, r.card_id, r.rating, r.state, r.elapsed_days, r.scheduled_days, r.duration_ms, r.reviewed_at],
        )?;
        inserted_logs += changed as i64;
    }

    tx.commit()?;

    Ok(BackupRestoreReport {
        decks: payload.decks.len() as i64,
        notes: payload.notes.len() as i64,
        cards: payload.cards.len() as i64,
        review_logs: inserted_logs,
    })
}

fn webdav_file_url(base_url: &str) -> AppResult<String> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "WebDAV URL không được để trống".into(),
        ));
    }
    if trimmed.ends_with(".json") {
        Ok(trimmed.to_string())
    } else {
        Ok(format!("{trimmed}/vlearning-backup.json"))
    }
}

pub async fn webdav_push(settings: &WebDavSettings, json_payload: String) -> AppResult<()> {
    let url = webdav_file_url(&settings.url)?;
    let client = crate::http::build_client(30)?;
    let mut req = client.put(&url).body(json_payload);
    if !settings.username.trim().is_empty() {
        req = req.basic_auth(settings.username.trim(), Some(settings.password.as_str()));
    }
    let resp = req
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Lỗi kết nối WebDAV: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::Internal(format!(
            "WebDAV Push thất bại (HTTP {})",
            resp.status()
        )));
    }
    Ok(())
}

pub async fn webdav_pull(settings: &WebDavSettings) -> AppResult<String> {
    let url = webdav_file_url(&settings.url)?;
    let client = crate::http::build_client(30)?;
    let mut req = client.get(&url);
    if !settings.username.trim().is_empty() {
        req = req.basic_auth(settings.username.trim(), Some(settings.password.as_str()));
    }
    let resp = req
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Lỗi kết nối WebDAV: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::Internal(format!(
            "WebDAV Pull thất bại (HTTP {})",
            resp.status()
        )));
    }
    resp.text()
        .await
        .map_err(|e| AppError::Internal(format!("Lỗi đọc dữ liệu WebDAV: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::deck_repo::{create_deck, list_decks};
    use crate::db::note_repo::create_note;
    use crate::db::open_in_memory_for_test;
    use crate::models::{CreateDeckInput, CreateNoteInput};

    #[test]
    fn backup_export_and_import_roundtrip() {
        let mut src = open_in_memory_for_test();
        let deck = create_deck(
            &src,
            CreateDeckInput {
                name: "Backup Deck".into(),
                description: None,
                new_cards_per_day: None,
                desired_retention: None,
            },
        )
        .expect("deck");

        create_note(
            &mut src,
            CreateNoteInput {
                deck_id: deck.id,
                note_type: "vocab".into(),
                word: "inevitable".into(),
                ipa: None,
                meaning: "không thể tránh khỏi".into(),
                example_sentence: None,
                collocations: None,
                tags: None,
            },
        )
        .expect("note");

        let json = export_backup_json(&src).expect("export");

        let mut dst = open_in_memory_for_test();
        let report = import_backup_json(&mut dst, &json).expect("import");
        assert_eq!(report.decks, 1);
        assert_eq!(report.notes, 1);
        assert_eq!(report.cards, 1);

        let decks = list_decks(&dst).expect("list");
        assert_eq!(decks[0].name, "Backup Deck");

        let reimport = import_backup_json(&mut dst, &json).expect("reimport");
        assert_eq!(reimport.review_logs, 0);

        assert!(import_backup_json(&mut dst, "{invalid json").is_err());
        assert!(webdav_file_url("   ").is_err());
        assert_eq!(
            webdav_file_url("https://dav.example.com/vlearning/").expect("dir url"),
            "https://dav.example.com/vlearning/vlearning-backup.json"
        );
        assert_eq!(
            webdav_file_url("https://dav.example.com/my-backup.json").expect("file url"),
            "https://dav.example.com/my-backup.json"
        );
    }

    #[test]
    fn backup_gzip_export_and_import_roundtrip() {
        let mut src = open_in_memory_for_test();
        let deck = create_deck(
            &src,
            CreateDeckInput {
                name: "GzipDeck".into(),
                description: Some("Compressed backup test".into()),
                new_cards_per_day: None,
                desired_retention: None,
            },
        )
        .expect("deck");
        for i in 0..10 {
            create_note(
                &mut src,
                CreateNoteInput {
                    deck_id: deck.id.clone(),
                    note_type: "vocab".into(),
                    word: format!("word_{i}"),
                    ipa: Some("/wɜːd/".into()),
                    meaning: "từ vựng mẫu để kiểm tra nén gzip".into(),
                    example_sentence: Some("This is an example sentence for gzip testing.".into()),
                    collocations: None,
                    tags: None,
                },
            )
            .expect("note");
        }

        let pretty = export_backup_json(&src).expect("pretty");
        let gzipped = export_backup_gzip(&src).expect("gzip");
        assert!(gzipped.starts_with(&[0x1f, 0x8b]));
        assert!(gzipped.len() < pretty.len());

        let mut dst = open_in_memory_for_test();
        let rep = import_backup_bytes(&mut dst, &gzipped).expect("import gzip");
        assert_eq!(rep.decks, 1);
        assert_eq!(rep.notes, 10);

        let mut dst2 = open_in_memory_for_test();
        let rep2 = import_backup_bytes(&mut dst2, pretty.as_bytes()).expect("import plain");
        assert_eq!(rep2.notes, 10);
    }

    #[test]
    fn merge_backup_bytes_keeps_freshest_notes_cards_and_unions_logs() {
        let mut dev_a = open_in_memory_for_test();
        let mut dev_b = open_in_memory_for_test();

        let deck_a = create_deck(
            &dev_a,
            CreateDeckInput {
                name: "DeckA".into(),
                description: None,
                new_cards_per_day: None,
                desired_retention: None,
            },
        )
        .expect("deck a");
        create_note(
            &mut dev_a,
            CreateNoteInput {
                deck_id: deck_a.id.clone(),
                note_type: "vocab".into(),
                word: "alpha".into(),
                ipa: None,
                meaning: "old_meaning".into(),
                example_sentence: None,
                collocations: None,
                tags: None,
            },
        )
        .expect("note alpha");

        // Clone A -> B, then edit alpha on B (newer) and add beta on A
        let snap_a = export_backup_gzip(&dev_a).expect("snap");
        import_backup_bytes(&mut dev_b, &snap_a).expect("seed b");

        let b_notes =
            crate::db::note_repo::list_notes_by_deck(&dev_b, &deck_a.id).expect("b notes");
        std::thread::sleep(std::time::Duration::from_millis(5));
        crate::db::note_repo::update_note(
            &dev_b,
            crate::models::UpdateNoteInput {
                id: b_notes[0].id.clone(),
                word: "alpha".into(),
                ipa: None,
                meaning: "newer_meaning_from_b".into(),
                example_sentence: None,
                collocations: None,
                tags: None,
            },
        )
        .expect("update alpha on b");

        create_note(
            &mut dev_a,
            CreateNoteInput {
                deck_id: deck_a.id.clone(),
                note_type: "vocab".into(),
                word: "beta".into(),
                ipa: None,
                meaning: "from_a".into(),
                example_sentence: None,
                collocations: None,
                tags: None,
            },
        )
        .expect("note beta on a");

        // Merge B's backup into A -> should keep beta from A AND newer alpha meaning from B
        let snap_b = export_backup_gzip(&dev_b).expect("snap b");
        let merged = merge_backup_bytes(&mut dev_a, &snap_b).expect("merge");
        assert_eq!(merged.notes, 2);

        let final_notes =
            crate::db::note_repo::list_notes_by_deck(&dev_a, &deck_a.id).expect("final");
        let alpha = final_notes
            .iter()
            .find(|n| n.word == "alpha")
            .expect("alpha");
        let beta = final_notes.iter().find(|n| n.word == "beta").expect("beta");
        assert_eq!(alpha.meaning, "newer_meaning_from_b");
        assert_eq!(beta.meaning, "from_a");
    }

    #[test]
    fn webdav_push_and_pull_over_mock_http() {
        let (base_url, push_rx) =
            crate::http_mock::spawn_one_shot("201 Created", "text/plain", Vec::new());
        let settings = WebDavSettings {
            url: base_url,
            username: "alice".into(),
            password: "pw".into(),
        };
        tauri::async_runtime::block_on(webdav_push(&settings, r#"{"version":1}"#.into()))
            .expect("push");
        let push_req = push_rx.recv().expect("push req");
        assert!(
            push_req
                .request_line
                .starts_with("PUT /vlearning-backup.json ")
        );
        assert!(
            push_req
                .headers
                .to_ascii_lowercase()
                .contains("authorization: basic ")
        );
        assert_eq!(push_req.body, r#"{"version":1}"#);

        let (pull_url, pull_rx) = crate::http_mock::spawn_one_shot(
            "200 OK",
            "application/json",
            br#"{"version":1,"decks":[]}"#.to_vec(),
        );
        let pull_settings = WebDavSettings {
            url: format!("{pull_url}/custom.json"),
            username: String::new(),
            password: String::new(),
        };
        let pulled = tauri::async_runtime::block_on(webdav_pull(&pull_settings)).expect("pull");
        assert_eq!(pulled, r#"{"version":1,"decks":[]}"#);
        let pull_req = pull_rx.recv().expect("pull req");
        assert!(pull_req.request_line.starts_with("GET /custom.json "));

        let (err_url, _) =
            crate::http_mock::spawn_one_shot("404 Not Found", "text/plain", b"missing".to_vec());
        let err_settings = WebDavSettings {
            url: err_url,
            username: String::new(),
            password: String::new(),
        };
        assert!(tauri::async_runtime::block_on(webdav_pull(&err_settings)).is_err());
    }
}
