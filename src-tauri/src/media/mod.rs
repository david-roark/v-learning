use std::fs;
use std::path::PathBuf;

use reqwest::header::USER_AGENT;
use tauri::{AppHandle, Manager};

use crate::dictionary::{self, percent_encode_query};
use crate::error::{AppError, AppResult};
use crate::models::TtsSettings;

const CHROME_UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36";

fn safe_cache_stem(text: &str, scope: &str) -> String {
    let trimmed = text.trim().to_lowercase();
    let prefix: String = trimmed
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .take(40)
        .collect();
    // FNV-1a 64-bit hash over scope + text to avoid collisions and separate caches per provider/voice
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in scope
        .as_bytes()
        .iter()
        .chain(b":")
        .chain(trimmed.as_bytes())
    {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{prefix}_{hash:016x}")
}

fn audio_cache_path(app: &AppHandle, word: &str, scope: &str) -> AppResult<PathBuf> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let audio_dir = app_dir.join("audio");
    fs::create_dir_all(&audio_dir).map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(audio_dir.join(format!("{}.mp3", safe_cache_stem(word, scope))))
}

fn google_tts_url(text: &str, voice: &str) -> String {
    let tl = match voice.trim() {
        "en-GB" | "en-AU" | "en-IN" => voice.trim(),
        _ => "en-US",
    };
    format!(
        "https://translate.google.com/translate_tts?ie=UTF-8&tl={tl}&client=tw-ob&q={}",
        percent_encode_query(text.trim())
    )
}

fn youdao_tts_url(text: &str, voice: &str) -> String {
    let type_code = match voice.trim() {
        "uk" | "en-GB" => "1",
        _ => "2", // 2 = US English
    };
    format!(
        "https://dict.youdao.com/dictvoice?audio={}&type={type_code}",
        percent_encode_query(text.trim())
    )
}

fn build_http_client() -> AppResult<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Internal(format!("Không thể tạo HTTP client: {e}")))
}

async fn fetch_from_provider(
    client: &reqwest::Client,
    provider: &str,
    text: &str,
    audio_url: Option<&str>,
    voice: &str,
) -> Result<Vec<u8>, String> {
    let tts_text: String = text.trim().chars().take(200).collect();
    let url = match provider {
        "dictionary_audio" => {
            if let Some(u) = audio_url.filter(|s| !s.trim().is_empty()) {
                u.to_string()
            } else if text.split_whitespace().count() == 1 {
                dictionary::fetch_online_entry(text)
                    .await
                    .ok()
                    .and_then(|e| e.audio_url)
                    .ok_or_else(|| "Không có file phát âm người thật cho từ này".to_string())?
            } else {
                return Err("Free Dictionary API chỉ hỗ trợ từ đơn".to_string());
            }
        }
        "youdao_tts" => youdao_tts_url(&tts_text, voice),
        "google_tts" => google_tts_url(&tts_text, voice),
        other => return Err(format!("Nguồn TTS không hợp lệ: {other}")),
    };

    let resp = client
        .get(&url)
        .header(USER_AGENT, CHROME_UA)
        .send()
        .await
        .map_err(|e| format!("Lỗi kết nối: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("Lỗi đọc audio: {e}"))?;
    if bytes.len() <= 128 {
        return Err("Dữ liệu audio trống".to_string());
    }
    Ok(bytes.to_vec())
}

fn resolve_provider_voice<'a>(settings: &'a TtsSettings, provider: &str) -> &'a str {
    if let Some(v) = settings.provider_voices.get(provider)
        && !v.trim().is_empty()
    {
        return v.as_str();
    }
    if provider == "google_tts" {
        return if settings.voice.trim().is_empty() {
            "en-US"
        } else {
            settings.voice.as_str()
        };
    }
    if provider == "youdao_tts" {
        return "us";
    }
    ""
}

