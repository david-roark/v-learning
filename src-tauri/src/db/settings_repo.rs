use rusqlite::{Connection, OptionalExtension, params};

use crate::error::AppResult;
use crate::models::AiSettings;

fn get_key(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    let val = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = ?1",
            params![key],
            |r| r.get(0),
        )
        .optional()?;
    Ok(val)
}

fn set_key(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn get_ai_settings(conn: &Connection) -> AppResult<AiSettings> {
    Ok(AiSettings {
        base_url: get_key(conn, "ai_base_url")?
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
        api_key: get_key(conn, "ai_api_key")?.unwrap_or_default(),
        model: get_key(conn, "ai_model")?.unwrap_or_else(|| "gpt-4o-mini".to_string()),
    })
}

pub fn save_ai_settings(conn: &Connection, settings: &AiSettings) -> AppResult<()> {
    set_key(conn, "ai_base_url", settings.base_url.trim())?;
    set_key(conn, "ai_api_key", settings.api_key.trim())?;
    set_key(conn, "ai_model", settings.model.trim())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory_for_test;

    #[test]
    fn ai_settings_defaults_and_save_roundtrip() {
        let conn = open_in_memory_for_test();
        let defaults = get_ai_settings(&conn).expect("defaults");
        assert_eq!(defaults.model, "gpt-4o-mini");
        assert!(defaults.api_key.is_empty());

        save_ai_settings(
            &conn,
            &AiSettings {
                base_url: " https://generativelanguage.googleapis.com/v1beta/openai ".into(),
                api_key: " AIzaTestKey ".into(),
                model: " gemini-2.5-flash ".into(),
            },
        )
        .expect("save");

        let loaded = get_ai_settings(&conn).expect("loaded");
        assert_eq!(loaded.base_url, "https://generativelanguage.googleapis.com/v1beta/openai");
        assert_eq!(loaded.api_key, "AIzaTestKey");
        assert_eq!(loaded.model, "gemini-2.5-flash");
    }
}
