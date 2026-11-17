use chrono::Utc;
use rusqlite::{Connection, Row, params};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{CreateNoteInput, Note, UpdateNoteInput};

pub fn row_to_note(row: &Row<'_>, offset: usize) -> rusqlite::Result<Note> {
    let collocations_json: String = row.get(offset + 7)?;
    let tags_json: String = row.get(offset + 8)?;

    Ok(Note {
        id: row.get(offset)?,
        deck_id: row.get(offset + 1)?,
        note_type: row.get(offset + 2)?,
        word: row.get(offset + 3)?,
        ipa: row.get(offset + 4)?,
        meaning: row.get(offset + 5)?,
        example_sentence: row.get(offset + 6)?,
        collocations: serde_json::from_str(&collocations_json).unwrap_or_default(),
        tags: serde_json::from_str(&tags_json).unwrap_or_default(),
        created_at: row.get(offset + 9)?,
        updated_at: row.get(offset + 10)?,
    })
}

pub fn list_notes_by_deck(conn: &Connection, deck_id: &str) -> AppResult<Vec<Note>> {
    let mut stmt = conn.prepare(
        "SELECT id, deck_id, note_type, word, ipa, meaning, example_sentence,
                collocations, tags, created_at, updated_at
         FROM notes
         WHERE deck_id = ?1
         ORDER BY created_at DESC",
    )?;

    let rows = stmt.query_map(params![deck_id], |row| row_to_note(row, 0))?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn create_note(conn: &mut Connection, input: CreateNoteInput) -> AppResult<Note> {
    let word = input.word.trim().to_string();
    let meaning = input.meaning.trim().to_string();
    if word.is_empty() || meaning.is_empty() {
        return Err(AppError::Validation(
            "Từ vựng và nghĩa không được để trống".into(),
        ));
    }

    let duplicate: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM notes WHERE deck_id = ?1 AND word = ?2 COLLATE NOCASE)",
        params![input.deck_id, word],
        |r| r.get(0),
    )?;
    if duplicate {
        return Err(AppError::Validation(format!(
            "Từ \"{word}\" đã tồn tại trong bộ thẻ này"
        )));
    }

    let now_ms = Utc::now().timestamp_millis();
    let note_type = match input.note_type.as_str() {
        "cloze" | "vocab" => input.note_type,
        _ => "vocab".to_string(),
    };

    let note = Note {
        id: Uuid::new_v4().to_string(),
        deck_id: input.deck_id,
        note_type: note_type.clone(),
        word,
        ipa: input
            .ipa
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        meaning,
        example_sentence: input
            .example_sentence
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        collocations: input.collocations.unwrap_or_default(),
        tags: input.tags.unwrap_or_default(),
        created_at: now_ms,
        updated_at: now_ms,
    };

    let collocations_json =
        serde_json::to_string(&note.collocations).unwrap_or_else(|_| "[]".into());
    let tags_json = serde_json::to_string(&note.tags).unwrap_or_else(|_| "[]".into());
    let template = if note_type == "cloze" {
        "cloze"
    } else {
        "recognition"
    };

    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO notes (id, deck_id, note_type, word, ipa, meaning, example_sentence, collocations, tags, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            note.id,
            note.deck_id,
            note.note_type,
            note.word,
            note.ipa,
            note.meaning,
            note.example_sentence,
            collocations_json,
            tags_json,
            note.created_at,
            note.updated_at
        ],
    )?;

    let card_id = Uuid::new_v4().to_string();
    tx.execute(
        "INSERT INTO cards (id, note_id, card_template, state, due)
         VALUES (?1, ?2, ?3, 0, ?4)",
        params![card_id, note.id, template, now_ms],
    )?;
    tx.commit()?;

    Ok(note)
}

pub fn update_note(conn: &Connection, input: UpdateNoteInput) -> AppResult<()> {
    let word = input.word.trim().to_string();
    let meaning = input.meaning.trim().to_string();
    if word.is_empty() || meaning.is_empty() {
        return Err(AppError::Validation(
            "Từ vựng và nghĩa không được để trống".into(),
        ));
    }

    let duplicate: bool = conn.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM notes
            WHERE deck_id = (SELECT deck_id FROM notes WHERE id = ?1)
              AND word = ?2 COLLATE NOCASE
              AND id != ?1
        )",
        params![input.id, word],
        |r| r.get(0),
    )?;
    if duplicate {
        return Err(AppError::Validation(format!(
            "Từ \"{word}\" đã tồn tại trong bộ thẻ này"
        )));
    }

    let now_ms = Utc::now().timestamp_millis();
    let collocations_json = serde_json::to_string(&input.collocations.unwrap_or_default())
        .unwrap_or_else(|_| "[]".into());
    let tags_json =
        serde_json::to_string(&input.tags.unwrap_or_default()).unwrap_or_else(|_| "[]".into());

    let affected = conn.execute(
        "UPDATE notes
         SET word = ?1, ipa = ?2, meaning = ?3, example_sentence = ?4,
             collocations = ?5, tags = ?6, updated_at = ?7
         WHERE id = ?8",
        params![
            word,
            input
                .ipa
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            meaning,
            input
                .example_sentence
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            collocations_json,
            tags_json,
            now_ms,
            input.id
        ],
    )?;

    if affected == 0 {
        return Err(AppError::NotFound("Không tìm thấy thẻ".into()));
    }
    Ok(())
}

