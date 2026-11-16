pub mod importer;

use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::models::UpsertDictionaryInput;

#[derive(Debug, Deserialize)]
struct ApiEntry {
    word: String,
    phonetic: Option<String>,
    #[serde(default)]
    phonetics: Vec<ApiPhonetic>,
    #[serde(default)]
    meanings: Vec<ApiMeaning>,
}

#[derive(Debug, Deserialize)]
struct ApiPhonetic {
    text: Option<String>,
    audio: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiMeaning {
    #[serde(rename = "partOfSpeech")]
    part_of_speech: Option<String>,
    #[serde(default)]
    definitions: Vec<ApiDefinition>,
}

#[derive(Debug, Deserialize)]
struct ApiDefinition {
    definition: Option<String>,
    example: Option<String>,
}

pub fn parse_api_response(json_str: &str) -> AppResult<UpsertDictionaryInput> {
    let entries: Vec<ApiEntry> = serde_json::from_str(json_str)
        .map_err(|e| AppError::Internal(format!("Invalid dictionary JSON: {e}")))?;

    let first = entries
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound("Không tìm thấy từ trong từ điển online".into()))?;

    let ipa = first
        .phonetic
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            first
                .phonetics
                .iter()
                .find_map(|p| p.text.as_ref().filter(|s| !s.trim().is_empty()).cloned())
        });

    let audio_url = first
        .phonetics
        .iter()
        .find_map(|p| p.audio.as_ref().filter(|s| !s.trim().is_empty()).cloned());

    let first_meaning = first.meanings.into_iter().next();
    let pos = first_meaning.as_ref().and_then(|m| m.part_of_speech.clone());
    let first_def = first_meaning.and_then(|m| m.definitions.into_iter().next());

    let meaning_en = first_def.as_ref().and_then(|d| d.definition.clone());
    let example_sentence = first_def.and_then(|d| d.example);

    Ok(UpsertDictionaryInput {
        word: first.word,
        ipa,
        pos,
        meaning_vi: None,
        meaning_en,
        example_sentence,
        audio_url,
        source: "online".into(),
    })
}

pub async fn fetch_online_entry(word: &str) -> AppResult<UpsertDictionaryInput> {
    let trimmed = word.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("Từ tra cứu không được để trống".into()));
    }

    let url = format!(
        "https://api.dictionaryapi.dev/api/v2/entries/en/{}",
        trimmed.to_lowercase()
    );
    let resp = reqwest::get(&url)
        .await
        .map_err(|e| AppError::Internal(format!("Không thể kết nối từ điển online: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::NotFound(format!(
            "Không tìm thấy từ \"{trimmed}\" trên từ điển online"
        )));
    }

    let body = resp
        .text()
        .await
        .map_err(|e| AppError::Internal(format!("Lỗi đọc phản hồi từ điển: {e}")))?;

    parse_api_response(&body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dictionaryapi_json_extracts_ipa_audio_and_example() {
        let sample = r#"[{
            "word": "ephemeral",
            "phonetics": [
                { "text": "/ɪˈfɛm.ə.ɹəl/", "audio": "https://api.dictionaryapi.dev/media/pronunciations/en/ephemeral.mp3" }
            ],
            "meanings": [{
                "partOfSpeech": "adjective",
                "definitions": [{
                    "definition": "Lasting for a very short time.",
                    "example": "fashions are ephemeral"
                }]
            }]
        }]"#;

        let parsed = parse_api_response(sample).expect("parse sample");
        assert_eq!(parsed.word, "ephemeral");
        assert_eq!(parsed.ipa.as_deref(), Some("/ɪˈfɛm.ə.ɹəl/"));
        assert_eq!(parsed.pos.as_deref(), Some("adjective"));
        assert_eq!(parsed.meaning_en.as_deref(), Some("Lasting for a very short time."));
        assert_eq!(parsed.example_sentence.as_deref(), Some("fashions are ephemeral"));
        assert!(parsed.audio_url.is_some());
    }
}
