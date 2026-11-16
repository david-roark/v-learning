use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::models::{AiPracticeFeedback, AiSettings};

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    temperature: f32,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Deserialize)]
struct ChatChoiceMessage {
    content: String,
}

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<ModelItem>,
}

#[derive(Deserialize)]
struct ModelItem {
    id: String,
}

pub fn parse_models_json(json_str: &str) -> AppResult<Vec<String>> {
    let parsed: ModelsResponse = serde_json::from_str(json_str)
        .map_err(|e| AppError::Internal(format!("Lỗi đọc danh sách model: {e}")))?;

    let mut ids: Vec<String> = parsed
        .data
        .into_iter()
        .map(|m| m.id.strip_prefix("models/").unwrap_or(&m.id).to_string())
        .filter(|id| !id.is_empty())
        .collect();
    ids.sort();
    ids.dedup();
    Ok(ids)
}

pub async fn fetch_models(base_url: &str, api_key: &str) -> AppResult<Vec<String>> {
    let base = base_url.trim().trim_end_matches('/');
    if base.is_empty() {
        return Err(AppError::Validation("Base URL không được để trống.".into()));
    }

    let url = format!("{base}/models");
    let client = reqwest::Client::new();
    let mut req = client.get(&url);
    if !api_key.trim().is_empty() {
        req = req.bearer_auth(api_key.trim());
    }

    let resp = req
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Không thể kết nối tới {base}: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Internal(format!(
            "Lỗi kết nối ({status}): {body}"
        )));
    }

    let body = resp
        .text()
        .await
        .map_err(|e| AppError::Internal(format!("Lỗi đọc phản hồi: {e}")))?;

    parse_models_json(&body)
}

pub async fn evaluate_practice(
    settings: &AiSettings,
    target_words: &[String],
    user_text: &str,
) -> AppResult<AiPracticeFeedback> {
    if user_text.trim().is_empty() {
        return Err(AppError::Validation(
            "Vui lòng nhập câu tiếng Anh của bạn.".into(),
        ));
    }
    let base = settings.base_url.trim().trim_end_matches('/');
    if base.is_empty() || settings.model.trim().is_empty() {
        return Err(AppError::Validation(
            "Vui lòng cấu hình Base URL và Model AI trước.".into(),
        ));
    }

    let words_str = target_words.join(", ");
    let system_prompt = "You are a concise, encouraging English tutor for a Vietnamese learner. \
        Evaluate the user's English sentence(s) that practice the target vocabulary words. \
        Reply in Vietnamese with 3 short sections: \
        1) Nhận xét ngữ pháp & cách dùng từ mục tiêu; \
        2) Câu sửa lại (nếu có lỗi); \
        3) 1-2 gợi ý diễn đạt tự nhiên hơn (natural phrasing).";

    let user_prompt = format!("Từ vựng mục tiêu: [{words_str}]\nCâu của tôi: {user_text}");

    let client = reqwest::Client::new();
    let url = format!("{base}/chat/completions");
    let mut req = client.post(&url).json(&ChatRequest {
        model: settings.model.trim(),
        messages: vec![
            ChatMessage {
                role: "system",
                content: system_prompt,
            },
            ChatMessage {
                role: "user",
                content: &user_prompt,
            },
        ],
        temperature: 0.4,
    });

    if !settings.api_key.trim().is_empty() {
        req = req.bearer_auth(settings.api_key.trim());
    }

    let resp = req
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Không thể kết nối AI API: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Internal(format!(
            "AI API trả lỗi {status}: {body}"
        )));
    }

    let parsed: ChatResponse = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Lỗi đọc phản hồi AI: {e}")))?;

    let content = parsed
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| AppError::Internal("AI không trả về nội dung".into()))?;

    Ok(AiPracticeFeedback { feedback: content })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_models_json_strips_models_prefix_and_sorts() {
        let json = r#"{"data":[{"id":"models/gemini-2.5-flash"},{"id":"models/gemini-2.5-pro"},{"id":"gpt-4o-mini"}]}"#;
        let ids = parse_models_json(json).expect("parse models");
        assert_eq!(
            ids,
            vec!["gemini-2.5-flash", "gemini-2.5-pro", "gpt-4o-mini"]
        );
    }
}
