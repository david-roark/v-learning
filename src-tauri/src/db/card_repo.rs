use chrono::{Local, TimeZone, Utc};
use rusqlite::{Connection, params};
use uuid::Uuid;

use crate::db::note_repo::row_to_note;
use crate::error::{AppError, AppResult};
use crate::models::{Card, DailyStats, StudyCard};
use crate::srs::{preview_intervals, schedule_card};

fn start_of_today_ms() -> i64 {
    let today = Local::now().date_naive();
    let naive_dt = today.and_hms_opt(0, 0, 0).expect("valid midnight");
    Local
        .from_local_datetime(&naive_dt)
        .single()
        .map(|dt| dt.timestamp_millis())
        .unwrap_or_else(|| Utc::now().timestamp_millis() - 86_400_000)
}

pub fn get_study_queue(conn: &Connection, deck_id: &str, limit: i64) -> AppResult<Vec<StudyCard>> {
    let now_ms = Utc::now().timestamp_millis();
    let today_start = start_of_today_ms();

    let new_cards_per_day: i64 = conn
        .query_row(
            "SELECT new_cards_per_day FROM decks WHERE id = ?1",
            params![deck_id],
            |r| r.get(0),
        )
        .unwrap_or(20);

    let new_learned_today: i64 = conn.query_row(
        "SELECT COUNT(*) FROM review_logs r
         JOIN cards c ON c.id = r.card_id
         JOIN notes n ON n.id = c.note_id
         WHERE n.deck_id = ?1 AND r.reviewed_at >= ?2 AND r.state = 0",
        params![deck_id, today_start],
        |r| r.get(0),
    )?;

    let remaining_new = (new_cards_per_day - new_learned_today).max(0).min(limit);

    let mut stmt = conn.prepare(
        r#"
        SELECT * FROM (
            SELECT
                c.id, c.note_id, c.card_template, c.state, c.due,
                c.stability, c.difficulty, c.elapsed_days, c.scheduled_days,
                c.reps, c.lapses, c.last_review,
                n.id, n.deck_id, n.note_type, n.word, n.ipa, n.meaning, n.example_sentence,
                n.collocations, n.tags, n.created_at, n.updated_at,
                d.desired_retention,
                0 AS sort_group
            FROM cards c
            JOIN notes n ON n.id = c.note_id
            JOIN decks d ON d.id = n.deck_id
            WHERE n.deck_id = ?1 AND c.state != 0 AND c.due <= ?2
            ORDER BY CASE WHEN c.state IN (1, 3) THEN 0 ELSE 1 END, c.due ASC
            LIMIT ?3
        )
        UNION ALL
        SELECT * FROM (
            SELECT
                c.id, c.note_id, c.card_template, c.state, c.due,
                c.stability, c.difficulty, c.elapsed_days, c.scheduled_days,
                c.reps, c.lapses, c.last_review,
                n.id, n.deck_id, n.note_type, n.word, n.ipa, n.meaning, n.example_sentence,
                n.collocations, n.tags, n.created_at, n.updated_at,
                d.desired_retention,
                1 AS sort_group
            FROM cards c
            JOIN notes n ON n.id = c.note_id
            JOIN decks d ON d.id = n.deck_id
            WHERE n.deck_id = ?1 AND c.state = 0
            ORDER BY c.due ASC
            LIMIT ?4
        )
        LIMIT ?3
        "#,
    )?;

    let rows = stmt.query_map(params![deck_id, now_ms, limit, remaining_new], |row| {
        let card = Card {
            id: row.get(0)?,
            note_id: row.get(1)?,
            card_template: row.get(2)?,
            state: row.get(3)?,
            due: row.get(4)?,
            stability: row.get(5)?,
            difficulty: row.get(6)?,
            elapsed_days: row.get(7)?,
            scheduled_days: row.get(8)?,
            reps: row.get(9)?,
            lapses: row.get(10)?,
            last_review: row.get(11)?,
        };
        let note = row_to_note(row, 12)?;
        let desired_retention: f64 = row.get(23)?;
        Ok((card, note, desired_retention))
    })?;

    let mut result = Vec::new();
    for item in rows {
        let (card, note, desired_retention) = item?;
        let intervals = preview_intervals(&card, desired_retention)?;
        result.push(StudyCard {
            card,
            note,
            desired_retention,
            intervals,
        });
    }

    Ok(result)
}