pub async fn get_or_fetch_audio(
    app: &AppHandle,
    word: &str,
    audio_url: Option<String>,
    settings: &TtsSettings,
) -> AppResult<Vec<u8>> {
    let trimmed = word.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("Không có nội dung để phát âm".into()));
    }

    let order = if settings.fallback_order.is_empty() {
        TtsSettings::default().fallback_order
    } else {
        settings.fallback_order.clone()
    };
    let scope = order
        .iter()
        .map(|p| format!("{p}={}", resolve_provider_voice(settings, p)))
        .collect::<Vec<_>>()
        .join(",");
    let path = audio_cache_path(app, trimmed, &scope)?;
    if path.exists() {
        return fs::read(&path).map_err(|e| AppError::Internal(e.to_string()));
    }

    let client = build_http_client()?;
    for provider in &order {
        if provider == "system_tts" {
            continue;
        }
        let voice = resolve_provider_voice(settings, provider);
        if let Ok(bytes) =
            fetch_from_provider(&client, provider, trimmed, audio_url.as_deref(), voice).await
        {
            fs::write(&path, &bytes).map_err(|e| AppError::Internal(e.to_string()))?;
            return Ok(bytes);
        }
    }

    Err(AppError::NotFound("Không thể tải âm thanh phát âm".into()))
}

pub fn speak_with_os_tts(text: &str, speed: f64) -> AppResult<()> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("Không có nội dung để đọc".into()));
    }

    #[cfg(target_os = "linux")]
    {
        // spd-say rate is -100..100 (0 is normal); espeak wpm ~175 * speed
        let spd_rate = ((speed - 1.0) * 100.0).clamp(-80.0, 80.0) as i32;
        if std::process::Command::new("spd-say")
            .args(["-l", "en", "-r", &spd_rate.to_string(), trimmed])
            .spawn()
            .is_ok()
        {
            return Ok(());
        }
        let wpm = (175.0 * speed).clamp(80.0, 320.0) as i32;
        for bin in ["espeak-ng", "espeak"] {
            if std::process::Command::new(bin)
                .args(["-v", "en", "-s", &wpm.to_string(), trimmed])
                .spawn()
                .is_ok()
            {
                return Ok(());
            }
        }
        return Err(AppError::NotFound(
            "Không tìm thấy giọng máy (cần speech-dispatcher / spd-say hoặc espeak-ng trên Ubuntu)"
                .into(),
        ));
    }

    #[cfg(target_os = "macos")]
    {
        let wpm = (180.0 * speed).clamp(90.0, 320.0) as i32;
        std::process::Command::new("say")
            .args(["-r", &wpm.to_string(), trimmed])
            .spawn()
            .map_err(|e| AppError::Internal(format!("Lỗi gọi giọng máy macOS: {e}")))?;
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let safe = trimmed.replace('\'', " ");
        let rate = ((speed - 1.0) * 5.0).clamp(-5.0, 5.0) as i32;
        let script = format!(
            "Add-Type -AssemblyName System.Speech; $s=New-Object System.Speech.Synthesis.SpeechSynthesizer; $s.Rate={rate}; $s.Speak('{safe}');"
        );
        std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .spawn()
            .map_err(|e| AppError::Internal(format!("Lỗi gọi giọng máy Windows: {e}")))?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err(AppError::NotFound(
        "Hệ điều hành không hỗ trợ giọng máy".into(),
    ))
}

pub async fn preview_provider_audio(provider: &str, voice: &str, text: &str) -> AppResult<Vec<u8>> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("Không có nội dung nghe thử".into()));
    }
    let client = build_http_client()?;
    fetch_from_provider(&client, provider, trimmed, None, voice)
        .await
        .map_err(|err| AppError::NotFound(format!("Nguồn {provider} lỗi: {err}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_cache_stem_bounds_length_and_separates_voices() {
        let long_a = "a ".repeat(100) + "one";
        let stem_joanna = safe_cache_stem(&long_a, "streamelements=Joanna");
        let stem_brian = safe_cache_stem(&long_a, "streamelements=Brian");
        assert!(stem_joanna.len() <= 70);
        assert_ne!(stem_joanna, stem_brian);
    }

    #[test]
    fn tts_urls_encode_query_parameters() {
        let yd = youdao_tts_url("give up", "uk");
        assert!(yd.contains("type=1"));
        assert!(yd.contains("audio=give%20up"));
        let g = google_tts_url("give up", "en-GB");
        assert!(g.contains("tl=en-GB"));
        assert!(g.contains("q=give%20up"));
    }

    #[test]
    fn resolve_provider_voice_and_empty_os_tts_validation() {
        let mut settings = TtsSettings::default();
        assert_eq!(resolve_provider_voice(&settings, "google_tts"), "en-US");
        assert_eq!(resolve_provider_voice(&settings, "youdao_tts"), "us");
        settings
            .provider_voices
            .insert("youdao_tts".into(), "uk".into());
        assert_eq!(resolve_provider_voice(&settings, "youdao_tts"), "uk");
        assert!(speak_with_os_tts("   ", 1.0).is_err());
    }
}
