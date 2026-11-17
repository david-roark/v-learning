use std::collections::HashMap;

use chrono::{Duration, Local, TimeZone};
use rusqlite::{Connection, params};

use crate::error::AppResult;
use crate::models::{AnalyticsSummary, ForecastDay, HeatmapDay};

fn compute_streak(day_counts: &HashMap<String, i64>) -> i64 {
    let today = Local::now().date_naive();
    let mut streak = 0_i64;
    let mut cursor = today;

    // If user hasn't reviewed today yet, allow streak to continue from yesterday
    let today_str = today.format("%Y-%m-%d").to_string();
    if day_counts.get(&today_str).copied().unwrap_or(0) == 0 {
        cursor = today - Duration::days(1);
    }

    loop {
        let key = cursor.format("%Y-%m-%d").to_string();
        if day_counts.get(&key).copied().unwrap_or(0) > 0 {
            streak += 1;
            cursor -= Duration::days(1);
        } else {
            break;
        }
    }
    streak
}

pub fn get_analytics_summary(conn: &Connection) -> AppResult<AnalyticsSummary> {
    let total_reviews: i64 =
        conn.query_row("SELECT COUNT(*) FROM review_logs", [], |r| r.get(0))?;

    let passed_reviews: i64 = conn.query_row(
        "SELECT COUNT(*) FROM review_logs WHERE rating >= 2",
        [],
        |r| r.get(0),
    )?;

    let true_retention = if total_reviews > 0 {
        ((passed_reviews as f64 / total_reviews as f64) * 1000.0).round() / 10.0
    } else {
        0.0
    };

    let mature_cards: i64 = conn.query_row(
        "SELECT COUNT(*) FROM cards WHERE state = 2 AND scheduled_days >= 21",
        [],
        |r| r.get(0),
    )?;

    let today = Local::now().date_naive();
    let start_180 = today - Duration::days(179);
    let start_ms = Local
        .from_local_datetime(&start_180.and_hms_opt(0, 0, 0).expect("midnight"))
        .single()
        .map(|dt| dt.timestamp_millis())
        .unwrap_or(0);

    let mut day_counts: HashMap<String, i64> = HashMap::new();
    {
        let mut stmt =
            conn.prepare("SELECT reviewed_at FROM review_logs WHERE reviewed_at >= ?1")?;
        let rows = stmt.query_map(params![start_ms], |r| r.get::<_, i64>(0))?;
        for ts_res in rows {
            let ts = ts_res?;
            if let Some(dt) = Local.timestamp_millis_opt(ts).single() {
                let key = dt.date_naive().format("%Y-%m-%d").to_string();
                *day_counts.entry(key).or_insert(0) += 1;
            }
        }
    }

    let streak_days = compute_streak(&day_counts);

    let mut heatmap = Vec::with_capacity(180);
    for i in 0..180 {
        let d = start_180 + Duration::days(i);
        let key = d.format("%Y-%m-%d").to_string();
        let count = day_counts.get(&key).copied().unwrap_or(0);
        heatmap.push(HeatmapDay { date: key, count });
    }

    let mut forecast_7d = Vec::with_capacity(14);
    for i in 0..14 {
        let d = today + Duration::days(i);
        let day_start = Local
            .from_local_datetime(&d.and_hms_opt(0, 0, 0).expect("midnight"))
            .single()
            .map(|dt| dt.timestamp_millis())
            .unwrap_or(0);
        let day_end = Local
            .from_local_datetime(&d.and_hms_opt(23, 59, 59).expect("eod"))
            .single()
            .map(|dt| dt.timestamp_millis() + 999)
            .unwrap_or(i64::MAX);

        let due_count: i64 = if i == 0 {
            conn.query_row(
                "SELECT COUNT(*) FROM cards WHERE state != 0 AND due <= ?1",
                params![day_end],
                |r| r.get(0),
            )?
        } else {
            conn.query_row(
                "SELECT COUNT(*) FROM cards WHERE state != 0 AND due >= ?1 AND due <= ?2",
                params![day_start, day_end],
                |r| r.get(0),
            )?
        };

        forecast_7d.push(ForecastDay {
            date: d.format("%d/%m").to_string(),
            due_count,
        });
    }

    Ok(AnalyticsSummary {
        streak_days,
        total_reviews,
        true_retention,
        mature_cards,
        heatmap,
        forecast_7d,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::card_repo::submit_review;
    use crate::db::deck_repo::create_deck;
    use crate::db::note_repo::create_note;
    use crate::db::open_in_memory_for_test;
    use crate::models::{CreateDeckInput, CreateNoteInput};

    #[test]
    fn analytics_summary_computes_streak_retention_and_forecast() {
        let mut conn = open_in_memory_for_test();
        let deck = create_deck(
            &conn,
            CreateDeckInput {
                name: "Analytics Deck".into(),
                description: None,
                new_cards_per_day: None,
                desired_retention: None,
            },
        )
        .expect("deck");

        create_note(
            &mut conn,
            CreateNoteInput {
                deck_id: deck.id,
                note_type: "vocab".into(),
                word: "eloquent".into(),
                ipa: None,
                meaning: "hùng hồn".into(),
                example_sentence: None,
                collocations: None,
                tags: None,
            },
        )
        .expect("note");

        let card_id: String = conn
            .query_row("SELECT id FROM cards LIMIT 1", [], |r| r.get(0))
            .expect("card");
        submit_review(&mut conn, &card_id, 3, Some(1200)).expect("review");

        let summary = get_analytics_summary(&conn).expect("summary");
        assert_eq!(summary.total_reviews, 1);
        assert_eq!(summary.true_retention, 100.0);
        assert_eq!(summary.streak_days, 1);
        assert_eq!(summary.heatmap.len(), 180);
        assert_eq!(summary.forecast_7d.len(), 14);
    }

    #[test]
    fn compute_streak_continues_from_yesterday_and_stops_on_gap() {
        let today = Local::now().date_naive();
        let yesterday = (today - Duration::days(1)).format("%Y-%m-%d").to_string();
        let two_days_ago = (today - Duration::days(2)).format("%Y-%m-%d").to_string();
        let four_days_ago = (today - Duration::days(4)).format("%Y-%m-%d").to_string();

        let counts = HashMap::from([
            (yesterday, 3),
            (two_days_ago, 1),
            // gap on today - 3 days
            (four_days_ago, 5),
        ]);
        assert_eq!(compute_streak(&counts), 2);
        assert_eq!(compute_streak(&HashMap::new()), 0);
    }
}
