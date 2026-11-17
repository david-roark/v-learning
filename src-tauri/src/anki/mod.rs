use std::fs;
use std::io::{Cursor, Read};

use rusqlite::Connection;
use uuid::Uuid;
use zip::ZipArchive;

use crate::db::{deck_repo, note_repo};
use crate::error::{AppError, AppResult};
use crate::models::{ApkgImportReport, CreateDeckInput, CreateNoteInput};

fn strip_html_and_sound(input: &str) -> String {
    // First remove [sound:...] tags without stripping legitimate [n.], [adj.], or IPA brackets
    let mut no_sound = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(pos) = rest.find("[sound:") {
        no_sound.push_str(&rest[..pos]);
        if let Some(end_rel) = rest[pos..].find(']') {
            rest = &rest[pos + end_rel + 1..];
        } else {
            rest = &rest[pos + 7..];
        }
    }
    no_sound.push_str(rest);

    let mut out = String::with_capacity(no_sound.len());
    let mut in_tag = false;
    for ch in no_sound.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }

    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn import_apkg_bytes(
    conn: &mut Connection,
    deck_name: &str,
    bytes: &[u8],
) -> AppResult<ApkgImportReport> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| AppError::Validation(format!("File .apkg không hợp lệ: {e}")))?;

    let db_entry_name = if archive.by_name("collection.anki21").is_ok() {
        "collection.anki21"
    } else if archive.by_name("collection.anki2").is_ok() {
        "collection.anki2"
    } else {
        return Err(AppError::Validation(
            "Không tìm thấy collection.anki2/anki21 trong file .apkg".into(),
        ));
    };

    let mut db_bytes = Vec::new();
    {
        let mut file = archive
            .by_name(db_entry_name)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        file.read_to_end(&mut db_bytes)
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }

    let temp_path = std::env::temp_dir().join(format!("vlearning_apkg_{}.db", Uuid::new_v4()));
    fs::write(&temp_path, &db_bytes).map_err(|e| AppError::Internal(e.to_string()))?;

    let anki_conn = Connection::open(&temp_path);
    let result = (|| -> AppResult<ApkgImportReport> {
        let anki = anki_conn?;
        let mut stmt = anki.prepare("SELECT flds FROM notes")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        let clean_name = if deck_name.trim().is_empty() {
            "Imported Anki Deck".to_string()
        } else {
            deck_name.trim().to_string()
        };

        let deck = deck_repo::create_deck(
            conn,
            CreateDeckInput {
                name: clean_name.clone(),
                description: Some("Imported from Anki (.apkg)".into()),
                new_cards_per_day: None,
                desired_retention: None,
            },
        )?;

        let mut imported = 0_i64;
        let mut skipped = 0_i64;

        for flds_res in rows {
            let flds = flds_res?;
            let parts: Vec<&str> = flds.split('\x1f').collect();
            let word = strip_html_and_sound(parts.first().copied().unwrap_or(""));
            let meaning = strip_html_and_sound(parts.get(1).copied().unwrap_or(""));
            let example = parts
                .get(2)
                .map(|s| strip_html_and_sound(s))
                .filter(|s| !s.is_empty());

            if word.is_empty() || meaning.is_empty() {
                skipped += 1;
                continue;
            }

            match note_repo::create_note(
                conn,
                CreateNoteInput {
                    deck_id: deck.id.clone(),
                    note_type: "vocab".into(),
                    word,
                    ipa: None,
                    meaning,
                    example_sentence: example,
                    collocations: None,
                    tags: None,
                },
            ) {
                Ok(_) => imported += 1,
                Err(_) => skipped += 1,
            }
        }

        Ok(ApkgImportReport {
            deck_id: deck.id,
            deck_name: clean_name,
            imported,
            skipped,
        })
    })();

    let _ = fs::remove_file(&temp_path);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory_for_test;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    #[test]
    fn import_apkg_extracts_notes_and_strips_html() {
        let anki_tmp = std::env::temp_dir().join(format!("test_anki_{}.db", Uuid::new_v4()));
        {
            let anki = Connection::open(&anki_tmp).expect("create anki db");
            anki.execute_batch("CREATE TABLE notes (id INTEGER PRIMARY KEY, flds TEXT NOT NULL);")
                .expect("create table");
            anki.execute(
                "INSERT INTO notes (id, flds) VALUES (1, ?1)",
                ["<b>ephemeral</b>[sound:test.mp3]\x1f<div>ngắn ngủi</div>\x1fFashions are ephemeral."],
            )
            .expect("insert note");
        }
        let anki_bytes = fs::read(&anki_tmp).expect("read anki db");
        let _ = fs::remove_file(&anki_tmp);

        let mut zip_buf = Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut zip_buf);
            zip.start_file("collection.anki2", SimpleFileOptions::default())
                .expect("start file");
            zip.write_all(&anki_bytes).expect("write db");
            zip.finish().expect("finish zip");
        }

        let mut conn = open_in_memory_for_test();
        let report =
            import_apkg_bytes(&mut conn, "My Anki Deck", zip_buf.get_ref()).expect("import apkg");
        assert_eq!(report.imported, 1);
        assert_eq!(report.skipped, 0);

        let notes = note_repo::list_notes_by_deck(&conn, &report.deck_id).expect("list");
        assert_eq!(notes[0].word, "ephemeral");
        assert_eq!(notes[0].meaning, "ngắn ngủi");
        assert_eq!(
            notes[0].example_sentence.as_deref(),
            Some("Fashions are ephemeral.")
        );
    }

    #[test]
    fn strip_html_and_sound_preserves_brackets_other_than_sound() {
        assert_eq!(
            strip_html_and_sound("<b>ubiquitous</b> [adj.] [sound:ubi.mp3]"),
            "ubiquitous [adj.]"
        );
        let mut conn = open_in_memory_for_test();
        assert!(import_apkg_bytes(&mut conn, "Bad", b"not-a-zip").is_err());
    }
}
