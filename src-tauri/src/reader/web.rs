use reqwest::header::USER_AGENT;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptLine {
    pub start_sec: f64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleBlock {
    pub tag: String, // "h1" | "h2" | "h3" | "p" | "blockquote" | "li"
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebReaderResult {
    pub kind: String, // "article" | "youtube"
    pub title: String,
    pub article_text: Option<String>,
    pub article_blocks: Vec<ArticleBlock>,
    pub youtube_id: Option<String>,
    pub transcript: Vec<TranscriptLine>,
}

pub fn extract_youtube_id(url: &str) -> Option<String> {
    let trimmed = url.trim();
    if let Some(rest) = trimmed.split("youtu.be/").nth(1) {
        let id: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if id.len() == 11 {
            return Some(id);
        }
    }
    if let Some(pos) = trimmed.find("v=") {
        let rest = &trimmed[pos + 2..];
        let id: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if id.len() == 11 {
            return Some(id);
        }
    }
    for marker in ["/shorts/", "/embed/"] {
        if let Some(rest) = trimmed.split(marker).nth(1) {
            let id: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            if id.len() == 11 {
                return Some(id);
            }
        }
    }
    None
}

fn decode_entities(s: &str) -> String {
    s.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&rsquo;", "'")
        .replace("&lsquo;", "'")
        .replace("&rdquo;", "\"")
        .replace("&ldquo;", "\"")
        .replace("&mdash;", "—")
        .replace("&ndash;", "–")
}

fn strip_tags_simple(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    decode_entities(&out)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_tag_content(html: &str, tag: &str) -> Option<String> {
    let open_pat = format!("<{tag}");
    let close_pat = format!("</{tag}>");
    let lower = html.to_ascii_lowercase();
    let start = lower.find(&open_pat)?;
    let gt = lower[start..].find('>')? + start + 1;
    let end = lower[gt..].find(&close_pat)? + gt;
    Some(html[gt..end].to_string())
}

pub fn extract_article_from_html(html: &str) -> (String, String, Vec<ArticleBlock>) {
    let title = extract_tag_content(html, "title")
        .map(|t| strip_tags_simple(&t))
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "Web Article".to_string());

    let scope = extract_tag_content(html, "article")
        .or_else(|| extract_tag_content(html, "main"))
        .or_else(|| extract_tag_content(html, "body"))
        .unwrap_or_else(|| html.to_string());

    let scope_lower = scope.to_ascii_lowercase();
    let mut cleaned = String::with_capacity(scope.len());
    let mut i = 0;
    let bytes = scope.as_bytes();
    let skip_tags = [
        "script", "style", "nav", "aside", "footer", "header", "noscript", "svg",
    ];

    while i < bytes.len() {
        if bytes[i] == b'<' {
            let rest_lower = &scope_lower[i..];
            let mut skipped = false;
            for tag in skip_tags {
                let prefix_len = 1 + tag.len();
                if rest_lower.len() > prefix_len
                    && rest_lower.as_bytes()[1..].starts_with(tag.as_bytes())
                    && matches!(
                        rest_lower.as_bytes()[prefix_len],
                        b'>' | b' ' | b'\n' | b'\r' | b'\t'
                    )
                {
                    let close = format!("</{tag}>");
                    if let Some(end_rel) = rest_lower.find(&close) {
                        i += end_rel + close.len();
                        skipped = true;
                        break;
                    }
                }
            }
            if skipped {
                continue;
            }
        }
        let ch = scope[i..].chars().next().unwrap_or(' ');
        cleaned.push(ch);
        i += ch.len_utf8();
    }

    let lower_clean = cleaned.to_ascii_lowercase();
    let mut paragraphs: Vec<(usize, usize, ArticleBlock)> = Vec::new();
    for tag in ["h1", "h2", "h3", "p", "blockquote", "li"] {
        let mut cursor = 0;
        let open_pat = format!("<{tag}");
        let close_pat = format!("</{tag}>");

        while let Some(rel_start) = lower_clean[cursor..].find(&open_pat) {
            let abs_start = cursor + rel_start;
            let after = lower_clean[abs_start..].chars().nth(open_pat.len());
            if !matches!(after, Some('>' | ' ' | '\n' | '\r' | '\t')) {
                cursor = abs_start + open_pat.len();
                continue;
            }
            if let Some(gt_rel) = lower_clean[abs_start..].find('>') {
                let content_start = abs_start + gt_rel + 1;
                if let Some(end_rel) = lower_clean[content_start..].find(&close_pat) {
                    let abs_end = content_start + end_rel + close_pat.len();
                    let raw_inner = &cleaned[content_start..content_start + end_rel];
                    let text = strip_tags_simple(raw_inner);
                    let word_count = text.split_whitespace().count();
                    if word_count >= 3 || tag.starts_with('h') && !text.is_empty() {
                        paragraphs.push((
                            abs_start,
                            abs_end,
                            ArticleBlock {
                                tag: tag.to_string(),
                                text,
                            },
                        ));
                    }
                    cursor = abs_end;
                    continue;
                }
            }
            break;
        }
    }

    paragraphs.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));
    let mut blocks: Vec<ArticleBlock> = Vec::new();
    let mut last_end = 0_usize;
    for (start, end, block) in paragraphs {
        if start < last_end && end <= last_end {
            continue;
        }
        last_end = end.max(last_end);
        blocks.push(block);
    }
    let body_text = blocks
        .iter()
        .map(|b| b.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");

    (title, body_text, blocks)
}

#[derive(Deserialize)]
struct CaptionTrack {
    #[serde(rename = "baseUrl")]
    base_url: String,
    #[serde(rename = "languageCode")]
    language_code: String,
}

pub fn parse_caption_track_url(watch_html: &str) -> Option<String> {
    let marker = "\"captionTracks\":";
    let start = watch_html.find(marker)? + marker.len();
    let rest = watch_html[start..].trim_start();
    if !rest.starts_with('[') {
        return None;
    }
    let end = rest.find(']')? + 1;
    let tracks: Vec<CaptionTrack> = serde_json::from_str(&rest[..end]).ok()?;
    let chosen = tracks
        .iter()
        .find(|t| t.language_code == "en")
        .or_else(|| tracks.iter().find(|t| t.language_code.starts_with("en")))
        .or_else(|| tracks.first())?;
    Some(chosen.base_url.replace("\\u0026", "&"))
}

pub fn parse_transcript_xml(xml: &str) -> Vec<TranscriptLine> {
    let mut lines = Vec::new();
    let mut cursor = 0;
    while let Some(rel) = xml[cursor..].find("<text ") {
        let start = cursor + rel;
        let Some(gt_rel) = xml[start..].find('>') else {
            break;
        };
        let attrs = &xml[start..start + gt_rel];
        let content_start = start + gt_rel + 1;
        let Some(end_rel) = xml[content_start..].find("</text>") else {
            break;
        };
        let raw_text = &xml[content_start..content_start + end_rel];
        let text = strip_tags_simple(raw_text);

        let start_sec = attrs
            .split("start=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        if !text.is_empty() {
            lines.push(TranscriptLine { start_sec, text });
        }
        cursor = content_start + end_rel + 7;
    }

    if !lines.is_empty() {
        return lines;
    }

    // Fallback for YouTube srv3 format: <p t="1500" d="2000">...</p>
    cursor = 0;
    while let Some(rel) = xml[cursor..].find("<p ") {
        let start = cursor + rel;
        let Some(gt_rel) = xml[start..].find('>') else {
            break;
        };
        let attrs = &xml[start..start + gt_rel];
        let content_start = start + gt_rel + 1;
        let Some(end_rel) = xml[content_start..].find("</p>") else {
            break;
        };
        let raw_text = &xml[content_start..content_start + end_rel];
        let text = strip_tags_simple(raw_text);

        let start_ms = attrs
            .split("t=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        if !text.is_empty() {
            lines.push(TranscriptLine {
                start_sec: start_ms / 1000.0,
                text,
            });
        }
        cursor = content_start + end_rel + 4;
    }

    lines
}

fn extract_innertube_api_key(html: &str) -> Option<String> {
    let marker = "\"INNERTUBE_API_KEY\":\"";
    let start = html.find(marker)? + marker.len();
    let end = html[start..].find('"')?;
    Some(html[start..start + end].to_string())
}

fn pick_caption_url(tracks: &[CaptionTrack]) -> Option<String> {
    let chosen = tracks
        .iter()
        .find(|t| t.language_code == "en")
        .or_else(|| tracks.iter().find(|t| t.language_code.starts_with("en")))
        .or_else(|| tracks.first())?;
    Some(chosen.base_url.replace("\\u0026", "&"))
}

#[derive(Deserialize)]
struct InnertubePlayerResp {
    captions: Option<InnertubeCaptions>,
}

#[derive(Deserialize)]
struct InnertubeCaptions {
    #[serde(rename = "playerCaptionsTracklistRenderer")]
    tracklist: Option<InnertubeTracklist>,
}

#[derive(Deserialize)]
struct InnertubeTracklist {
    #[serde(rename = "captionTracks")]
    caption_tracks: Option<Vec<CaptionTrack>>,
}

async fn fetch_youtube_transcript(
    client: &reqwest::Client,
    video_id: &str,
    watch_html: &str,
    web_ua: &str,
) -> Vec<TranscriptLine> {
    let android_ua = "com.google.android.youtube/20.10.38 (Linux; U; Android 11) gzip";
    if let Some(api_key) = extract_innertube_api_key(watch_html) {
        let player_url = format!("https://www.youtube.com/youtubei/v1/player?key={api_key}");
        let body = serde_json::json!({
            "context": {
                "client": {
                    "clientName": "ANDROID",
                    "clientVersion": "20.10.38",
                    "androidSdkVersion": 30,
                    "hl": "en",
                    "gl": "US"
                }
            },
            "videoId": video_id
        });
        if let Ok(resp) = client
            .post(&player_url)
            .header(USER_AGENT, android_ua)
            .json(&body)
            .send()
            .await
            && let Ok(parsed) = resp.json::<InnertubePlayerResp>().await
            && let Some(tracks) = parsed
                .captions
                .and_then(|c| c.tracklist)
                .and_then(|t| t.caption_tracks)
            && let Some(cap_url) = pick_caption_url(&tracks)
            && let Ok(cap_resp) = client
                .get(&cap_url)
                .header(USER_AGENT, android_ua)
                .send()
                .await
            && let Ok(xml) = cap_resp.text().await
        {
            let lines = parse_transcript_xml(&xml);
            if !lines.is_empty() {
                return lines;
            }
        }
    }

    if let Some(cap_url) = parse_caption_track_url(watch_html) {
        for suffix in ["", "&fmt=srv3"] {
            let url = format!("{cap_url}{suffix}");
            if let Ok(resp) = client.get(&url).header(USER_AGENT, web_ua).send().await
                && let Ok(xml) = resp.text().await
            {
                let lines = parse_transcript_xml(&xml);
                if !lines.is_empty() {
                    return lines;
                }
            }
        }
    }

    Vec::new()
}

pub async fn load_url(url: &str) -> AppResult<WebReaderResult> {
    let trimmed = url.trim();
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err(AppError::Validation(
            "URL phải bắt đầu bằng http:// hoặc https://".into(),
        ));
    }

    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(6))
        .timeout(std::time::Duration::from_secs(12))
        .build()
        .map_err(|e| AppError::Internal(format!("Không thể khởi tạo HTTP client: {e}")))?;
    let ua = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36";

    if let Some(video_id) = extract_youtube_id(trimmed) {
        let watch_url = format!("https://www.youtube.com/watch?v={video_id}");
        let html = client
            .get(&watch_url)
            .header(USER_AGENT, ua)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Không thể tải trang YouTube: {e}")))?
            .text()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let title = extract_tag_content(&html, "title")
            .map(|t| strip_tags_simple(&t).replace(" - YouTube", ""))
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| format!("YouTube ({video_id})"));

        let transcript = fetch_youtube_transcript(&client, &video_id, &html, ua).await;

        return Ok(WebReaderResult {
            kind: "youtube".into(),
            title,
            article_text: None,
            article_blocks: Vec::new(),
            youtube_id: Some(video_id),
            transcript,
        });
    }

    let resp = client
        .get(trimmed)
        .header(USER_AGENT, ua)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Không thể tải URL: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::Internal(format!(
            "Trang web trả về lỗi HTTP {}",
            resp.status()
        )));
    }

    let html = resp
        .text()
        .await
        .map_err(|e| AppError::Internal(format!("Lỗi đọc nội dung trang: {e}")))?;

    let (title, body_text, article_blocks) = extract_article_from_html(&html);
    if body_text.trim().is_empty() {
        return Err(AppError::Validation(
            "Không trích xuất được bài viết (trang có thể yêu cầu JavaScript hoặc chặn bot)."
                .into(),
        ));
    }

    Ok(WebReaderResult {
        kind: "article".into(),
        title,
        article_text: Some(body_text),
        article_blocks,
        youtube_id: None,
        transcript: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_youtube_id_handles_standard_short_and_embed_urls() {
        assert_eq!(
            extract_youtube_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ").as_deref(),
            Some("dQw4w9WgXcQ")
        );
        assert_eq!(
            extract_youtube_id("https://youtu.be/dQw4w9WgXcQ?t=10").as_deref(),
            Some("dQw4w9WgXcQ")
        );
        assert_eq!(
            extract_youtube_id("https://www.youtube.com/shorts/dQw4w9WgXcQ").as_deref(),
            Some("dQw4w9WgXcQ")
        );
    }

    #[test]
    fn extract_article_from_html_ignores_nav_and_scripts() {
        let html = r#"
            <html><head><title>Sample Article</title></head>
            <body>
                <nav><a href="/">Home Menu Link</a></nav>
                <article>
                    <h1>Learning Pragmatic English</h1>
                    <script>var x = 1;</script>
                    <p>Smartphones have become ubiquitous in modern life.</p>
                </article>
            </body></html>
        "#;
        let (title, text, blocks) = extract_article_from_html(html);
        assert_eq!(title, "Sample Article");
        assert!(text.contains("Learning Pragmatic English"));
        assert!(text.contains("Smartphones have become ubiquitous"));
        assert!(!text.contains("Home Menu Link"));
        assert!(!text.contains("var x = 1"));
        assert_eq!(blocks[0].tag, "h1");
        assert_eq!(blocks[1].tag, "p");
    }

    #[test]
    fn parse_transcript_xml_extracts_timestamps_and_text() {
        let xml = r#"<transcript><text start="1.5" dur="2.0">Hello &amp; welcome.</text><text start="4.2" dur="1.8">Stay resilient!</text></transcript>"#;
        let lines = parse_transcript_xml(xml);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].start_sec, 1.5);
        assert_eq!(lines[0].text, "Hello & welcome.");
        assert_eq!(lines[1].text, "Stay resilient!");
    }

    #[test]
    fn extract_article_handles_multiline_tags_and_avoids_nested_duplicates() {
        let html = r#"
            <article>
                <p
                    class="lead">Multiline paragraph tag works properly.</p>
                <li><p>Nested paragraph inside list item.</p></li>
            </article>
        "#;
        let (_title, _text, blocks) = extract_article_from_html(html);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].text, "Multiline paragraph tag works properly.");
        assert_eq!(blocks[1].text, "Nested paragraph inside list item.");
    }
}
