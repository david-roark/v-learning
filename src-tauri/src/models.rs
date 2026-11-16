use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deck {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub new_cards_per_day: i64,
    pub desired_retention: f64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckWithStats {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub new_cards_per_day: i64,
    pub desired_retention: f64,
    pub created_at: i64,
    pub total_cards: i64,
    pub new_count: i64,
    pub due_count: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDeckInput {
    pub name: String,
    pub description: Option<String>,
    pub new_cards_per_day: Option<i64>,
    pub desired_retention: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub deck_id: String,
    pub note_type: String,
    pub word: String,
    pub ipa: Option<String>,
    pub meaning: String,
    pub example_sentence: Option<String>,
    pub collocations: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNoteInput {
    pub deck_id: String,
    pub note_type: String,
    pub word: String,
    pub ipa: Option<String>,
    pub meaning: String,
    pub example_sentence: Option<String>,
    pub collocations: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNoteInput {
    pub id: String,
    pub word: String,
    pub ipa: Option<String>,
    pub meaning: String,
    pub example_sentence: Option<String>,
    pub collocations: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub id: String,
    pub note_id: String,
    pub card_template: String,
    pub state: i64,
    pub due: i64,
    pub stability: f64,
    pub difficulty: f64,
    pub elapsed_days: i64,
    pub scheduled_days: i64,
    pub reps: i64,
    pub lapses: i64,
    pub last_review: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntervalPreview {
    pub again: String,
    pub hard: String,
    pub good: String,
    pub easy: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyCard {
    pub card: Card,
    pub note: Note,
    pub desired_retention: f64,
    pub intervals: IntervalPreview,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyStats {
    pub reviewed_today: i64,
    pub new_learned_today: i64,
    pub due_remaining: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryEntry {
    pub id: i64,
    pub word: String,
    pub ipa: Option<String>,
    pub pos: Option<String>,
    pub meaning_vi: Option<String>,
    pub meaning_en: Option<String>,
    pub example_sentence: Option<String>,
    pub audio_url: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertDictionaryInput {
    pub word: String,
    pub ipa: Option<String>,
    pub pos: Option<String>,
    pub meaning_vi: Option<String>,
    pub meaning_en: Option<String>,
    pub example_sentence: Option<String>,
    pub audio_url: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSettings {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiPracticeFeedback {
    pub feedback: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApkgImportReport {
    pub deck_id: String,
    pub deck_name: String,
    pub imported: i64,
    pub skipped: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeatmapDay {
    pub date: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForecastDay {
    pub date: String,
    pub due_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsSummary {
    pub streak_days: i64,
    pub total_reviews: i64,
    pub true_retention: f64,
    pub mature_cards: i64,
    pub heatmap: Vec<HeatmapDay>,
    pub forecast_7d: Vec<ForecastDay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebDavSettings {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRestoreReport {
    pub decks: i64,
    pub notes: i64,
    pub cards: i64,
    pub review_logs: i64,
}