pub fn submit_review(
    conn: &mut Connection,
    card_id: &str,
    rating: i64,
    duration_ms: Option<i64>,
) -> AppResult<()> {
    let (card, desired_retention): (Card, f64) = conn
        .query_row(
            r#"
            SELECT
                c.id, c.note_id, c.card_template, c.state, c.due,
                c.stability, c.difficulty, c.elapsed_days, c.scheduled_days,
                c.reps, c.lapses, c.last_review, d.desired_retention
            FROM cards c
            JOIN notes n ON n.id = c.note_id
            JOIN decks d ON d.id = n.deck_id
            WHERE c.id = ?1
            "#,
            params![card_id],
            |row| {
                Ok((
                    Card {
                        id: row.get(0)?,
                        note_id: row.get(1)?,
                        card_template: row.get(2)?,
                        state: row.get(3)?,
                        due: row.get(4)?,
                        stability: row.get(5)?,
                        difficulty: row.get(6)?,
                        elapsed_days: row.get(7)?,
                        scheduled_days: row.get(8)?,
                        reps: row.get(9)?,
                        lapses: row.get(10)?,
                        last_review: row.get(11)?,
                    },
                    row.get(12)?,
                ))
            },
        )
        .map_err(|_| AppError::NotFound("Không tìm thấy thẻ cần ôn".into()))?;

    let prev_state = card.state;
    let res = schedule_card(&card, rating, desired_retention)?;
    let log_id = Uuid::new_v4().to_string();

    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE cards
         SET state = ?1, due = ?2, stability = ?3, difficulty = ?4,
             elapsed_days = ?5, scheduled_days = ?6, reps = ?7,
             lapses = ?8, last_review = ?9
         WHERE id = ?10",
        params![
            res.new_state,
            res.due_ms,
            res.stability,
            res.difficulty,
            res.elapsed_days,
            res.scheduled_days,
            res.reps,
            res.lapses,
            res.reviewed_at,
            card.id
        ],
    )?;

    tx.execute(
        "INSERT INTO review_logs (id, card_id, rating, state, elapsed_days, scheduled_days, duration_ms, reviewed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            log_id,
            card.id,
            rating,
            prev_state,
            res.elapsed_days,
            res.scheduled_days,
            duration_ms,
            res.reviewed_at
        ],
    )?;
    tx.commit()?;

    Ok(())
}

pub fn get_daily_stats(conn: &Connection) -> AppResult<DailyStats> {
    let today_start = start_of_today_ms();
    let now_ms = Utc::now().timestamp_millis();

    let reviewed_today: i64 = conn.query_row(
        "SELECT COUNT(*) FROM review_logs WHERE reviewed_at >= ?1",
        params![today_start],
        |row| row.get(0),
    )?;

    let new_learned_today: i64 = conn.query_row(
        "SELECT COUNT(*) FROM review_logs WHERE reviewed_at >= ?1 AND state = 0",
        params![today_start],
        |row| row.get(0),
    )?;

    let due_review_remaining: i64 = conn.query_row(
        "SELECT COUNT(*) FROM cards WHERE state != 0 AND due <= ?1",
        params![now_ms],
        |row| row.get(0),
    )?;

    let new_remaining: i64 = conn.query_row(
        "SELECT COALESCE(SUM(MIN(new_count, MAX(0, new_cards_per_day - learned_today))), 0)
         FROM (
             SELECT
                 d.new_cards_per_day,
                 SUM(CASE WHEN c.state = 0 THEN 1 ELSE 0 END) AS new_count,
                 (
                     SELECT COUNT(*) FROM review_logs r
                     JOIN cards c2 ON c2.id = r.card_id
                     JOIN notes n2 ON n2.id = c2.note_id
                     WHERE n2.deck_id = d.id AND r.reviewed_at >= ?1 AND r.state = 0
                 ) AS learned_today
             FROM decks d
             LEFT JOIN notes n ON n.deck_id = d.id
             LEFT JOIN cards c ON c.note_id = n.id
             GROUP BY d.id
         )",
        params![today_start],
        |row| row.get(0),
    )?;

    Ok(DailyStats {
        reviewed_today,
        new_learned_today,
        due_remaining: due_review_remaining + new_remaining,
    })
}

