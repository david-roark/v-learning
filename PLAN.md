# Kế Hoạch Sản Phẩm: V-Learning (Cross-Platform English Desktop App)

> **Định vị:** Ứng dụng Desktop học tiếng Anh theo phương pháp Spaced Repetition (SRS) thế hệ mới — **Nhanh, nhẹ, Offline-first**, khắc phục điểm yếu của Anki (giao diện cũ, tạo thẻ thủ công tốn sức, thiếu thực hành Nghe–Nói–Viết).

---

## 1. Kiến Trúc & Công Nghệ Đề Xuất (Tech Stack)

- **Desktop Framework:** **Tauri v2** (Rust backend + Web frontend).
  - _Lý do:_ File cài đặt nhỏ (~10–20MB), tiêu thụ RAM cực thấp khi chạy ngầm trên System Tray, hỗ trợ tốt Global Shortcut, File System và Audio.
- **Database:** **SQLite** (chạy local 100%, sử dụng `rusqlite` hoặc `sqlx` ở tầng Rust + bật `FTS5` để tìm kiếm từ vựng tức thì).
- **Thuật toán lặp lại ngắt quãng:** **FSRS** (Free Spaced Repetition Scheduler — dùng crate `fsrs` của Rust hoặc `ts-fsrs`).
- **Frontend:** **Svelte 5 + TypeScript + TailwindCSS + Vite** (compile ra JS thuần cực nhẹ, dùng Svelte Runes quản lý state gọn, hỗ trợ Dark Mode và điều khiển hoàn toàn bằng bàn phím).
- **Audio & TTS:** Lưu file audio cục bộ (MP3/OGG), fallback bằng TTS (Edge TTS / Piper local TTS).

### Quy Tắc Tổ Chức Codebase (Modular Architecture)

- **Nguyên tắc chia nhỏ:** Codebase tổ chức theo **feature-based modules**, mỗi file chỉ đảm nhận 1 trách nhiệm duy nhất (Single Responsibility), giữ kích thước gọn nhẹ (lý tưởng **< 200–300 dòng/file**), tránh file "khổng lồ".
- **Cấu trúc Backend (Rust - `src-tauri/src/`):**
  - `commands/` — Các Tauri command mỏng (chỉ nhận input, gọi service, trả kết quả).
  - `db/` — Kết nối SQLite, migrations và các repository nhỏ theo từng bảng (`deck_repo.rs`, `note_repo.rs`, `card_repo.rs`, `dict_repo.rs`).
  - `srs/` — Đóng gói thuật toán FSRS và logic tính lịch ôn.
  - `dictionary/` — Engine tra cứu offline (FTS5) và adapter gọi online fallback.
  - `media/` — Quản lý cache audio, ghi âm và TTS.
  - `desktop/` — System tray, global shortcut, mini pop-up window.
- **Cấu trúc Frontend (`src/`):**
  - `features/` — Tách biệt theo nghiệp vụ: `decks/`, `study/`, `card-editor/`, `dictionary/`, `dictation/`, `analytics/`. Mỗi feature tự chứa `components/`, `state.svelte.ts` (Svelte 5 Runes), `types.ts`, `api.ts`.
  - `shared/` — UI primitives dùng chung, keyboard shortcut manager, utils.

---

## 2. Chi Tiết Hệ Thống Tính Năng

### Module 1: Lõi SRS & Trải Nghiệm Ôn Tập (Core SRS)

- **Thuật toán FSRS mặc định:**
  - Cho phép người dùng đặt mục tiêu tỷ lệ nhớ (_Desired Retention_, mặc định 90%).
  - Tự động tính toán khoảng thời gian ôn lại tối ưu, hiển thị trước thời gian dự kiến trên 4 nút: `Again (1)`, `Hard (2)`, `Good (3)`, `Easy (4)`.
