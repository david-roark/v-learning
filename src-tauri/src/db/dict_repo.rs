use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::error::AppResult;
use crate::models::{DictionaryEntry, UpsertDictionaryInput};

fn row_to_entry(row: &Row<'_>) -> rusqlite::Result<DictionaryEntry> {
    Ok(DictionaryEntry {
        id: row.get(0)?,
        word: row.get(1)?,
        ipa: row.get(2)?,
        ipa_us: None,
        ipa_uk: None,
        pos: row.get(3)?,
        meaning_vi: row.get(4)?,
        meaning_en: row.get(5)?,
        example_sentence: row.get(6)?,
        audio_url: row.get(7)?,
        source: row.get(8)?,
        matched_form: None,
        variants: Vec::new(),
        sources: Vec::new(),
        senses: Vec::new(),
    })
}

pub fn find_exact(conn: &Connection, word: &str) -> AppResult<Option<DictionaryEntry>> {
    let trimmed = word.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let entry = conn
        .query_row(
            "SELECT id, word, ipa, pos, meaning_vi, meaning_en, example_sentence, audio_url, source
             FROM dictionary_entries
             WHERE word = ?1 COLLATE NOCASE",
            params![trimmed],
            row_to_entry,
        )
        .optional()?;

    Ok(entry)
}

pub fn search_fts(conn: &Connection, query: &str, limit: i64) -> AppResult<Vec<DictionaryEntry>> {
    let sanitized: String = query
        .trim()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '\'')
        .collect();

    if sanitized.is_empty() {
        return Ok(Vec::new());
    }

    let fts_query = format!("\"{sanitized}\"*");
    let mut stmt = conn.prepare(
        "SELECT d.id, d.word, d.ipa, d.pos, d.meaning_vi, d.meaning_en, d.example_sentence, d.audio_url, d.source
         FROM dictionary_fts f
         JOIN dictionary_entries d ON d.id = f.rowid
         WHERE dictionary_fts MATCH ?1
         ORDER BY bm25(dictionary_fts)
         LIMIT ?2",
    )?;

    let rows = stmt.query_map(params![fts_query, limit], row_to_entry)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn count_entries(conn: &Connection) -> AppResult<i64> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM dictionary_entries", [], |r| r.get(0))?;
    Ok(count)
}

pub fn list_entries(conn: &Connection, limit: i64, offset: i64) -> AppResult<Vec<DictionaryEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, word, ipa, pos, meaning_vi, meaning_en, example_sentence, audio_url, source
         FROM dictionary_entries
         ORDER BY word ASC
         LIMIT ?1 OFFSET ?2",
    )?;
    let rows = stmt.query_map(params![limit, offset], row_to_entry)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn bulk_import_tsv(conn: &mut Connection, content: &str) -> AppResult<i64> {
    let tx = conn.transaction()?;
    let mut imported = 0_i64;

    {
        let mut stmt = tx.prepare(
            "INSERT INTO dictionary_entries (word, ipa, pos, meaning_vi, example_sentence, source)
             VALUES (?1, ?2, ?3, ?4, ?5, 'imported')
             ON CONFLICT(word) DO UPDATE SET
               ipa = COALESCE(excluded.ipa, dictionary_entries.ipa),
               pos = COALESCE(excluded.pos, dictionary_entries.pos),
               meaning_vi = COALESCE(excluded.meaning_vi, dictionary_entries.meaning_vi),
               example_sentence = COALESCE(excluded.example_sentence, dictionary_entries.example_sentence)",
        )?;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let cols: Vec<&str> = line.trim_end_matches(['\r', '\n']).split('\t').collect();
            let word = cols.first().map(|s| s.trim()).unwrap_or("");
            if word.is_empty() {
                continue;
            }
            let ipa = cols.get(1).map(|s| s.trim()).filter(|s| !s.is_empty());
            let pos = cols.get(2).map(|s| s.trim()).filter(|s| !s.is_empty());
            let meaning_vi = cols.get(3).map(|s| s.trim()).filter(|s| !s.is_empty());
            let example = cols.get(4).map(|s| s.trim()).filter(|s| !s.is_empty());

            stmt.execute(params![word, ipa, pos, meaning_vi, example])?;
            imported += 1;
        }
    }

    tx.commit()?;
    Ok(imported)
}

