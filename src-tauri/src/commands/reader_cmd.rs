use crate::error::AppResult;
use crate::reader::epub::{self, EpubBook};
use crate::reader::web::{self, WebReaderResult};

#[tauri::command]
pub fn parse_epub(bytes: Vec<u8>) -> AppResult<EpubBook> {
    epub::parse_epub_bytes(&bytes)
}

#[tauri::command]
pub async fn load_reader_url(url: String) -> AppResult<WebReaderResult> {
    web::load_url(&url).await
}