- **Các loại thẻ (Card Types) chuyên biệt cho tiếng Anh:**
  1. **Vocabulary Card:** Mặt trước (Từ/Cụm từ + Audio + Câu ví dụ ẩn nghĩa) -> Mặt sau (IPA, Audio US/UK, Nghĩa Anh-Việt/Anh-Anh, Collocations, Họ từ, Giải thích ngữ cảnh).
  2. **Cloze Deletion (Điền khuyết theo ngữ cảnh):** Hiển thị cả câu nhưng đục lỗ từ mục tiêu (kèm gợi ý chữ cái đầu hoặc loại từ nếu cần).
  3. **Listening / Dictation Card:** Chỉ phát audio (không hiện chữ) -> người học tự nhớ hoặc gõ lại.
- **Keyboard-first UX (Tối ưu thao tác bàn phím):**
  - `Space` / `Enter`: Lật thẻ.
  - `1` `2` `3` `4`: Chấm mức độ nhớ.
  - `R`: Phát lại audio (hỗ trợ chỉnh tốc độ `0.8x`, `1.0x`, `1.2x`).
  - `E`: Sửa nhanh nội dung thẻ ngay trong lúc ôn.
  - `Ctrl+Z`: Hoàn tác (Undo) lượt chấm vừa rồi.

### Module 2: Tạo Thẻ Thông Minh (Smart Card Creation)

- **Auto-fill 1 chạm:** Khi gõ một từ hoặc cụm từ, app tự động điền:
  - Phiên âm IPA (US/UK) + Audio phát âm chuẩn.
  - Loại từ (noun, verb, adj...) và các nghĩa phổ biến.
  - Collocations (cụm từ hay đi kèm) và câu ví dụ thực tế.
- **Từ điển tích hợp (Offline Dictionary + Online Fallback):**
  - **Nguồn Offline (Build sẵn vào SQLite FTS5):**
    - _Anh–Việt:_ Bộ dữ liệu **Hồ Ngọc Đức (`anhviet109K`)** từ Free Vietnamese Dictionary Project (~109.000 mục từ, định dạng mở, dễ parse sang SQLite).
    - _Anh–Anh, IPA & Ngữ pháp:_ **Kaikki.org (Wiktionary JSONL dump)** lọc riêng tiếng Anh — có IPA (US/UK), loại từ, nghĩa chi tiết, từ đồng nghĩa/trái nghĩa và link audio Wikimedia.
    - _IPA bổ sung:_ **open-dict-data/ipa-dict** hoặc **CMU Pronouncing Dictionary**.
    - _Câu ví dụ thực tế:_ **Tatoeba** (CC BY) để lấy câu ví dụ song ngữ Anh–Việt tự nhiên.
  - **Nguồn Online Fallback (khi từ mới/slang chưa có offline):**
    - **Free Dictionary API (`dictionaryapi.dev`):** Miễn phí, không cần API key, trả về IPA, link MP3 phát âm, nghĩa Anh–Anh và ví dụ.
    - **Datamuse API:** Tìm _collocations_, từ đồng nghĩa, từ liên quan theo ngữ cảnh.
  - **Audio & Cache:** Tải file MP3 từ `dictionaryapi.dev` / Wikimedia ở lần tra đầu tiên và cache vào thư mục local để các lần ôn sau phát offline hoàn toàn; dùng TTS (Edge TTS / Piper) làm fallback cho câu ví dụ dài.
- **Batch Import / Export:**
  - Import từ file CSV/TSV, JSON.
  - Hỗ trợ đọc/chuyển đổi bộ thẻ `.apkg` của Anki để người dùng cũ chuyển sang dễ dàng.

### Module 3: Global Quick-Lookup & Capture (Điểm khác biệt của Desktop)

- **Chạy ngầm nhẹ nhàng (System Tray):** App luôn sẵn sàng ở thanh tác vụ mà không tốn tài nguyên.
- **Phím tắt toàn hệ thống (Global Shortcut):** Ví dụ `Ctrl + Shift + D` (Windows/Linux) hoặc `Cmd + Shift + D` (macOS).
  - Khi đang đọc báo, PDF, xem tài liệu hoặc code: Bôi đen từ/câu -> nhấn phím tắt.
  - Hiện một **Mini Pop-up Window** ngay cạnh con trỏ để tra nghĩa nhanh + nghe phát âm.
  - Nút **"Add to Deck" (1 click / Enter):** Tự động lưu từ đó vào bộ thẻ kèm theo chính câu văn bạn vừa bôi đen làm câu ví dụ ngữ cảnh.