pub fn upsert_entry(
    conn: &Connection,
    input: &UpsertDictionaryInput,
) -> AppResult<DictionaryEntry> {
    conn.execute(
        "INSERT INTO dictionary_entries (word, ipa, pos, meaning_vi, meaning_en, example_sentence, audio_url, source)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(word) DO UPDATE SET
           ipa = COALESCE(excluded.ipa, dictionary_entries.ipa),
           pos = COALESCE(excluded.pos, dictionary_entries.pos),
           meaning_vi = COALESCE(excluded.meaning_vi, dictionary_entries.meaning_vi),
           meaning_en = COALESCE(excluded.meaning_en, dictionary_entries.meaning_en),
           example_sentence = COALESCE(excluded.example_sentence, dictionary_entries.example_sentence),
           audio_url = COALESCE(excluded.audio_url, dictionary_entries.audio_url),
           source = excluded.source",
        params![
            input.word.trim(),
            input.ipa,
            input.pos,
            input.meaning_vi,
            input.meaning_en,
            input.example_sentence,
            input.audio_url,
            input.source
        ],
    )?;

    find_exact(conn, &input.word)?
        .ok_or_else(|| crate::error::AppError::Internal("Failed to read upserted entry".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory_for_test;

    #[test]
    fn fts_search_and_upsert_work() {
        let conn = open_in_memory_for_test();

        upsert_entry(
            &conn,
            &UpsertDictionaryInput {
                word: "ubiquitous".into(),
                ipa: Some("/juːˈbɪk.wɪ.təs/".into()),
                pos: Some("adj".into()),
                meaning_vi: Some("có mặt ở khắp mọi nơi".into()),
                meaning_en: None,
                example_sentence: None,
                audio_url: None,
                source: "tsv".into(),
            },
        )
        .expect("seed ubiquitous");

        upsert_entry(
            &conn,
            &UpsertDictionaryInput {
                word: "pragmatic".into(),
                ipa: Some("/præɡˈmæt.ɪk/".into()),
                pos: Some("adj".into()),
                meaning_vi: Some("thực tế".into()),
                meaning_en: None,
                example_sentence: None,
                audio_url: None,
                source: "tsv".into(),
            },
        )
        .expect("seed pragmatic");

        let exact = find_exact(&conn, "Ubiquitous").expect("find_exact");
        assert!(exact.is_some());
        assert_eq!(exact.unwrap().ipa.as_deref(), Some("/juːˈbɪk.wɪ.təs/"));

        let fts = search_fts(&conn, "prag", 10).expect("fts");
        assert_eq!(fts.len(), 1);
        assert_eq!(fts[0].word, "pragmatic");

        let upserted = upsert_entry(
            &conn,
            &UpsertDictionaryInput {
                word: "serendipity".into(),
                ipa: Some("/ˌser.ənˈdɪp.ə.ti/".into()),
                pos: Some("noun".into()),
                meaning_vi: Some("sự tình cờ may mắn".into()),
                meaning_en: Some("the occurrence of finding pleasant things by chance".into()),
                example_sentence: Some("Finding this book was pure serendipity.".into()),
                audio_url: None,
                source: "online".into(),
            },
        )
        .expect("upsert");

        assert_eq!(upserted.word, "serendipity");
        let found = search_fts(&conn, "serend", 5).expect("search new word");
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn bulk_import_tsv_skips_comments_empty_word_col_and_handles_empty_search() {
        let mut conn = open_in_memory_for_test();
        let tsv = "# comment line\n\n\t/ˈfɔː.nə/\tnoun\tđộng vật\nresilient\t/rɪˈzɪl.jənt/\tadj\tkiên cường\tStay resilient.\n";
        let imported = bulk_import_tsv(&mut conn, tsv).expect("import tsv");
        assert_eq!(imported, 1);
        assert!(find_exact(&conn, "/ˈfɔː.nə/").expect("find").is_none());

        let entry = find_exact(&conn, "resilient")
            .expect("find")
            .expect("exists");
        assert_eq!(entry.meaning_vi.as_deref(), Some("kiên cường"));
        assert!(search_fts(&conn, "!!!", 10).expect("punct").is_empty());
    }
}
