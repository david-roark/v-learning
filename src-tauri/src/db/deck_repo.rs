use chrono::Utc;
use rusqlite::{Connection, params};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{CreateDeckInput, Deck, DeckWithStats};

pub fn list_decks(conn: &Connection) -> AppResult<Vec<DeckWithStats>> {
    let now_ms = Utc::now().timestamp_millis();
    let mut stmt = conn.prepare(
        r#"
        SELECT
            d.id,
            d.name,
            d.description,
            d.new_cards_per_day,
            d.desired_retention,
            d.created_at,
            COUNT(c.id) AS total_cards,
            COALESCE(SUM(CASE WHEN c.state = 0 THEN 1 ELSE 0 END), 0) AS new_count,
            COALESCE(SUM(CASE WHEN c.state != 0 AND c.due <= ?1 THEN 1 ELSE 0 END), 0) AS due_count
        FROM decks d
        LEFT JOIN notes n ON n.deck_id = d.id
        LEFT JOIN cards c ON c.note_id = n.id
        GROUP BY d.id
        ORDER BY d.created_at ASC
        "#,
    )?;

    let rows = stmt.query_map(params![now_ms], |row| {
        Ok(DeckWithStats {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            new_cards_per_day: row.get(3)?,
            desired_retention: row.get(4)?,
            created_at: row.get(5)?,
            total_cards: row.get(6)?,
            new_count: row.get(7)?,
            due_count: row.get(8)?,
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn create_deck(conn: &Connection, input: CreateDeckInput) -> AppResult<Deck> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Validation(
            "Tên bộ thẻ không được để trống".into(),
        ));
    }

    let deck = Deck {
        id: Uuid::new_v4().to_string(),
        name,
        description: input
            .description
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        new_cards_per_day: input.new_cards_per_day.unwrap_or(20).clamp(1, 500),
        desired_retention: input.desired_retention.unwrap_or(0.90).clamp(0.70, 0.99),
        created_at: Utc::now().timestamp_millis(),
    };

    conn.execute(
        "INSERT INTO decks (id, name, description, new_cards_per_day, desired_retention, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            deck.id,
            deck.name,
            deck.description,
            deck.new_cards_per_day,
            deck.desired_retention,
            deck.created_at
        ],
    )?;

    Ok(deck)
}

pub fn delete_deck(conn: &Connection, deck_id: &str) -> AppResult<()> {
    let affected = conn.execute("DELETE FROM decks WHERE id = ?1", params![deck_id])?;
    if affected == 0 {
        return Err(AppError::NotFound("Không tìm thấy bộ thẻ".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory_for_test;

    #[test]
    fn crud_deck_works_and_validates_name() {
        let conn = open_in_memory_for_test();

        let empty = create_deck(
            &conn,
            CreateDeckInput {
                name: "   ".into(),
                description: None,
                new_cards_per_day: None,
                desired_retention: None,
            },
        );
        assert!(empty.is_err());

        let deck = create_deck(
            &conn,
            CreateDeckInput {
                name: "IELTS Vocab".into(),
                description: Some("Core words".into()),
                new_cards_per_day: Some(30),
                desired_retention: Some(0.92),
            },
        )
        .expect("create deck");

        let list = list_decks(&conn).expect("list decks");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "IELTS Vocab");
        assert_eq!(list[0].new_cards_per_day, 30);
        assert_eq!(list[0].total_cards, 0);

        delete_deck(&conn, &deck.id).expect("delete deck");
        assert!(list_decks(&conn).expect("list after delete").is_empty());
        assert!(delete_deck(&conn, "missing-id").is_err());
    }

    #[test]
    fn create_deck_clamps_limits_and_tracks_stats() {
        let conn = open_in_memory_for_test();
        let clamped = create_deck(
            &conn,
            CreateDeckInput {
                name: "Clamped".into(),
                description: Some("   ".into()),
                new_cards_per_day: Some(0),
                desired_retention: Some(1.5),
            },
        )
        .expect("clamped deck");
        assert_eq!(clamped.new_cards_per_day, 1);
        assert!((clamped.desired_retention - 0.99).abs() < 1e-6);
        assert!(clamped.description.is_none());

        let high = create_deck(
            &conn,
            CreateDeckInput {
                name: "High".into(),
                description: None,
                new_cards_per_day: Some(9999),
                desired_retention: Some(0.1),
            },
        )
        .expect("high deck");
        assert_eq!(high.new_cards_per_day, 500);
        assert!((high.desired_retention - 0.70).abs() < 1e-6);
    }
}
