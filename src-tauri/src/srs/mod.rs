use chrono::Utc;
use fsrs::{FSRS, ItemState, MemoryState};

use crate::error::{AppError, AppResult};
use crate::models::{Card, IntervalPreview};

const MS_PER_DAY: f64 = 86_400_000.0;

#[derive(Debug, Clone, Copy)]
pub struct ScheduleResult {
    pub new_state: i64,
    pub due_ms: i64,
    pub stability: f64,
    pub difficulty: f64,
    pub elapsed_days: i64,
    pub scheduled_days: i64,
    pub reps: i64,
    pub lapses: i64,
    pub reviewed_at: i64,
}

fn memory_state_from_card(card: &Card) -> Option<MemoryState> {
    if card.reps == 0 || card.stability <= 0.0 || card.difficulty <= 0.0 {
        return None;
    }
    Some(MemoryState {
        stability: card.stability as f32,
        difficulty: card.difficulty as f32,
        stability_fast: card.stability as f32,
    })
}

fn elapsed_days_since_last(card: &Card, now_ms: i64) -> u32 {
    match card.last_review {
        Some(last) if now_ms > last => ((now_ms - last) as f64 / MS_PER_DAY).floor() as u32,
        _ => 0,
    }
}

pub fn format_interval_days(days: f32) -> String {
    let rounded = days.round().max(1.0) as i64;
    if rounded == 1 {
        "1 ngày".to_string()
    } else if rounded < 30 {
        format!("{rounded} ngày")
    } else if rounded < 365 {
        let months = (rounded as f64 / 30.0).round() as i64;
        format!("{months} tháng")
    } else {
        let years = (rounded as f64 / 365.0 * 10.0).round() / 10.0;
        format!("{years} năm")
    }
}

pub fn preview_intervals(card: &Card, desired_retention: f64) -> AppResult<IntervalPreview> {
    let fsrs = FSRS::default();
    let now_ms = Utc::now().timestamp_millis();
    let prev = memory_state_from_card(card);
    let elapsed = elapsed_days_since_last(card, now_ms);

    let next = fsrs
        .next_states(prev, desired_retention as f32, elapsed)
        .map_err(|e| AppError::Srs(e.to_string()))?;

    Ok(IntervalPreview {
        again: "10 phút".to_string(),
        hard: format_interval_days(next.hard.interval),
        good: format_interval_days(next.good.interval),
        easy: format_interval_days(next.easy.interval),
    })
}

pub fn schedule_card(card: &Card, rating: i64, desired_retention: f64) -> AppResult<ScheduleResult> {
    if !(1..=4).contains(&rating) {
        return Err(AppError::Validation(format!("Invalid rating: {rating}")));
    }

    let fsrs = FSRS::default();
    let now_ms = Utc::now().timestamp_millis();
    let prev = memory_state_from_card(card);
    let elapsed = elapsed_days_since_last(card, now_ms);

    let next = fsrs
        .next_states(prev, desired_retention as f32, elapsed)
        .map_err(|e| AppError::Srs(e.to_string()))?;

    let chosen: ItemState = match rating {
        1 => next.again,
        2 => next.hard,
        3 => next.good,
        4 => next.easy,
        _ => unreachable!(),
    };

    let (new_state, due_ms, scheduled_days, lapses) = if rating == 1 {
        // Again: short-term relearning (10 minutes)
        let ten_min_ms = 10 * 60 * 1000;
        let next_state = if card.state == 0 { 1 } else { 3 };
        (next_state, now_ms + ten_min_ms, 0, card.lapses + 1)
    } else {
        let days = chosen.interval.round().max(1.0) as i64;
        let due = now_ms + (days * MS_PER_DAY as i64);
        (2, due, days, card.lapses)
    };

    Ok(ScheduleResult {
        new_state,
        due_ms,
        stability: chosen.memory.stability as f64,
        difficulty: chosen.memory.difficulty as f64,
        elapsed_days: elapsed as i64,
        scheduled_days,
        reps: card.reps + 1,
        lapses,
        reviewed_at: now_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_new_card() -> Card {
        Card {
            id: "c1".into(),
            note_id: "n1".into(),
            card_template: "recognition".into(),
            state: 0,
            due: 0,
            stability: 0.0,
            difficulty: 0.0,
            elapsed_days: 0,
            scheduled_days: 0,
            reps: 0,
            lapses: 0,
            last_review: None,
        }
    }

    #[test]
    fn preview_intervals_returns_non_empty_labels() {
        let card = sample_new_card();
        let preview = preview_intervals(&card, 0.90).expect("preview");
        assert_eq!(preview.again, "10 phút");
        assert!(!preview.hard.is_empty());
        assert!(!preview.good.is_empty());
        assert!(!preview.easy.is_empty());
    }

    #[test]
    fn schedule_good_advances_new_card_to_review() {
        let card = sample_new_card();
        let res = schedule_card(&card, 3, 0.90).expect("schedule good");
        assert_eq!(res.new_state, 2);
        assert_eq!(res.reps, 1);
        assert_eq!(res.lapses, 0);
        assert!(res.scheduled_days >= 1);
        assert!(res.stability > 0.0);
        assert!(res.difficulty > 0.0);
        assert!(res.due_ms > res.reviewed_at);
    }

    #[test]
    fn schedule_again_sets_relearning_and_increments_lapses() {
        let mut card = sample_new_card();
        let res_new = schedule_card(&card, 1, 0.90).expect("again on new");
        assert_eq!(res_new.new_state, 1);
        assert_eq!(res_new.lapses, 1);
        assert_eq!(res_new.scheduled_days, 0);

        card.state = 2;
        card.reps = 2;
        card.stability = 5.0;
        card.difficulty = 5.0;
        let res_review = schedule_card(&card, 1, 0.90).expect("again on review");
        assert_eq!(res_review.new_state, 3);
        assert_eq!(res_review.lapses, 1);
    }

    #[test]
    fn schedule_rejects_invalid_rating() {
        let card = sample_new_card();
        assert!(schedule_card(&card, 0, 0.90).is_err());
        assert!(schedule_card(&card, 5, 0.90).is_err());
    }
}
