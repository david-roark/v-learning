use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use serde::Serialize;

use crate::error::{AppError, AppResult};
use crate::models::{DictionaryEntry, DictionarySense};

pub struct DictPackState(pub Mutex<Option<Connection>>);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictPackInfo {
    pub installed: bool,
    pub profile: Option<String>,
    pub word_count: i64,
    pub sense_count: i64,
    pub form_count: i64,
    pub built_at: Option<String>,
    pub attribution: Option<String>,
}

pub fn open_pack_readonly(path: &Path) -> AppResult<Option<Connection>> {
    if !path.exists() {
        return Ok(None);
    }
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    Ok(Some(conn))
}

fn join_json_array(raw: Option<String>, max_items: usize) -> Vec<String> {
    let Some(s) = raw else {
        return Vec::new();
    };
    serde_json::from_str::<Vec<String>>(&s)
        .unwrap_or_default()
        .into_iter()
        .take(max_items)
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn load_entry_with_senses(
    conn: &Connection,
    id: i64,
    word: String,
    ipa_us: Option<String>,
    ipa_uk: Option<String>,
    meaning_vi: Option<String>,
    variants_raw: Option<String>,
    sources_raw: Option<String>,
    matched_form: Option<String>,
) -> AppResult<DictionaryEntry> {
    let mut stmt = conn.prepare(
        "SELECT pos, definition_en, definition_vi, examples, tags, synonyms, antonyms
         FROM senses WHERE entry_id = ?1 ORDER BY id LIMIT 16",
    )?;
    let rows = stmt.query_map(params![id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, Option<String>>(3)?,
            r.get::<_, Option<String>>(4)?,
            r.get::<_, Option<String>>(5)?,
            r.get::<_, Option<String>>(6)?,
        ))
    })?;

    let mut poses = Vec::new();
    let mut en_defs = Vec::new();
    let mut vi_defs = Vec::new();
    let mut first_example: Option<String> = None;
    let mut senses = Vec::new();

    for row in rows {
        let (pos, def_en, def_vi, examples_raw, tags_raw, syns_raw, ants_raw) = row?;
        if !poses.contains(&pos) {
            poses.push(pos.clone());
        }
        let examples = join_json_array(examples_raw, 4);
        let tags = join_json_array(tags_raw, 6);
        let synonyms = join_json_array(syns_raw, 8);
        let antonyms = join_json_array(ants_raw, 6);

        if let Some(en) = def_en.as_deref()
            && !en.trim().is_empty()
        {
            en_defs.push(format!("[{pos}] {en}"));
        }
        if let Some(vi) = def_vi.as_deref()
            && !vi.trim().is_empty()
            && !vi_defs.contains(&vi.to_string())
        {
            vi_defs.push(vi.to_string());
        }
        if first_example.is_none() {
            first_example = examples.first().cloned();
        }

        senses.push(DictionarySense {
            pos,
            definition_en: def_en.filter(|s| !s.trim().is_empty()),
            definition_vi: def_vi.filter(|s| !s.trim().is_empty()),
            examples,
            tags,
            synonyms,
            antonyms,
        });
    }

    let final_vi = if vi_defs.is_empty() {
        meaning_vi
    } else {
        Some(vi_defs.join("; "))
    };
    let final_en = if en_defs.is_empty() {
        None
    } else {
        Some(en_defs.join("\n"))
    };
    let ipa = ipa_us.clone().or_else(|| ipa_uk.clone());

    Ok(DictionaryEntry {
        id: -id, // Negative ID distinguishes read-only pack entries from user DB entries
        word,
        ipa,
        ipa_us,
        ipa_uk,
        pos: if poses.is_empty() {
            None
        } else {
            Some(poses.join(", "))
        },
        meaning_vi: final_vi,
        meaning_en: final_en,
        example_sentence: first_example,
        audio_url: None,
        source: "dict-pack".to_string(),
        matched_form,
        variants: join_json_array(variants_raw, 8),
        sources: join_json_array(sources_raw, 8),
        senses,
    })
}

