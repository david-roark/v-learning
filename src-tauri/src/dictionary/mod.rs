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

    let ipa = first.phonetic.filter(|s| !s.trim().is_empty()).or_else(|| {
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
    let pos = first_meaning
        .as_ref()
        .and_then(|m| m.part_of_speech.clone());
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

pub fn parse_translate_response(json_str: &str) -> AppResult<String> {
    let val: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| AppError::Internal(format!("Lỗi phân tích kết quả dịch: {e}")))?;
    let chunks = val
        .get(0)
        .and_then(|v| v.as_array())
        .ok_or_else(|| AppError::NotFound("Không có kết quả dịch phù hợp".into()))?;

    let mut out = String::new();
    for item in chunks {
        if let Some(piece) = item.get(0).and_then(|s| s.as_str()) {
            out.push_str(piece);
        }
    }
    let trimmed = out.trim().to_string();
    if trimmed.is_empty() {
        return Err(AppError::NotFound("Không có kết quả dịch phù hợp".into()));
    }
    Ok(trimmed)
}

pub(crate) fn percent_encode_query(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 2);
    for b in input.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char);
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

async fn translate_en_to_vi(text: &str) -> AppResult<UpsertDictionaryInput> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Internal(format!("Không thể tạo HTTP client: {e}")))?;

    let url = format!(
        "https://translate.googleapis.com/translate_a/single?client=gtx&sl=en&tl=vi&dt=t&q={}",
        percent_encode_query(text)
    );
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Không thể kết nối dịch vụ dịch: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::Internal(format!(
            "Dịch vụ dịch trả về lỗi HTTP {}",
            resp.status()
        )));
    }

    let body = resp
        .text()
        .await
        .map_err(|e| AppError::Internal(format!("Lỗi đọc kết quả dịch: {e}")))?;
    let meaning_vi = parse_translate_response(&body)?;
    let word_count = text.split_whitespace().count();

    Ok(UpsertDictionaryInput {
        word: text.to_string(),
        ipa: None,
        pos: Some(
            if word_count >= 5 {
                "sentence"
            } else {
                "phrase"
            }
            .into(),
        ),
        meaning_vi: Some(meaning_vi),
        meaning_en: None,
        example_sentence: None,
        audio_url: None,
        source: "translate".into(),
    })
}

pub async fn fetch_online_entry(word: &str) -> AppResult<UpsertDictionaryInput> {
    let trimmed = word.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "Từ tra cứu không được để trống".into(),
        ));
    }

    if trimmed.split_whitespace().count() > 1 {
        return translate_en_to_vi(trimmed).await;
    }

    let url = format!(
        "https://api.dictionaryapi.dev/api/v2/entries/en/{}",
        trimmed.to_lowercase()
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Internal(format!("Không thể tạo HTTP client: {e}")))?;
    let resp = client
        .get(&url)
        .header(
            reqwest::header::USER_AGENT,
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
        )
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Không thể kết nối từ điển online: {e}")))?;

    if !resp.status().is_success() {
        return translate_en_to_vi(trimmed).await.map_err(|_| {
            AppError::NotFound(format!(
                "Không tìm thấy từ \"{trimmed}\" trên từ điển online"
            ))
        });
    }

    let body = resp
        .text()
        .await
        .map_err(|e| AppError::Internal(format!("Lỗi đọc phản hồi từ điển: {e}")))?;

    parse_api_response(&body)
}

#[derive(Debug, Deserialize)]
struct DatamuseWord {
    word: String,
}

pub async fn fetch_collocations(word: &str) -> AppResult<Vec<String>> {
    let trimmed = word.trim().to_lowercase();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let url = format!("https://api.datamuse.com/words?rel_bga={trimmed}&max=6");
    let resp = reqwest::get(&url)
        .await
        .map_err(|e| AppError::Internal(format!("Lỗi kết nối Datamuse: {e}")))?;

    if !resp.status().is_success() {
        return Ok(Vec::new());
    }

    let items: Vec<DatamuseWord> = resp.json().await.unwrap_or_default();
    Ok(items
        .into_iter()
        .map(|item| format!("{trimmed} {}", item.word))
        .collect())
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
        assert_eq!(
            parsed.meaning_en.as_deref(),
            Some("Lasting for a very short time.")
        );
        assert_eq!(
            parsed.example_sentence.as_deref(),
            Some("fashions are ephemeral")
        );
        assert!(parsed.audio_url.is_some());
    }

    #[test]
    fn parse_translate_response_extracts_full_sentence() {
        let sample = r#"[[["Điện thoại thông minh đã trở nên phổ biến ", "Smartphones have become ubiquitous ", null, null, 1],["trong cuộc sống hiện đại.", "in modern life.", null, null, 1]], null, "en"]"#;
        let translated = parse_translate_response(sample).expect("parse translate");
        assert_eq!(
            translated,
            "Điện thoại thông minh đã trở nên phổ biến trong cuộc sống hiện đại."
        );
    }
}
