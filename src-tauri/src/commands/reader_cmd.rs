use crate::error::AppResult;
use crate::reader::epub::{self, EpubBook};

#[tauri::command]
pub fn parse_epub(bytes: Vec<u8>) -> AppResult<EpubBook> {
    epub::parse_epub_bytes(&bytes)
}
