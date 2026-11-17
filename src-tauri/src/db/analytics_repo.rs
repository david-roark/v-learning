use std::collections::HashMap;

use chrono::{Datelike, Duration, Local, TimeZone, Timelike};
use rusqlite::{Connection, params};

use crate::error::AppResult;
use crate::models::{AnalyticsSummary, ForecastDay, HeatmapDay, LeechWord, RatingCounts};

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

pub fn get_analytics_summary(
    conn: &Connection,
    deck_id: Option<&str>,
) -> AppResult<AnalyticsSummary> {
    let clean_deck = deck_id.map(str::trim).filter(|s| !s.is_empty());

    let total_notes: i64 = match clean_deck {
        Some(d) => conn.query_row(
            "SELECT COUNT(*) FROM notes WHERE deck_id = ?1",
            params![d],
            |r| r.get(0),
        )?,
        None => conn.query_row("SELECT COUNT(*) FROM notes", [], |r| r.get(0))?,
    };

    let (total_cards, new_cards, learning_cards, young_cards, mature_cards): (
        i64,
        i64,
        i64,
        i64,
        i64,
    ) = match clean_deck {
        Some(d) => conn.query_row(
            "SELECT
                COUNT(*),
                COALESCE(SUM(CASE WHEN c.state = 0 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN c.state IN (1, 3) THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN c.state = 2 AND c.scheduled_days < 21 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN c.state = 2 AND c.scheduled_days >= 21 THEN 1 ELSE 0 END), 0)
             FROM cards c
             JOIN notes n ON n.id = c.note_id
             WHERE n.deck_id = ?1",
            params![d],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )?,
        None => conn.query_row(
            "SELECT
                COUNT(*),
                COALESCE(SUM(CASE WHEN state = 0 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN state IN (1, 3) THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN state = 2 AND scheduled_days < 21 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN state = 2 AND scheduled_days >= 21 THEN 1 ELSE 0 END), 0)
             FROM cards",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )?,
    };

    let today = Local::now().date_naive();
    let today_start_ms = Local
        .from_local_datetime(&today.and_hms_opt(0, 0, 0).expect("midnight"))
        .single()
        .map(|dt| dt.timestamp_millis())
        .unwrap_or(0);

    let (
        total_reviews,
        again,
        hard,
        good,
        easy,
        total_study_ms,
        dur_count,
        study_ms_today,
    ): (i64, i64, i64, i64, i64, i64, i64, i64) = match clean_deck {
        Some(d) => conn.query_row(
            "SELECT
                COUNT(*),
                COALESCE(SUM(CASE WHEN r.rating = 1 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN r.rating = 2 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN r.rating = 3 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN r.rating = 4 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(COALESCE(r.duration_ms, 0)), 0),
                COALESCE(SUM(CASE WHEN r.duration_ms IS NOT NULL THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN r.reviewed_at >= ?2 THEN COALESCE(r.duration_ms, 0) ELSE 0 END), 0)
             FROM review_logs r
             JOIN cards c ON c.id = r.card_id
             JOIN notes n ON n.id = c.note_id
             WHERE n.deck_id = ?1",
            params![d, today_start_ms],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get(7)?,
                ))
            },
        )?,
        None => conn.query_row(
            "SELECT
                COUNT(*),
                COALESCE(SUM(CASE WHEN rating = 1 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN rating = 2 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN rating = 3 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN rating = 4 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(COALESCE(duration_ms, 0)), 0),
                COALESCE(SUM(CASE WHEN duration_ms IS NOT NULL THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN reviewed_at >= ?1 THEN COALESCE(duration_ms, 0) ELSE 0 END), 0)
             FROM review_logs",
            params![today_start_ms],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get(7)?,
                ))
            },
        )?,
    };

    let passed_reviews = hard + good + easy;
    let true_retention = if total_reviews > 0 {
        ((passed_reviews as f64 / total_reviews as f64) * 1000.0).round() / 10.0
    } else {
        0.0
    };
    let forget_rate = if total_reviews > 0 {
        ((100.0 - true_retention) * 10.0).round() / 10.0
    } else {
        0.0
    };
    let avg_duration_ms = if dur_count > 0 {
        total_study_ms / dur_count
    } else {
        0
    };

    let start_180 = today - Duration::days(179);
    let start_ms = Local
        .from_local_datetime(&start_180.and_hms_opt(0, 0, 0).expect("midnight"))
        .single()
        .map(|dt| dt.timestamp_millis())
        .unwrap_or(0);

    let mut day_counts: HashMap<String, i64> = HashMap::new();
    let mut hourly_weekday_heatmap = vec![vec![0_i64; 24]; 7];
    let mut hour_totals = [0_i64; 24];
    let mut weekday_totals = [0_i64; 7];

    {
        let mut stmt = match clean_deck {
            Some(_) => conn.prepare(
                "SELECT r.reviewed_at FROM review_logs r
                 JOIN cards c ON c.id = r.card_id
                 JOIN notes n ON n.id = c.note_id
                 WHERE n.deck_id = ?1",
            )?,
            None => conn.prepare("SELECT reviewed_at FROM review_logs")?,
        };
        let mapper = |r: &rusqlite::Row<'_>| r.get::<_, i64>(0);
        let rows = match clean_deck {
            Some(d) => stmt.query_map(params![d], mapper)?,
            None => stmt.query_map([], mapper)?,
        };
        for ts_res in rows {
            let ts = ts_res?;
            if let Some(dt) = Local.timestamp_millis_opt(ts).single() {
                let wd = dt.weekday().num_days_from_monday() as usize;
                let hr = dt.hour() as usize;
                hourly_weekday_heatmap[wd][hr] += 1;
                hour_totals[hr] += 1;
                weekday_totals[wd] += 1;

                if ts >= start_ms {
                    let key = dt.date_naive().format("%Y-%m-%d").to_string();
                    *day_counts.entry(key).or_insert(0) += 1;
                }
            }
        }
    }

    let peak_hour = hour_totals
        .iter()
        .enumerate()
        .filter(|&(_, &c)| c > 0)
        .max_by_key(|&(_, &c)| c)
        .map(|(h, _)| h as i64);
    let peak_weekday = weekday_totals
        .iter()
        .enumerate()
        .filter(|&(_, &c)| c > 0)
        .max_by_key(|&(_, &c)| c)
        .map(|(w, _)| w as i64);

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

        let due_count: i64 = match (i == 0, clean_deck) {
            (true, Some(dk)) => conn.query_row(
                "SELECT COUNT(*) FROM cards c
                 JOIN notes n ON n.id = c.note_id
                 WHERE n.deck_id = ?1 AND c.state != 0 AND c.due <= ?2",
                params![dk, day_end],
                |r| r.get(0),
            )?,
            (true, None) => conn.query_row(
                "SELECT COUNT(*) FROM cards WHERE state != 0 AND due <= ?1",
                params![day_end],
                |r| r.get(0),
            )?,
            (false, Some(dk)) => conn.query_row(
                "SELECT COUNT(*) FROM cards c
                 JOIN notes n ON n.id = c.note_id
                 WHERE n.deck_id = ?1 AND c.state != 0 AND c.due >= ?2 AND c.due <= ?3",
                params![dk, day_start, day_end],
                |r| r.get(0),
            )?,
            (false, None) => conn.query_row(
                "SELECT COUNT(*) FROM cards WHERE state != 0 AND due >= ?1 AND due <= ?2",
                params![day_start, day_end],
                |r| r.get(0),
            )?,
        };

        forecast_7d.push(ForecastDay {
            date: d.format("%d/%m").to_string(),
            due_count,
        });
    }

    let leech_words = {
        let mut stmt = match clean_deck {
            Some(_) => conn.prepare(
                "SELECT n.word, n.meaning, c.lapses, c.reps
                 FROM cards c
                 JOIN notes n ON n.id = c.note_id
                 WHERE n.deck_id = ?1 AND c.lapses > 0
                 ORDER BY c.lapses DESC, c.reps DESC
                 LIMIT 8",
            )?,
            None => conn.prepare(
                "SELECT n.word, n.meaning, c.lapses, c.reps
                 FROM cards c
                 JOIN notes n ON n.id = c.note_id
                 WHERE c.lapses > 0
                 ORDER BY c.lapses DESC, c.reps DESC
                 LIMIT 8",
            )?,
        };
        let leech_mapper = |r: &rusqlite::Row<'_>| {
            Ok(LeechWord {
                word: r.get(0)?,
                meaning: r.get(1)?,
                lapses: r.get(2)?,
                reps: r.get(3)?,
            })
        };
        let rows = match clean_deck {
            Some(d) => stmt.query_map(params![d], leech_mapper)?,
            None => stmt.query_map([], leech_mapper)?,
        };
        rows.collect::<Result<Vec<_>, _>>()?
    };

    Ok(AnalyticsSummary {
        streak_days,
        total_reviews,
        true_retention,
        forget_rate,
        mature_cards,
        total_notes,
        total_cards,
        new_cards,
        learning_cards,
        young_cards,
        rating_counts: RatingCounts {
            again,
            hard,
            good,
            easy,
        },
        total_study_ms,
        avg_duration_ms,
        study_ms_today,
        hourly_weekday_heatmap,
        peak_hour,
        peak_weekday,
        leech_words,
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
                deck_id: deck.id.clone(),
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
        submit_review(&mut conn, &card_id, 3, Some(1200)).expect("review good");
        submit_review(&mut conn, &card_id, 1, Some(800)).expect("review again");

        let summary = get_analytics_summary(&conn, None).expect("summary");
        assert_eq!(summary.total_notes, 1);
        assert_eq!(summary.total_cards, 1);
        assert_eq!(summary.total_reviews, 2);
        assert_eq!(summary.true_retention, 50.0);
        assert_eq!(summary.forget_rate, 50.0);
        assert_eq!(summary.rating_counts.good, 1);
        assert_eq!(summary.rating_counts.again, 1);
        assert_eq!(summary.total_study_ms, 2000);
        assert_eq!(summary.avg_duration_ms, 1000);
        assert_eq!(summary.streak_days, 1);
        assert_eq!(summary.heatmap.len(), 180);
        assert_eq!(summary.forecast_7d.len(), 14);
        assert_eq!(summary.hourly_weekday_heatmap.len(), 7);
        assert_eq!(summary.hourly_weekday_heatmap[0].len(), 24);
        assert!(summary.peak_hour.is_some());
        assert!(summary.peak_weekday.is_some());
        assert_eq!(summary.leech_words.len(), 1);
        assert_eq!(summary.leech_words[0].word, "eloquent");

        // Deck filter for an empty deck returns zeroes
        let empty_deck = create_deck(
            &conn,
            CreateDeckInput {
                name: "Empty".into(),
                description: None,
                new_cards_per_day: None,
                desired_retention: None,
            },
        )
        .expect("empty");
        let filtered = get_analytics_summary(&conn, Some(&empty_deck.id)).expect("filtered");
        assert_eq!(filtered.total_notes, 0);
        assert_eq!(filtered.total_reviews, 0);
        assert!(filtered.peak_hour.is_none());
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
