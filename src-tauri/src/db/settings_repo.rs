use rusqlite::{Connection, OptionalExtension, params};

use crate::error::AppResult;
use crate::models::{AiSettings, TtsSettings, WebDavSettings};

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

pub fn get_webdav_settings(conn: &Connection) -> AppResult<WebDavSettings> {
    Ok(WebDavSettings {
        url: get_key(conn, "webdav_url")?.unwrap_or_default(),
        username: get_key(conn, "webdav_username")?.unwrap_or_default(),
        password: get_key(conn, "webdav_password")?.unwrap_or_default(),
    })
}

pub fn save_webdav_settings(conn: &Connection, settings: &WebDavSettings) -> AppResult<()> {
    set_key(conn, "webdav_url", settings.url.trim())?;
    set_key(conn, "webdav_username", settings.username.trim())?;
    set_key(conn, "webdav_password", settings.password.trim())?;
    Ok(())
}

pub fn get_tts_settings(conn: &Connection) -> AppResult<TtsSettings> {
    let default = TtsSettings::default();
    let order = get_key(conn, "tts_fallback_order")?
        .and_then(|raw| serde_json::from_str::<Vec<String>>(&raw).ok())
        .filter(|v| !v.is_empty())
        .unwrap_or(default.fallback_order);
    let voice = get_key(conn, "tts_voice")?
        .filter(|v| !v.trim().is_empty())
        .unwrap_or(default.voice);
    let mut provider_voices = default.provider_voices;
    if let Some(raw) = get_key(conn, "tts_provider_voices")?
        && let Ok(parsed) = serde_json::from_str::<std::collections::HashMap<String, String>>(&raw)
    {
        for (k, v) in parsed {
            if !v.trim().is_empty() {
                provider_voices.insert(k, v);
            }
        }
    }
    Ok(TtsSettings {
        fallback_order: order,
        voice,
        provider_voices,
    })
}

pub fn save_tts_settings(conn: &Connection, settings: &TtsSettings) -> AppResult<()> {
    let clean_order: Vec<String> = settings
        .fallback_order
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let json_order = serde_json::to_string(&clean_order)
        .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
    let json_voices = serde_json::to_string(&settings.provider_voices)
        .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
    set_key(conn, "tts_fallback_order", &json_order)?;
    set_key(conn, "tts_voice", settings.voice.trim())?;
    set_key(conn, "tts_provider_voices", &json_voices)?;
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
        assert_eq!(
            loaded.base_url,
            "https://generativelanguage.googleapis.com/v1beta/openai"
        );
        assert_eq!(loaded.api_key, "AIzaTestKey");
        assert_eq!(loaded.model, "gemini-2.5-flash");
    }

    #[test]
    fn tts_settings_defaults_and_save_roundtrip() {
        let conn = open_in_memory_for_test();
        let defaults = get_tts_settings(&conn).expect("defaults");
        assert_eq!(
            defaults.fallback_order,
            vec!["google_tts", "youdao_tts", "system_tts"]
        );
        assert_eq!(defaults.voice, "en-US");

        save_tts_settings(
            &conn,
            &TtsSettings {
                fallback_order: vec!["google_tts".into(), "youdao_tts".into()],
                voice: " en-GB ".into(),
                provider_voices: std::collections::HashMap::from([(
                    "youdao_tts".into(),
                    "uk".into(),
                )]),
            },
        )
        .expect("save tts");

        let loaded = get_tts_settings(&conn).expect("loaded tts");
        assert_eq!(loaded.fallback_order, vec!["google_tts", "youdao_tts"]);
        assert_eq!(loaded.voice, "en-GB");
        assert_eq!(
            loaded.provider_voices.get("youdao_tts").map(String::as_str),
            Some("uk")
        );
    }
}
