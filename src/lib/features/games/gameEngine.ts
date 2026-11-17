import type { Note } from "../../shared/types";

export interface MatchCard {
  id: string;
  text: string;
  subText?: string;
}

export interface MatchRound {
  left: MatchCard[];
  right: MatchCard[];
}

export interface QuizOption {
  id: string;
  label: string;
  isCorrect: boolean;
}

export interface QuizQuestion {
  note: Note;
  prompt: string;
  subPrompt?: string;
  options: QuizOption[];
}

export type GameModeId = "match" | "quiz" | "audio";

const SCORES_KEY = "vlearning_game_scores";

export function shuffleArray<T>(items: T[]): T[] {
  const arr = [...items];
  for (let i = arr.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [arr[i], arr[j]] = [arr[j], arr[i]];
  }
  return arr;
}

export function buildMatchRound(notes: Note[], pairCount = 6): MatchRound {
  const valid = notes.filter((n) => n.word.trim() && n.meaning.trim());
  const chosen = shuffleArray(valid).slice(0, pairCount);
  const left = shuffleArray(
    chosen.map((n) => ({
      id: n.id,
      text: n.word,
      subText: n.ipa ?? undefined,
    })),
  );
  const right = shuffleArray(
    chosen.map((n) => ({
      id: n.id,
      text: n.meaning,
    })),
  );
  return { left, right };
}

export function buildQuizQuestion(
  target: Note,
  pool: Note[],
  direction: "en-to-vi" | "vi-to-en" = "en-to-vi",
): QuizQuestion {
  const targetLabel = direction === "en-to-vi" ? target.meaning : target.word;
  const seenLabels = new Set<string>([targetLabel.trim().toLowerCase()]);
  const distractors: QuizOption[] = [];

  for (const candidate of shuffleArray(pool)) {
    if (candidate.id === target.id) continue;
    const label = (direction === "en-to-vi" ? candidate.meaning : candidate.word).trim();
    const key = label.toLowerCase();
    if (!label || seenLabels.has(key)) continue;
    seenLabels.add(key);
    distractors.push({ id: candidate.id, label, isCorrect: false });
    if (distractors.length >= 3) break;
  }

  const options = shuffleArray([
    { id: target.id, label: targetLabel.trim(), isCorrect: true },
    ...distractors,
  ]);

  return {
    note: target,
    prompt: direction === "en-to-vi" ? target.word : target.meaning,
    subPrompt: direction === "en-to-vi" ? (target.ipa ?? undefined) : undefined,
    options,
  };
}

export function loadBestScores(): Record<string, number> {
  if (typeof localStorage === "undefined") return {};
  try {
    const raw = localStorage.getItem(SCORES_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw);
    return parsed && typeof parsed === "object" ? parsed : {};
  } catch {
    return {};
  }
}

export function saveBestScore(mode: GameModeId, value: number, lowerIsBetter = false): number {
  const scores = loadBestScores();
  const prev = scores[mode];
  const isBetter = prev === undefined ? true : lowerIsBetter ? value < prev : value > prev;
  const best = isBetter ? value : prev;
  scores[mode] = best;
  if (typeof localStorage !== "undefined") {
    localStorage.setItem(SCORES_KEY, JSON.stringify(scores));
  }
  return best;
}