pub fn find_exact_or_lemma(conn: &Connection, word: &str) -> AppResult<Option<DictionaryEntry>> {
    let trimmed = word.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    // 1. Direct match in entries
    let direct = conn
        .query_row(
            "SELECT id, word, ipa_us, ipa_uk, meaning_vi, variants, sources
             FROM entries WHERE word = ?1 COLLATE NOCASE LIMIT 1",
            params![trimmed],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, Option<String>>(6)?,
                ))
            },
        )
        .optional()?;

    if let Some((id, w, ipa_us, ipa_uk, m_vi, variants, sources)) = direct {
        return load_entry_with_senses(conn, id, w, ipa_us, ipa_uk, m_vi, variants, sources, None)
            .map(Some);
    }

    // 2. Fallback to word_forms (inflections & UK/US spelling variants)
    let via_form = conn
        .query_row(
            "SELECT e.id, e.word, e.ipa_us, e.ipa_uk, e.meaning_vi, e.variants, e.sources
             FROM word_forms wf
             JOIN entries e ON e.id = wf.entry_id
             WHERE wf.form = ?1 COLLATE NOCASE
             LIMIT 1",
            params![trimmed],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, Option<String>>(6)?,
                ))
            },
        )
        .optional()?;

    if let Some((id, w, ipa_us, ipa_uk, m_vi, variants, sources)) = via_form {
        return load_entry_with_senses(
            conn,
            id,
            w,
            ipa_us,
            ipa_uk,
            m_vi,
            variants,
            sources,
            Some(trimmed.to_string()),
        )
        .map(Some);
    }

    Ok(None)
}

pub fn search_fts(conn: &Connection, query: &str, limit: i64) -> AppResult<Vec<DictionaryEntry>> {
    let clean = query.trim().replace('"', "");
    if clean.is_empty() {
        return Ok(Vec::new());
    }
    let fts_query = format!("\"{clean}\"*");
    let mut stmt = conn.prepare(
        "SELECT e.id, e.word, e.ipa_us, e.ipa_uk, e.meaning_vi, e.variants, e.sources
         FROM entries_fts f
         JOIN entries e ON e.id = f.rowid
         WHERE entries_fts MATCH ?1
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![fts_query, limit], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, Option<String>>(3)?,
            r.get::<_, Option<String>>(4)?,
            r.get::<_, Option<String>>(5)?,
            r.get::<_, Option<String>>(6)?,
        ))
    })?;

    let mut out = Vec::new();
    for row in rows {
        let (id, w, ipa_us, ipa_uk, m_vi, variants, sources) = row?;
        out.push(load_entry_with_senses(
            conn, id, w, ipa_us, ipa_uk, m_vi, variants, sources, None,
        )?);
    }
    Ok(out)
}

pub fn count_entries(conn: &Connection) -> AppResult<i64> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM entries", [], |r| r.get(0))?;
    Ok(count)
}

pub fn read_pack_info(conn: Option<&Connection>) -> AppResult<DictPackInfo> {
    let Some(conn) = conn else {
        return Ok(DictPackInfo {
            installed: false,
            profile: None,
            word_count: 0,
            sense_count: 0,
            form_count: 0,
            built_at: None,
            attribution: None,
        });
    };

    let mut meta_map = std::collections::HashMap::new();
    if let Ok(mut stmt) = conn.prepare("SELECT key, value FROM meta")
        && let Ok(rows) =
            stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
    {
        for row in rows.flatten() {
            meta_map.insert(row.0, row.1);
        }
    }

    let word_count = meta_map
        .get("word_count")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or_else(|| count_entries(conn).unwrap_or(0));
    let sense_count = meta_map
        .get("sense_count")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    let form_count = meta_map
        .get("form_count")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);

    Ok(DictPackInfo {
        installed: true,
        profile: meta_map.remove("profile"),
        word_count,
        sense_count,
        form_count,
        built_at: meta_map.remove("built_at"),
        attribution: meta_map.remove("attribution"),
    })
}