### Module 4: Luyện Nghe – Nói – Viết (Active Production)

_Khắc phục điểm yếu "học vẹt mặt chữ nhưng không dùng được" của flashcard truyền thống:_

- **Dictation Mode (Nghe – Chép chính tả):**
  - App phát audio của câu ví dụ. Người học gõ lại những gì nghe được.
  - Hệ thống so sánh (diff) và tô màu: từ đúng (xanh), từ sai chính tả/nghe nhầm (đỏ), từ bị thiếu (vàng). Xử lý linh hoạt dấu câu và viết tắt (`don't` / `do not`).
- **Shadowing & Ghi âm so sánh:**
  - Nút ghi âm nhanh ngay trên thẻ ôn tập.
  - Cho phép nghe lại giọng mẫu và giọng của chính mình (A/B comparison) để tự chỉnh ngữ điệu.
  - _(Mở rộng)_ Tích hợp AI/Speech-to-Text để chỉ ra lỗi phát âm theo từng âm vị (IPA).
- **AI Contextual Practice (Tùy chọn dùng API Key cá nhân hoặc Local LLM):**
  - Cuối phiên ôn tập, chọn ra 3–5 từ có độ khó cao (lapses nhiều) yêu cầu người dùng viết 1–2 câu hoặc một đoạn văn ngắn có chứa các từ đó.
  - AI kiểm tra ngữ pháp, độ tự nhiên (natural phrasing) và sửa lỗi trực tiếp.

### Module 5: Content Mining (Mở rộng nâng cao)

- **Built-in Reader (EPUB / PDF / Markdown):** Đọc sách, tài liệu tiếng Anh ngay trong app. Click đúp vào từ lạ để tra và tạo thẻ kèm đoạn trích dẫn + tên sách.
- **Subtitle / Video Mining:** Kéo thả video + file phụ đề `.srt`/`.vtt` vào app. Khi gặp câu hay, bấm nút để tự động cắt đoạn audio ngắn (MP3) + ảnh chụp màn hình (Screenshot) tạo thành một flashcard sinh động.

### Module 6: Thống Kê & Chống Quá Tải (Analytics)

- **Dashboard trực quan:**
  - Biểu đồ Heatmap các ngày học trong năm, chuỗi ngày học (Streak).
  - Chỉ số **True Retention** (tỷ lệ nhớ thực tế) và **Estimated Vocabulary Size** (số từ đã thuộc vững).
- **Workload Forecast & Anti-Burnout:**
  - Dự báo số thẻ cần ôn trong 7–14 ngày tới.
  - Cảnh báo nếu người dùng thêm quá nhiều thẻ mới vượt ngưỡng chịu tải, gợi ý tạm dừng thẻ mới để dọn dẹp thẻ tồn (_Review Backlog_).

---

## 3. Thiết Kế Database SQLite Sơ Bộ

Tách biệt giữa **Nội dung kiến thức (`notes`)** và **Trạng thái lịch ôn (`cards`)** giống mô hình tối ưu của Anki + FSRS:

