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

export interface DictionaryEntry {
  id: number;
  word: string;
  ipa: string | null;
  pos: string | null;
  meaningVi: string | null;
  meaningEn: string | null;
  exampleSentence: string | null;
  audioUrl: string | null;
  source: string;
}

export interface DictionaryPack {
  id: string;
  name: string;
  description: string;
  sizeHint: string;
}

export interface ImportReport {
  inserted: number;
  updated: number;
  skipped: number;
  totalAfter: number;
}

export interface ImportProgress {
  stage: "downloading" | "importing" | "done";
  current: number;
  total: number;
  message: string;
}
