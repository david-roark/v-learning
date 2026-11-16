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

CREATE TABLE IF NOT EXISTS dictionary_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    word TEXT NOT NULL UNIQUE COLLATE NOCASE,
    ipa TEXT,
    pos TEXT,
    meaning_vi TEXT,
    meaning_en TEXT,
    example_sentence TEXT,
    audio_url TEXT,
    source TEXT NOT NULL DEFAULT 'local'
);

CREATE VIRTUAL TABLE IF NOT EXISTS dictionary_fts USING fts5(
    word,
    meaning_vi,
    meaning_en,
    content='dictionary_entries',
    content_rowid='id'
);

CREATE TRIGGER IF NOT EXISTS dict_ai AFTER INSERT ON dictionary_entries BEGIN
    INSERT INTO dictionary_fts(rowid, word, meaning_vi, meaning_en)
    VALUES (new.id, new.word, COALESCE(new.meaning_vi, ''), COALESCE(new.meaning_en, ''));
END;

CREATE TRIGGER IF NOT EXISTS dict_ad AFTER DELETE ON dictionary_entries BEGIN
    INSERT INTO dictionary_fts(dictionary_fts, rowid, word, meaning_vi, meaning_en)
    VALUES ('delete', old.id, old.word, COALESCE(old.meaning_vi, ''), COALESCE(old.meaning_en, ''));
END;

CREATE TRIGGER IF NOT EXISTS dict_au AFTER UPDATE ON dictionary_entries BEGIN
    INSERT INTO dictionary_fts(dictionary_fts, rowid, word, meaning_vi, meaning_en)
    VALUES ('delete', old.id, old.word, COALESCE(old.meaning_vi, ''), COALESCE(old.meaning_en, ''));
    INSERT INTO dictionary_fts(rowid, word, meaning_vi, meaning_en)
    VALUES (new.id, new.word, COALESCE(new.meaning_vi, ''), COALESCE(new.meaning_en, ''));
END;

CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

const STARTER_DICT: &[(&str, &str, &str, &str, &str)] = &[
    (
        "ubiquitous",
        "/juːˈbɪk.wɪ.təs/",
        "adj",
        "có mặt ở khắp mọi nơi",
        "Smartphones have become ubiquitous in modern life.",
    ),
    (
        "pragmatic",
        "/præɡˈmæt.ɪk/",
        "adj",
        "thực tế, thực dụng",
        "We need a pragmatic approach to solve this issue.",
    ),
    (
        "resilient",
        "/rɪˈzɪl.i.ənt/",
        "adj",
        "kiên cường, mau phục hồi",
        "Children are often remarkably resilient.",
    ),
    (
        "ambiguous",
        "/æmˈbɪɡ.ju.əs/",
        "adj",
        "mơ hồ, đa nghĩa",
        "The wording of the contract is somewhat ambiguous.",
    ),
    (
        "eloquent",
        "/ˈel.ə.kwənt/",
        "adj",
        "hùng hồn, có sức thuyết phục",
        "She gave an eloquent speech about climate change.",
    ),
    (
        "ephemeral",
        "/ɪˈfem.ər.əl/",
        "adj",
        "phù du, ngắn ngủi",
        "Fame in the digital age is often ephemeral.",
    ),
    (
        "meticulous",
        "/məˈtɪk.jə.ləs/",
        "adj",
        "tỉ mỉ, kỹ lưỡng",
        "He kept meticulous records of every transaction.",
    ),
    (
        "inevitable",
        "/ɪˈnev.ɪ.tə.bəl/",
        "adj",
        "không thể tránh khỏi",
        "Change is an inevitable part of growth.",
    ),
];

pub fn run_migrations(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(SCHEMA_SQL)?;

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM dictionary_entries", [], |r| r.get(0))?;
    if count == 0 {
        for (word, ipa, pos, meaning_vi, example) in STARTER_DICT {
            conn.execute(
                "INSERT OR IGNORE INTO dictionary_entries (word, ipa, pos, meaning_vi, example_sentence, source)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'local')",
                rusqlite::params![word, ipa, pos, meaning_vi, example],
            )?;
        }
    }
    Ok(())
}
