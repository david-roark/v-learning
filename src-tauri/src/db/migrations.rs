use rusqlite::Connection;

use crate::error::AppResult;

const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS decks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    new_cards_per_day INTEGER NOT NULL DEFAULT 20,
    desired_retention REAL NOT NULL DEFAULT 0.90,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS notes (
    id TEXT PRIMARY KEY,
    deck_id TEXT NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
    note_type TEXT NOT NULL,
    word TEXT NOT NULL,
    ipa TEXT,
    meaning TEXT NOT NULL,
    example_sentence TEXT,
    collocations TEXT NOT NULL DEFAULT '[]',
    audio_path TEXT,
    image_path TEXT,
    tags TEXT NOT NULL DEFAULT '[]',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS cards (
    id TEXT PRIMARY KEY,
    note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    card_template TEXT NOT NULL,
    state INTEGER NOT NULL DEFAULT 0,
    due INTEGER NOT NULL,
    stability REAL NOT NULL DEFAULT 0,
    difficulty REAL NOT NULL DEFAULT 0,
    elapsed_days INTEGER NOT NULL DEFAULT 0,
    scheduled_days INTEGER NOT NULL DEFAULT 0,
    reps INTEGER NOT NULL DEFAULT 0,
    lapses INTEGER NOT NULL DEFAULT 0,
    last_review INTEGER
);

CREATE TABLE IF NOT EXISTS review_logs (
    id TEXT PRIMARY KEY,
    card_id TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    rating INTEGER NOT NULL,
    state INTEGER NOT NULL,
    elapsed_days INTEGER NOT NULL,
    scheduled_days INTEGER NOT NULL,
    duration_ms INTEGER,
    reviewed_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_notes_deck_id ON notes(deck_id);
CREATE INDEX IF NOT EXISTS idx_cards_due_state ON cards(due, state);
CREATE INDEX IF NOT EXISTS idx_cards_note_id ON cards(note_id);
CREATE INDEX IF NOT EXISTS idx_review_logs_reviewed_at ON review_logs(reviewed_at);
"#;

pub fn run_migrations(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(SCHEMA_SQL)?;
    Ok(())
}