pub fn begin_pack_install(pack_path: &Path) -> AppResult<()> {
    if let Some(parent) = pack_path.parent() {
        fs::create_dir_all(parent).map_err(|e| AppError::Internal(e.to_string()))?;
    }
    let tmp_path = pack_path.with_extension("db.tmp");
    fs::write(&tmp_path, b"").map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub fn append_pack_chunk(pack_path: &Path, chunk: &[u8]) -> AppResult<()> {
    let tmp_path = pack_path.with_extension("db.tmp");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&tmp_path)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    file.write_all(chunk)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub fn finish_pack_install(state: &DictPackState, pack_path: &Path) -> AppResult<DictPackInfo> {
    let tmp_path = pack_path.with_extension("db.tmp");
    // Validate SQLite pack structure before replacing active pack
    {
        let test_conn = match open_pack_readonly(&tmp_path) {
            Ok(Some(c)) => c,
            Ok(None) => {
                return Err(AppError::Internal("Không mở được file dict.db tạm".into()));
            }
            Err(e) => {
                let _ = fs::remove_file(&tmp_path);
                return Err(e);
            }
        };
        let has_tables: i64 = test_conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('entries','senses','word_forms')",
                [],
                |r| r.get(0),
            )
            .map_err(|e| {
                let _ = fs::remove_file(&tmp_path);
                AppError::Validation(format!("File không phải gói dict.db hợp lệ: {e}"))
            })?;
        if has_tables < 3 {
            let _ = fs::remove_file(&tmp_path);
            return Err(AppError::Validation(
                "File dict.db thiếu bảng entries / senses / word_forms.".into(),
            ));
        }
    }

    let mut guard = state
        .0
        .lock()
        .map_err(|_| AppError::Internal("Dictionary pack lock poisoned".into()))?;
    // Close active connection before atomic rename
    *guard = None;
    fs::rename(&tmp_path, pack_path).map_err(|e| AppError::Internal(e.to_string()))?;
    let new_conn = open_pack_readonly(pack_path)?;
    let info = read_pack_info(new_conn.as_ref())?;
    *guard = new_conn;
    Ok(info)
}

