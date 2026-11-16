import { invokeCmd } from "$lib/shared/tauri";
import type { Card, DailyStats, Rating, StudyCard } from "$lib/shared/types";

export const studyApi = {
  getQueue: (deckId: string, limit = 50) =>
    invokeCmd<StudyCard[]>("get_study_queue", { deckId, limit }),
  submitReview: (cardId: string, rating: Rating, durationMs?: number) =>
    invokeCmd<void>("submit_review", { cardId, rating, durationMs }),
  undoReview: (prevCard: Card) => invokeCmd<void>("undo_review", { prevCard }),
  getDailyStats: () => invokeCmd<DailyStats>("get_daily_stats"),
};