pub fn undo_review(conn: &mut Connection, prev: &Card) -> AppResult<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM review_logs WHERE id = (
            SELECT id FROM review_logs WHERE card_id = ?1 ORDER BY reviewed_at DESC LIMIT 1
         )",
        params![prev.id],
    )?;

    tx.execute(
        "UPDATE cards
         SET state = ?1, due = ?2, stability = ?3, difficulty = ?4,
             elapsed_days = ?5, scheduled_days = ?6, reps = ?7,
             lapses = ?8, last_review = ?9
         WHERE id = ?10",
        params![
            prev.state,
            prev.due,
            prev.stability,
            prev.difficulty,
            prev.elapsed_days,
            prev.scheduled_days,
            prev.reps,
            prev.lapses,
            prev.last_review,
            prev.id
        ],
    )?;
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::deck_repo::{create_deck, list_decks};
    use crate::db::note_repo::create_note;
    use crate::db::open_in_memory_for_test;
    use crate::models::{CreateDeckInput, CreateNoteInput};

    #[test]
    fn study_queue_review_and_daily_stats_flow() {
        let mut conn = open_in_memory_for_test();
        let deck = create_deck(
            &conn,
            CreateDeckInput {
                name: "Daily Study".into(),
                description: None,
                new_cards_per_day: None,
                desired_retention: None,
            },
        )
        .expect("deck");

        for (word, meaning) in [("resilient", "kiên cường"), ("pragmatic", "thực tế")] {
            create_note(
                &mut conn,
                CreateNoteInput {
                    deck_id: deck.id.clone(),
                    note_type: "vocab".into(),
                    word: word.into(),
                    ipa: None,
                    meaning: meaning.into(),
                    example_sentence: None,
                    collocations: None,
                    tags: None,
                },
            )
            .expect("create note");
        }

        let queue = get_study_queue(&conn, &deck.id, 10).expect("queue");
        assert_eq!(queue.len(), 2);

        submit_review(&mut conn, &queue[0].card.id, 3, Some(1500)).expect("review good");
        submit_review(&mut conn, &queue[1].card.id, 1, Some(2500)).expect("review again");

        let stats = get_daily_stats(&conn).expect("daily stats");
        assert_eq!(stats.reviewed_today, 2);
        assert_eq!(stats.new_learned_today, 2);
        assert_eq!(stats.due_remaining, 0);

        let decks = list_decks(&conn).expect("decks");
        assert_eq!(decks[0].new_count, 0);
        assert_eq!(decks[0].total_cards, 2);

        // Undo first card review and verify stats/state revert
        undo_review(&mut conn, &queue[0].card).expect("undo");
        let stats_after_undo = get_daily_stats(&conn).expect("stats after undo");
        assert_eq!(stats_after_undo.reviewed_today, 1);
        assert_eq!(stats_after_undo.new_learned_today, 1);
    }

    #[test]
    fn study_queue_respects_new_cards_per_day_limit() {
        let mut conn = open_in_memory_for_test();
        let deck = create_deck(
            &conn,
            CreateDeckInput {
                name: "Limited Deck".into(),
                description: None,
                new_cards_per_day: Some(1),
                desired_retention: None,
            },
        )
        .expect("deck");

        for w in ["alpha", "beta", "gamma"] {
            create_note(
                &mut conn,
                CreateNoteInput {
                    deck_id: deck.id.clone(),
                    note_type: "vocab".into(),
                    word: w.into(),
                    ipa: None,
                    meaning: w.into(),
                    example_sentence: None,
                    collocations: None,
                    tags: None,
                },
            )
            .expect("note");
        }

        let q1 = get_study_queue(&conn, &deck.id, 50).expect("q1");
        assert_eq!(q1.len(), 1);
        assert_eq!(
            get_daily_stats(&conn).expect("stats before").due_remaining,
            1
        );
        submit_review(&mut conn, &q1[0].card.id, 3, None).expect("review");

        let q2 = get_study_queue(&conn, &deck.id, 50).expect("q2");
        assert_eq!(q2.len(), 0);
        assert_eq!(
            get_daily_stats(&conn).expect("stats after").due_remaining,
            0
        );

        assert!(submit_review(&mut conn, "nonexistent-card", 3, None).is_err());
    }
}