pub fn remove_pack(state: &DictPackState, pack_path: &Path) -> AppResult<()> {
    let mut guard = state
        .0
        .lock()
        .map_err(|_| AppError::Internal("Dictionary pack lock poisoned".into()))?;
    *guard = None;
    if pack_path.exists() {
        fs::remove_file(pack_path).map_err(|e| AppError::Internal(e.to_string()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_sample_pack_in_memory() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory pack");
        conn.execute_batch(
            "
            CREATE TABLE entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                word TEXT NOT NULL UNIQUE COLLATE NOCASE,
                ipa_us TEXT,
                ipa_uk TEXT,
                frequency_rank INTEGER,
                variants TEXT,
                meaning_vi TEXT,
                sources TEXT NOT NULL
            );
            CREATE TABLE senses (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entry_id INTEGER NOT NULL,
                pos TEXT NOT NULL,
                definition_en TEXT,
                definition_vi TEXT,
                examples TEXT,
                tags TEXT,
                synonyms TEXT,
                antonyms TEXT
            );
            CREATE TABLE word_forms (
                form TEXT NOT NULL COLLATE NOCASE,
                entry_id INTEGER NOT NULL,
                pos TEXT,
                morph_tag TEXT,
                PRIMARY KEY (form, entry_id)
            );
            CREATE VIRTUAL TABLE entries_fts USING fts5(word, meaning_vi, definitions_en, content='');

            INSERT INTO entries (id, word, ipa_us, meaning_vi, sources)
            VALUES (1, 'run', '/rʌn/', 'chạy', '[\"skypediacode\",\"kaikki-en\"]');
            INSERT INTO senses (entry_id, pos, definition_en, definition_vi, examples, synonyms)
            VALUES (1, 'verb', 'To move swiftly on foot.', 'chạy', '[\"I can run.\"]', '[\"sprint\"]');
            INSERT INTO word_forms (form, entry_id, pos, morph_tag)
            VALUES ('ran', 1, 'verb', 'past');
            INSERT INTO entries_fts (rowid, word, meaning_vi, definitions_en)
            VALUES (1, 'run', 'chạy', 'To move swiftly on foot.');
            ",
        )
        .expect("seed sample pack");
        conn
    }

    #[test]
    fn find_exact_or_lemma_resolves_inflections_and_fts() {
        let conn = open_sample_pack_in_memory();
        let via_lemma = find_exact_or_lemma(&conn, "ran")
            .expect("query")
            .expect("should resolve ran -> run");
        assert_eq!(via_lemma.word, "run");
        assert_eq!(via_lemma.source, "dict-pack");
        assert_eq!(via_lemma.matched_form.as_deref(), Some("ran"));
        assert_eq!(via_lemma.senses.len(), 1);
        assert!(via_lemma.senses[0].synonyms.contains(&"sprint".to_string()));

        let fts = search_fts(&conn, "swiftly", 10).expect("fts");
        assert_eq!(fts.len(), 1);
        assert_eq!(fts[0].word, "run");
        assert_eq!(count_entries(&conn).expect("count"), 1);
    }

    #[test]
    fn chunked_install_validates_and_removes_pack() {
        let dir = std::env::temp_dir().join(format!("vlearning-pack-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("mkdir");

        let src_db = dir.join("source.db");
        {
            let conn = Connection::open(&src_db).expect("open src");
            conn.execute_batch(
                "
                CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
                CREATE TABLE entries (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    word TEXT NOT NULL UNIQUE COLLATE NOCASE,
                    ipa_us TEXT, ipa_uk TEXT, frequency_rank INTEGER,
                    variants TEXT, meaning_vi TEXT, sources TEXT NOT NULL
                );
                CREATE TABLE senses (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    entry_id INTEGER NOT NULL, pos TEXT NOT NULL,
                    definition_en TEXT, definition_vi TEXT,
                    examples TEXT, tags TEXT, synonyms TEXT, antonyms TEXT
                );
                CREATE TABLE word_forms (
                    form TEXT NOT NULL COLLATE NOCASE, entry_id INTEGER NOT NULL,
                    pos TEXT, morph_tag TEXT, PRIMARY KEY (form, entry_id)
                );
                INSERT INTO meta (key, value) VALUES ('profile', 'modern'), ('word_count', '1'), ('sense_count', '1'), ('form_count', '0');
                INSERT INTO entries (id, word, ipa_us, meaning_vi, sources) VALUES (1, 'give up', '/ɡɪv ʌp/', 'từ bỏ', '[\"skypediacode\"]');
                INSERT INTO senses (entry_id, pos, definition_vi, tags, antonyms) VALUES (1, 'phrase', 'từ bỏ', '[\"phrasal\"]', '[\"persist\"]');
                ",
            )
            .expect("seed src");
        }

        let bytes = fs::read(&src_db).expect("read src");
        let pack_path = dir.join("dict.db");
        let state = DictPackState(Mutex::new(None));

        // Reject invalid bytes and clean up .db.tmp
        begin_pack_install(&pack_path).expect("begin invalid");
        append_pack_chunk(&pack_path, b"not a sqlite file").expect("append invalid");
        assert!(finish_pack_install(&state, &pack_path).is_err());
        assert!(!pack_path.with_extension("db.tmp").exists());

        // Stream valid DB in two chunks
        let mid = bytes.len() / 2;
        begin_pack_install(&pack_path).expect("begin valid");
        append_pack_chunk(&pack_path, &bytes[..mid]).expect("chunk 1");
        append_pack_chunk(&pack_path, &bytes[mid..]).expect("chunk 2");
        let info = finish_pack_install(&state, &pack_path).expect("finish");
        assert!(info.installed);
        assert_eq!(info.profile.as_deref(), Some("modern"));
        assert_eq!(info.word_count, 1);

        // Verify multi-word phrase lookup from installed pack
        let guard = state.0.lock().expect("lock");
        let phrase = find_exact_or_lemma(guard.as_ref().expect("opened"), "Give Up")
            .expect("lookup")
            .expect("phrase found");
        assert_eq!(phrase.word, "give up");
        assert_eq!(phrase.senses[0].antonyms, vec!["persist".to_string()]);
        drop(guard);

        remove_pack(&state, &pack_path).expect("remove");
        assert!(!pack_path.exists());
        let _ = fs::remove_dir_all(&dir);
    }
}
