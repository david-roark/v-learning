use std::collections::HashMap;
use std::io::{Cursor, Read};

use serde::Serialize;
use zip::ZipArchive;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpubChapter {
    pub title: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpubBook {
    pub title: String,
    pub chapters: Vec<EpubChapter>,
}

fn decode_entities(s: &str) -> String {
    s.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}

fn html_to_plain_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len() / 2);
    let mut in_tag = false;
    let mut skip_block = false;
    let mut tag_buf = String::new();

    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
            tag_buf.clear();
            continue;
        }
        if ch == '>' && in_tag {
            in_tag = false;
            let lower = tag_buf.trim().to_ascii_lowercase();
            if lower.starts_with("script") || lower.starts_with("style") {
                skip_block = true;
            } else if lower.starts_with("/script") || lower.starts_with("/style") {
                skip_block = false;
            } else if lower.starts_with("/p")
                || lower.starts_with("/div")
                || lower.starts_with("/h")
                || lower.starts_with("br")
                || lower.starts_with("/li")
            {
                out.push('\n');
            }
            continue;
        }
        if in_tag {
            tag_buf.push(ch);
        } else if !skip_block {
            out.push(ch);
        }
    }

    let decoded = decode_entities(&out);
    decoded
        .lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let pattern = format!("{attr}=\"");
    if let Some(start) = tag.find(&pattern) {
        let rest = &tag[start + pattern.len()..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    let single_pat = format!("{attr}='");
    if let Some(start) = tag.find(&single_pat) {
        let rest = &tag[start + single_pat.len()..];
        if let Some(end) = rest.find('\'') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

fn parse_opf_paths(opf_xml: &str, base_dir: &str) -> (Option<String>, Vec<String>) {
    let mut title = None;
    if let (Some(s), Some(e)) = (opf_xml.find("<dc:title"), opf_xml.find("</dc:title>"))
        && let Some(gt) = opf_xml[s..e].find('>')
    {
        let raw = opf_xml[s + gt + 1..e].trim();
        if !raw.is_empty() {
            title = Some(decode_entities(raw));
        }
    }

    let mut manifest: HashMap<String, String> = HashMap::new();
    let mut spine_ids: Vec<String> = Vec::new();

    for segment in opf_xml.split('<') {
        let trimmed = segment.trim();
        if trimmed.starts_with("item ") {
            if let (Some(id), Some(href)) =
                (extract_attr(trimmed, "id"), extract_attr(trimmed, "href"))
            {
                let full_path = if base_dir.is_empty() {
                    href
                } else {
                    format!("{base_dir}/{href}")
                };
                manifest.insert(id, full_path);
            }
        } else if trimmed.starts_with("itemref ")
            && let Some(idref) = extract_attr(trimmed, "idref")
        {
            spine_ids.push(idref);
        }
    }

    let paths = spine_ids
        .into_iter()
        .filter_map(|id| manifest.remove(&id))
        .collect();
    (title, paths)
}

pub fn parse_epub_bytes(bytes: &[u8]) -> AppResult<EpubBook> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| AppError::Validation(format!("File .epub không hợp lệ: {e}")))?;

    let mut opf_path: Option<String> = None;
    if let Ok(mut container) = archive.by_name("META-INF/container.xml") {
        let mut xml = String::new();
        let _ = container.read_to_string(&mut xml);
        opf_path = extract_attr(&xml, "full-path");
    }

    let mut book_title = "EPUB Book".to_string();
    let mut chapter_paths: Vec<String> = Vec::new();

    if let Some(ref opf) = opf_path
        && let Ok(mut opf_file) = archive.by_name(opf)
    {
        let mut opf_xml = String::new();
        let _ = opf_file.read_to_string(&mut opf_xml);
        let base_dir = opf.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
        let (title, paths) = parse_opf_paths(&opf_xml, base_dir);
        if let Some(t) = title {
            book_title = t;
        }
        chapter_paths = paths;
    }

    if chapter_paths.is_empty() {
        let mut fallback: Vec<String> = (0..archive.len())
            .filter_map(|i| {
                archive
                    .by_index(i)
                    .ok()
                    .and_then(|f| f.name().ok().map(|n| n.into_owned()))
            })
            .filter(|n| n.ends_with(".xhtml") || n.ends_with(".html"))
            .collect();
        fallback.sort();
        chapter_paths = fallback;
    }

    let mut chapters = Vec::new();
    for (idx, path) in chapter_paths.iter().enumerate() {
        if let Ok(mut file) = archive.by_name(path) {
            let mut html = String::new();
            if file.read_to_string(&mut html).is_ok() {
                let text = html_to_plain_text(&html);
                if text.len() > 20 {
                    chapters.push(EpubChapter {
                        title: format!("Chương {}", idx + 1),
                        text,
                    });
                }
            }
        }
    }

    if chapters.is_empty() {
        return Err(AppError::Validation(
            "Không trích xuất được nội dung chữ từ file .epub này.".into(),
        ));
    }

    Ok(EpubBook {
        title: book_title,
        chapters,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    #[test]
    fn parse_epub_extracts_title_and_chapters_in_spine_order() {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buf);
            zip.start_file("META-INF/container.xml", SimpleFileOptions::default())
                .unwrap();
            zip.write_all(
                br#"<?xml version="1.0"?><container><rootfiles><rootfile full-path="OEBPS/content.opf"/></rootfiles></container>"#,
            )
            .unwrap();

            zip.start_file("OEBPS/content.opf", SimpleFileOptions::default())
                .unwrap();
            zip.write_all(
                br#"<package><metadata><dc:title>Test English Book</dc:title></metadata><manifest><item id="ch1" href="ch1.xhtml"/></manifest><spine><itemref idref="ch1"/></spine></package>"#,
            )
            .unwrap();

            zip.start_file("OEBPS/ch1.xhtml", SimpleFileOptions::default())
                .unwrap();
            zip.write_all(
                br#"<html><body><p>Smartphones are ubiquitous in modern life.</p><p>Stay pragmatic!</p></body></html>"#,
            )
            .unwrap();
            zip.finish().unwrap();
        }

        let book = parse_epub_bytes(buf.get_ref()).expect("parse epub");
        assert_eq!(book.title, "Test English Book");
        assert_eq!(book.chapters.len(), 1);
        assert!(book.chapters[0].text.contains("Smartphones are ubiquitous"));
    }
}
