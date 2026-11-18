export interface Deck {
  id: string;
  name: string;
  description: string | null;
  newCardsPerDay: number;
  desiredRetention: number;
  createdAt: number;
}

export interface DeckWithStats extends Deck {
  totalCards: number;
  newCount: number;
  dueCount: number;
}

export interface CreateDeckInput {
  name: string;
  description?: string;
  newCardsPerDay?: number;
  desiredRetention?: number;
}

export type NoteType = "vocab" | "cloze";

export interface Note {
  id: string;
  deckId: string;
  noteType: NoteType;
  word: string;
  ipa: string | null;
  meaning: string;
  exampleSentence: string | null;
  collocations: string[];
  tags: string[];
  createdAt: number;
  updatedAt: number;
}

export interface CreateNoteInput {
  deckId: string;
  noteType: NoteType;
  word: string;
  ipa?: string;
  meaning: string;
  exampleSentence?: string;
  collocations?: string[];
  tags?: string[];
}

export interface UpdateNoteInput {
  id: string;
  word: string;
  ipa?: string;
  meaning: string;
  exampleSentence?: string;
  collocations?: string[];
  tags?: string[];
}

export interface Card {
  id: string;
  noteId: string;
  cardTemplate: string;
  state: number;
  due: number;
  stability: number;
  difficulty: number;
  elapsedDays: number;
  scheduledDays: number;
  reps: number;
  lapses: number;
  lastReview: number | null;
}

export interface IntervalPreview {
  again: string;
  hard: string;
  good: string;
  easy: string;
}

export interface StudyCard {
  card: Card;
  note: Note;
  desiredRetention: number;
  intervals: IntervalPreview;
}

export interface DailyStats {
  reviewedToday: number;
  newLearnedToday: number;
  dueRemaining: number;
}

export type Rating = 1 | 2 | 3 | 4;

export interface DictionarySense {
  pos: string;
  definitionEn: string | null;
  definitionVi: string | null;
  examples: string[];
  tags: string[];
  synonyms: string[];
  antonyms: string[];
}

export interface DictionaryEntry {
  id: number;
  word: string;
  ipa: string | null;
  ipaUs: string | null;
  ipaUk: string | null;
  pos: string | null;
  meaningVi: string | null;
  meaningEn: string | null;
  exampleSentence: string | null;
  audioUrl: string | null;
  source: string;
  matchedForm: string | null;
  variants: string[];
  sources: string[];
  senses: DictionarySense[];
}

export interface DictPackInfo {
  installed: boolean;
  profile: string | null;
  wordCount: number;
  senseCount: number;
  formCount: number;
  builtAt: string | null;
  attribution: string | null;
}

export interface TtsSettings {
  fallbackOrder: string[];
  voice: string;
  providerVoices: Record<string, string>;
}

export interface AiSettings {
  baseUrl: string;
  apiKey: string;
  model: string;
}

export interface AiPracticeFeedback {
  feedback: string;
}

export interface GoogleDriveStatus {
  connected: boolean;
  userEmail: string;
  clientId: string;
  hasClientSecret: boolean;
  autoSync: boolean;
}

export interface GoogleOAuthStart {
  authUrl: string;
}

export interface GoogleDriveBackupInfo {
  exists: boolean;
  modifiedTime: string | null;
  sizeBytes: number | null;
}

export interface GoogleDrivePushResult {
  pushed: boolean;
  conflict: boolean;
  remoteModifiedTime: string | null;
}

export interface LocalBackupEntry {
  filename: string;
  createdAt: number;
  sizeBytes: number;
  reason: string;
}

export interface ApkgImportReport {
  deckId: string;
  deckName: string;
  imported: number;
  skipped: number;
}

export interface HeatmapDay {
  date: string;
  count: number;
}

export interface ForecastDay {
  date: string;
  dueCount: number;
}

export interface RatingCounts {
  again: number;
  hard: number;
  good: number;
  easy: number;
}

export interface LeechWord {
  word: string;
  meaning: string;
  lapses: number;
  reps: number;
}

export interface AnalyticsSummary {
  streakDays: number;
  totalReviews: number;
  trueRetention: number;
  forgetRate: number;
  matureCards: number;
  totalNotes: number;
  totalCards: number;
  newCards: number;
  learningCards: number;
  youngCards: number;
  ratingCounts: RatingCounts;
  totalStudyMs: number;
  avgDurationMs: number;
  studyMsToday: number;
  hourlyWeekdayHeatmap: number[][];
  peakHour: number | null;
  peakWeekday: number | null;
  leechWords: LeechWord[];
  heatmap: HeatmapDay[];
  forecast7d: ForecastDay[];
}

export interface WebDavSettings {
  url: string;
  username: string;
  password: string;
}

export interface BackupRestoreReport {
  decks: number;
  notes: number;
  cards: number;
  reviewLogs: number;
}