```sql
-- 1. Bộ thẻ (Decks)
CREATE TABLE decks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    new_cards_per_day INTEGER DEFAULT 20,
    desired_retention REAL DEFAULT 0.90,
    created_at INTEGER NOT NULL
);

-- 2. Nội dung thẻ (Notes) - Chứa dữ liệu tiếng Anh
CREATE TABLE notes (
    id TEXT PRIMARY KEY,
    deck_id TEXT NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
    note_type TEXT NOT NULL, -- 'vocab' | 'cloze' | 'sentence'
    word TEXT,               -- Từ/cụm từ chính
    ipa TEXT,                -- Phiên âm
    meaning TEXT NOT NULL,   -- Nghĩa
    example_sentence TEXT,   -- Câu ví dụ ngữ cảnh
    collocations TEXT,       -- JSON array các cụm từ đi kèm
    audio_path TEXT,         -- Đường dẫn file audio local
    image_path TEXT,         -- Đường dẫn ảnh (nếu có)
    tags TEXT,               -- JSON array tags
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 3. Trạng thái SRS của từng thẻ (Cards) - Phục vụ thuật toán FSRS
CREATE TABLE cards (
    id TEXT PRIMARY KEY,
    note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    card_template TEXT NOT NULL, -- 'recognition' | 'recall' | 'dictation'
    state INTEGER DEFAULT 0,     -- 0: New, 1: Learning, 2: Review, 3: Relearning
    due INTEGER NOT NULL,        -- Timestamp (ms) đến hạn ôn tiếp theo
    stability REAL DEFAULT 0,    -- Tham số FSRS: Độ bền trí nhớ
    difficulty REAL DEFAULT 0,   -- Tham số FSRS: Độ khó
    elapsed_days INTEGER DEFAULT 0,
    scheduled_days INTEGER DEFAULT 0,
    reps INTEGER DEFAULT 0,
    lapses INTEGER DEFAULT 0,
    last_review INTEGER
);

-- 4. Lịch sử ôn tập (Review Logs) - Dùng để vẽ thống kê & tối ưu tham số FSRS
CREATE TABLE review_logs (
    id TEXT PRIMARY KEY,
    card_id TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    rating INTEGER NOT NULL, -- 1: Again, 2: Hard, 3: Good, 4: Easy
    state INTEGER NOT NULL,
    elapsed_days INTEGER NOT NULL,
    scheduled_days INTEGER NOT NULL,
    duration_ms INTEGER,     -- Thời gian suy nghĩ của người học
    reviewed_at INTEGER NOT NULL
);
```

---

## 4. Lộ Trình Phát Triển (Roadmap)

### Giai đoạn 1: MVP — Dùng được hàng ngày (Tuần 1–3)

- [x] Khởi tạo dự án **Tauri v2 + SQLite** và cấu trúc thư mục.
- [x] Xây dựng Schema SQLite & tích hợp thư viện **FSRS**.
- [x] CRUD Bộ thẻ (Decks) & Tạo thẻ thủ công (Vocabulary + Cloze).
- [x] Màn hình ôn tập mượt mà hỗ trợ đầy đủ phím tắt (`Space`, `1-4`, `R`).
- [x] Thống kê cơ bản (số thẻ đã học hôm nay, thẻ đến hạn).

### Giai đoạn 2: Sức Mạnh Desktop & Tự Động Hóa (Tuần 4–6)

- [x] Nhúng từ điển Offline (SQLite FTS5) + Auto-fill IPA, nghĩa, ví dụ.
- [x] Phát âm chuẩn (tích hợp tải/cache audio hoặc TTS).
- [x] Tính năng **Global Quick-Lookup** (System Tray + Phím tắt toàn cục + Mini Pop-up).
- [x] Chế độ **Dictation Mode** (nghe – gõ lại – highlight lỗi sai).

### Giai đoạn 3: Active Production & Mining (Tuần 7–10)

- [x] Ghi âm Shadowing so sánh trực tiếp A/B.
- [x] Tích hợp AI (tùy chọn API key của người dùng) để chấm câu viết từ vựng vừa ôn.
- [x] Trình đọc Text tích hợp & Import `.apkg` (Anki).

### Giai đoạn 4: Hoàn Thiện & Đồng Bộ (Mở rộng)

- [x] Analytics Dashboard (90-day Heatmap, Streak, True Retention, Dự báo 7 ngày).
- [x] Cloud Sync (Export/Restore JSON & WebDAV Push/Pull để giữ tính riêng tư).
- [x] Sẵn sàng đóng gói đa nền tảng qua `npm run tauri build` (`.msi`/`.exe`, `.dmg`, `.AppImage`/`.deb`).