pub fn delete_note(conn: &Connection, note_id: &str) -> AppResult<()> {
    let affected = conn.execute("DELETE FROM notes WHERE id = ?1", params![note_id])?;
    if affected == 0 {
        return Err(AppError::NotFound("Không tìm thấy thẻ".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::deck_repo::create_deck;
    use crate::db::open_in_memory_for_test;
    use crate::models::CreateDeckInput;

    #[test]
    fn create_update_delete_note_and_card_cascade() {
        let mut conn = open_in_memory_for_test();
        let deck = create_deck(
            &conn,
            CreateDeckInput {
                name: "Test Deck".into(),
                description: None,
                new_cards_per_day: None,
                desired_retention: None,
            },
        )
        .expect("deck");

        let note = create_note(
            &mut conn,
            CreateNoteInput {
                deck_id: deck.id.clone(),
                note_type: "vocab".into(),
                word: "ubiquitous".into(),
                ipa: Some("/juːˈbɪk.wɪ.təs/".into()),
                meaning: "có mặt khắp nơi".into(),
                example_sentence: Some("Smartphones are ubiquitous.".into()),
                collocations: Some(vec!["become ubiquitous".into()]),
                tags: None,
            },
        )
        .expect("create note");

        let notes = list_notes_by_deck(&conn, &deck.id).expect("list notes");
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].collocations, vec!["become ubiquitous"]);

        let card_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM cards WHERE note_id = ?1",
                params![note.id],
                |r| r.get(0),
            )
            .expect("count cards");
        assert_eq!(card_count, 1);

        update_note(
            &conn,
            UpdateNoteInput {
                id: note.id.clone(),
                word: "ubiquitous".into(),
                ipa: None,
                meaning: "phổ biến khắp nơi".into(),
                example_sentence: None,
                collocations: None,
                tags: None,
            },
        )
        .expect("update note");

        let updated = list_notes_by_deck(&conn, &deck.id).expect("list after update");
        assert_eq!(updated[0].meaning, "phổ biến khắp nơi");

        delete_note(&conn, &note.id).expect("delete note");
        let cards_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM cards", [], |r| r.get(0))
            .expect("count cards after delete");
        assert_eq!(cards_after, 0);
    }

    #[test]
    fn create_note_rejects_case_insensitive_duplicate_in_same_deck() {
        let mut conn = open_in_memory_for_test();
        let d1 = create_deck(
            &conn,
            CreateDeckInput {
                name: "Deck 1".into(),
                description: None,
                new_cards_per_day: None,
                desired_retention: None,
            },
        )
        .expect("d1");
        let d2 = create_deck(
            &conn,
            CreateDeckInput {
                name: "Deck 2".into(),
                description: None,
                new_cards_per_day: None,
                desired_retention: None,
            },
        )
        .expect("d2");

        let make = |deck_id: String, word: &str| CreateNoteInput {
            deck_id,
            note_type: "vocab".into(),
            word: word.into(),
            ipa: None,
            meaning: "thực tế".into(),
            example_sentence: None,
            collocations: None,
            tags: None,
        };

        create_note(&mut conn, make(d1.id.clone(), "Pragmatic")).expect("first");
        assert!(create_note(&mut conn, make(d1.id.clone(), "pragmatic")).is_err());
        assert!(create_note(&mut conn, make(d2.id, "pragmatic")).is_ok());

        let second = create_note(&mut conn, make(d1.id.clone(), "Ephemeral")).expect("second");
        let dup_update = update_note(
            &conn,
            UpdateNoteInput {
                id: second.id.clone(),
                word: "PRAGMATIC".into(),
                ipa: None,
                meaning: "thực tế".into(),
                example_sentence: None,
                collocations: None,
                tags: None,
            },
        );
        assert!(dup_update.is_err());

        // Empty word/meaning validation and cloze template
        assert!(
            create_note(&mut conn, make(d1.id.clone(), "   ")).is_err(),
            "empty word should fail"
        );
        let cloze = create_note(
            &mut conn,
            CreateNoteInput {
                deck_id: d1.id,
                note_type: "cloze".into(),
                word: "eloquent".into(),
                ipa: None,
                meaning: "hùng hồn".into(),
                example_sentence: Some("An eloquent speech.".into()),
                collocations: None,
                tags: None,
            },
        )
        .expect("cloze note");
        let tpl: String = conn
            .query_row(
                "SELECT card_template FROM cards WHERE note_id = ?1",
                params![cloze.id],
                |r| r.get(0),
            )
            .expect("card template");
        assert_eq!(tpl, "cloze");

        assert!(
            update_note(
                &conn,
                UpdateNoteInput {
                    id: "missing-note".into(),
                    word: "valid".into(),
                    ipa: None,
                    meaning: "hợp lệ".into(),
                    example_sentence: None,
                    collocations: None,
                    tags: None,
                },
            )
            .is_err()
        );
    }
}
