use rusqlite::{Connection, OptionalExtension, params};

use crate::error::AppResult;
use crate::models::{AiSettings, GoogleDriveSettings, TtsSettings, WebDavSettings};

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

pub fn get_google_drive_settings(conn: &Connection) -> AppResult<GoogleDriveSettings> {
    let client_id = get_key(conn, "gdrive_client_id")?.unwrap_or_default();
    let client_secret = get_key(conn, "gdrive_client_secret")?.unwrap_or_default();
    let access_token = get_key(conn, "gdrive_access_token")?.unwrap_or_default();
    let refresh_token = get_key(conn, "gdrive_refresh_token")?.unwrap_or_default();
    let expires_at = get_key(conn, "gdrive_expires_at")?
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    let user_email = get_key(conn, "gdrive_user_email")?.unwrap_or_default();
    let last_synced_remote_modified =
        get_key(conn, "gdrive_last_synced_remote_modified")?.unwrap_or_default();
    let auto_sync = get_key(conn, "gdrive_auto_sync")?
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let local_dirty = get_key(conn, "gdrive_local_dirty")?
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    Ok(GoogleDriveSettings {
        client_id,
        client_secret,
        access_token,
        refresh_token,
        expires_at,
        user_email,
        last_synced_remote_modified,
        auto_sync,
        local_dirty,
    })
}

pub fn set_google_drive_local_dirty(conn: &Connection, dirty: bool) -> AppResult<()> {
    set_key(conn, "gdrive_local_dirty", if dirty { "1" } else { "0" })
}

pub fn save_google_drive_settings(
    conn: &Connection,
    settings: &GoogleDriveSettings,
) -> AppResult<()> {
    set_key(conn, "gdrive_client_id", settings.client_id.trim())?;
    set_key(conn, "gdrive_client_secret", settings.client_secret.trim())?;
    set_key(conn, "gdrive_access_token", settings.access_token.trim())?;
    set_key(conn, "gdrive_refresh_token", settings.refresh_token.trim())?;
    set_key(conn, "gdrive_expires_at", &settings.expires_at.to_string())?;
    set_key(conn, "gdrive_user_email", settings.user_email.trim())?;
    set_key(
        conn,
        "gdrive_last_synced_remote_modified",
        settings.last_synced_remote_modified.trim(),
    )?;
    set_key(
        conn,
        "gdrive_auto_sync",
        if settings.auto_sync { "1" } else { "0" },
    )?;
    set_key(
        conn,
        "gdrive_local_dirty",
        if settings.local_dirty { "1" } else { "0" },
    )?;
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

    #[test]
    fn webdav_settings_defaults_and_save_roundtrip() {
        let conn = open_in_memory_for_test();
        let defaults = get_webdav_settings(&conn).expect("defaults");
        assert!(defaults.url.is_empty());

        save_webdav_settings(
            &conn,
            &WebDavSettings {
                url: " https://dav.example.com/files/ ".into(),
                username: " alice ".into(),
                password: "secret".into(),
            },
        )
        .expect("save webdav");

        let loaded = get_webdav_settings(&conn).expect("loaded");
        assert_eq!(loaded.url, "https://dav.example.com/files/");
        assert_eq!(loaded.username, "alice");
        assert_eq!(loaded.password, "secret");
    }

    #[test]
    fn google_drive_settings_defaults_and_roundtrip() {
        let conn = open_in_memory_for_test();
        let defaults = get_google_drive_settings(&conn).expect("defaults");
        assert!(defaults.client_id.is_empty());
        assert!(defaults.refresh_token.is_empty());
        assert_eq!(defaults.expires_at, 0);

        save_google_drive_settings(
            &conn,
            &GoogleDriveSettings {
                client_id: " 123.apps.googleusercontent.com ".into(),
                client_secret: " sec ".into(),
                access_token: " ya29.abc ".into(),
                refresh_token: " 1//0gxyz ".into(),
                expires_at: 1_750_000_000,
                user_email: " user@example.com ".into(),
                last_synced_remote_modified: " 2026-11-18T12:00:00Z ".into(),
                auto_sync: true,
                local_dirty: true,
            },
        )
        .expect("save gdrive");

        let loaded = get_google_drive_settings(&conn).expect("loaded");
        assert_eq!(loaded.client_id, "123.apps.googleusercontent.com");
        assert_eq!(loaded.client_secret, "sec");
        assert_eq!(loaded.access_token, "ya29.abc");
        assert_eq!(loaded.refresh_token, "1//0gxyz");
        assert_eq!(loaded.expires_at, 1_750_000_000);
        assert_eq!(loaded.user_email, "user@example.com");
        assert_eq!(loaded.last_synced_remote_modified, "2026-11-18T12:00:00Z");
        assert!(loaded.auto_sync);
        assert!(loaded.local_dirty);
    }
}
