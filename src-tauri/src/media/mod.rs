use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};

fn audio_cache_path(app: &AppHandle, word: &str) -> AppResult<PathBuf> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let audio_dir = app_dir.join("audio");
    fs::create_dir_all(&audio_dir).map_err(|e| AppError::Internal(e.to_string()))?;

    let safe_name: String = word
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();

    Ok(audio_dir.join(format!("{safe_name}.mp3")))
}

pub async fn get_or_fetch_audio(
    app: &AppHandle,
    word: &str,
    audio_url: Option<String>,
) -> AppResult<Vec<u8>> {
    let path = audio_cache_path(app, word)?;
    if path.exists() {
        return fs::read(&path).map_err(|e| AppError::Internal(e.to_string()));
    }

    let url = audio_url.ok_or_else(|| AppError::NotFound("Không có audio URL để tải".into()))?;
    let bytes = reqwest::get(&url)
        .await
        .map_err(|e| AppError::Internal(format!("Lỗi tải audio: {e}")))?
        .bytes()
        .await
        .map_err(|e| AppError::Internal(format!("Lỗi đọc audio: {e}")))?;

    fs::write(&path, &bytes).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(bytes.to_vec())
}
