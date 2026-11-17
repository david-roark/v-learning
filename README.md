<div align="center">
  <img src="static/logo.png" alt="V-Learning Logo" width="84" height="84" />
  <h1>V-Learning</h1>
  <p>Ứng dụng học tiếng Anh & từ vựng offline-first (FSRS, Đọc báo/Sách/Video, Từ điển, Mini-game) xây dựng bằng <strong>Tauri v2 + SvelteKit + Rust + SQLite</strong>.</p>
</div>

---

## Tính năng chính

- **Bộ thẻ & Ôn tập thông minh (FSRS):**
  - Lập lịch ôn tập theo thuật toán **FSRS**, thống kê tiến độ hàng ngày, chuỗi ngày học (streak) và dự báo thẻ đến hạn.
  - Hỗ trợ **Import `.apkg`** từ Anki, sao lưu/khôi phục JSON và đồng bộ **WebDAV**.
  - Tích hợp **Dictation** (chính tả), **Shadowing** và **AI Practice** (Gemini, OpenAI, Ollama hoặc endpoint tương thích OpenAI).
- **Từ điển Offline & Online:**
  - Tra cứu tức thì trên gói từ điển SQLite **FTS5** tách riêng (`dict.db`), tự nhận diện biến thể từ (`ran → run`, `colour → color`), IPA US/UK, collocations, từ đồng nghĩa/trái nghĩa.
  - Tự động bổ sung từ nguồn online khi cần và lưu trực tiếp thành flashcard.
- **Đọc & Xem (Read & Watch):**
  - Hỗ trợ đọc **bài báo Web**, xem **YouTube** kèm transcript đồng bộ tương tác, đọc sách **EPUB**, **PDF**, phụ đề `.srt/.vtt` và văn bản `.txt/.md`.
  - Click 1 từ hoặc bôi đen cả cụm từ / nguyên câu để tra nghĩa, dịch và lưu flashcard kèm câu ngữ cảnh.
  - Lưu lịch sử đọc gần đây (kèm cache file offline trong IndexedDB) để mở lại nhanh chóng.
- **Mini-game ôn từ vựng:**
  - 5 chế độ chơi: **Ghép cặp**, **Speed Quiz 45s** (có đảo chiều Anh ↔ Việt), **Nghe đoán từ**, **Điền từ vào câu (Cloze)** và **Xếp chữ chính tả**.
  - Hỗ trợ nguồn từ theo bộ thẻ hoặc **20 từ mới thêm gần đây**, kèm nút **Ôn ngay các từ vừa sai**.
- **Tra cứu nhanh (Desktop) & Phát âm (TTS):**
  - Phím tắt toàn cục `Ctrl+Shift+D` và System Tray để bật cửa sổ Quick Lookup mọi lúc trên Desktop.
  - Hệ thống TTS đa nguồn có thứ tự ưu tiên & fallback giọng đọc hệ thống.

---

## Công nghệ sử dụng

- **Frontend:** Svelte 5, SvelteKit (`@sveltejs/adapter-static`), TypeScript (`strict`), Tailwind CSS v4, `pdfjs-dist`
- **Backend:** Tauri v2, Rust, `rusqlite` (SQLite bundled + FTS5), `fsrs`, `reqwest`
- **Data Pipeline:** Python (`scripts/dict-builder`) đóng gói từ điển offline nén Zstandard (`.db.zst`)

---

## Phát triển & Kiểm thử

```bash
# Cài dependencies
npm install

# Chạy ứng dụng desktop ở chế độ dev
npm run tauri dev

# Kiểm tra format (Prettier + rustfmt)
npm run format:check

# Kiểm tra toàn diện (svelte-check, eslint, clippy, unit tests FE/Rust/Python)
npm run verify
```

### Đóng gói dữ liệu từ điển offline

Xem hướng dẫn chi tiết tại [scripts/dict-builder/README.md](scripts/dict-builder/README.md):

```bash
python3 -m unittest scripts/dict-builder/test_builder.py
python3 scripts/dict-builder/build_dict.py --sample --no-compress
```

---

## GitHub Actions & Release

- **CI ([ci.yml](.github/workflows/ci.yml)):** tự động format-check, type-check, lint, test (Frontend, Rust, Python) và build frontend khi push/PR lên `main`.
- **Desktop Release ([release-desktop.yml](.github/workflows/release-desktop.yml)):** tự động build bộ cài đặt cho **Linux, Windows, macOS** (Apple Silicon & Intel) khi push tag `v*` hoặc chạy tay.
- **Dictionary Pack ([dict-pack.yml](.github/workflows/dict-pack.yml)):** build và phát hành gói từ điển `modern` & `full` (kèm Kaikki English) lên GitHub Releases.
- **Android APK ([build-android.yml](.github/workflows/build-android.yml)):** build file `.apk` thử nghiệm cho Android trên `ubuntu-latest`.

---

## Giấy phép (License)

- **Mã nguồn ứng dụng:** Phát hành theo [LICENSE](LICENSE) — chỉ sử dụng cho mục đích **cá nhân, học tập và phi thương mại**.
- **Dữ liệu từ điển (`scripts/dict-builder`):** Tuân thủ giấy phép gốc của từng nguồn dữ liệu nguồn mở (`CC BY-SA 4.0`, `CC BY 4.0`, BSD, v.v.).
